use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use crate::llm::ConversationTurn;
use crate::runtime::RuntimePaths;

const MAX_STORED_MEMORY_ITEMS: usize = 64;

/// Project-scoped memories that survive across TUI sessions.
#[derive(Clone, Debug, Default)]
pub(crate) struct MemoryStore {
    path: PathBuf,
    items: Vec<MemoryItem>,
}

impl MemoryStore {
    pub(crate) fn load(paths: &RuntimePaths) -> Self {
        Self::load_from_path(paths.project_memory_path())
    }

    pub(crate) fn load_from_path(path: &Path) -> Self {
        let items = fs::read_to_string(path)
            .ok()
            .and_then(|content| serde_json::from_str::<MemoryFile>(&content).ok())
            .map(|file| file.items)
            .unwrap_or_default();

        Self {
            path: path.to_path_buf(),
            items,
        }
    }

    /// Returns only turn-type items projected as conversation exchanges.
    pub(crate) fn turns(&self) -> Vec<ConversationTurn> {
        self.items
            .iter()
            .filter_map(|item| {
                if let MemoryItem::Turn {
                    user, assistant, ..
                } = item
                {
                    Some(ConversationTurn {
                        user: user.clone(),
                        assistant: assistant.clone(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns a formatted prefix for note-type items, or an empty string if none exist.
    ///
    /// Notes are injected into prompts as a prefix so the model has project context
    /// without polluting the conversation history with fake exchanges.
    pub(crate) fn notes_prompt_prefix(&self) -> String {
        let notes: Vec<&str> = self
            .items
            .iter()
            .filter_map(|item| {
                if let MemoryItem::Note { content, .. } = item {
                    Some(content.as_str())
                } else {
                    None
                }
            })
            .collect();

        if notes.is_empty() {
            return String::new();
        }

        let mut prefix = String::from("[Project notes]\n");
        for note in &notes {
            prefix.push_str("- ");
            prefix.push_str(note);
            prefix.push('\n');
        }
        prefix
    }

    pub(crate) fn items(&self) -> &[MemoryItem] {
        &self.items
    }

    pub(crate) fn remember_turn(&mut self, user: &str, assistant: &str) -> io::Result<()> {
        self.items.push(MemoryItem::Turn {
            user: user.to_string(),
            assistant: assistant.to_string(),
            saved_at: timestamp_seconds(),
        });
        self.trim();
        self.save()
    }

    pub(crate) fn remember_note(&mut self, note: &str) -> io::Result<()> {
        self.items.push(MemoryItem::Note {
            content: note.to_string(),
            saved_at: timestamp_seconds(),
        });
        self.trim();
        self.save()
    }

    pub(crate) fn forget_latest_turn(&mut self, user: &str) -> io::Result<bool> {
        let Some(index) = self
            .items
            .iter()
            .rposition(|item| matches!(item, MemoryItem::Turn { user: u, .. } if u == user))
        else {
            return Ok(false);
        };

        self.items.remove(index);
        self.save()?;
        Ok(true)
    }

    pub(crate) fn clear(&mut self) -> io::Result<usize> {
        let count = self.items.len();
        self.items.clear();
        self.save()?;
        Ok(count)
    }

    fn trim(&mut self) {
        let overflow = self.items.len().saturating_sub(MAX_STORED_MEMORY_ITEMS);
        if overflow > 0 {
            self.items.drain(0..overflow);
        }
    }

    fn save(&self) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = MemoryFile {
            items: self.items.clone(),
        };
        let content = serde_json::to_string_pretty(&file)?;
        fs::write(&self.path, content)
    }
}

/// A single memory entry — either a persisted conversation turn or a project note.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum MemoryItem {
    /// A real user/assistant exchange selected for persistent context.
    Turn {
        user: String,
        assistant: String,
        saved_at: u64,
    },
    /// A durable project note injected as a prompt prefix.
    Note { content: String, saved_at: u64 },
}

impl MemoryItem {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::Turn { .. } => "turn",
            Self::Note { .. } => "note",
        }
    }

    /// The human-readable content of this item, used in display contexts.
    pub(crate) fn display_content(&self) -> &str {
        match self {
            Self::Turn { assistant, .. } => assistant,
            Self::Note { content, .. } => content,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct MemoryFile {
    items: Vec<MemoryItem>,
}

fn timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_round_trips_turns_and_notes() {
        let path = unique_path("round-trip");
        let mut store = MemoryStore::load_from_path(&path);

        store
            .remember_turn("how should this work?", "make it modular")
            .expect("turn should save");
        store
            .remember_note("Prefer terminal launches over API keys.")
            .expect("note should save");

        let loaded = MemoryStore::load_from_path(&path);
        assert_eq!(loaded.items().len(), 2);

        // turns() returns only Turn items, not Notes
        assert_eq!(loaded.turns().len(), 1);
        assert_eq!(loaded.turns()[0].user, "how should this work?");

        let _ = fs::remove_file(path);
    }

    #[test]
    fn notes_produce_prompt_prefix_turns_do_not() {
        let path = unique_path("prefix");
        let mut store = MemoryStore::load_from_path(&path);

        store.remember_note("Use async Rust.").unwrap();
        store.remember_turn("why?", "because performance").unwrap();

        let prefix = store.notes_prompt_prefix();
        assert!(prefix.contains("Use async Rust."));
        assert!(!prefix.contains("why?"));
        assert!(!prefix.contains("because performance"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn notes_prefix_is_empty_when_no_notes_exist() {
        let path = unique_path("no-notes");
        let mut store = MemoryStore::load_from_path(&path);
        store.remember_turn("q", "a").unwrap();

        assert!(store.notes_prompt_prefix().is_empty());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn forget_latest_turn_matches_user_prompt() {
        let path = unique_path("forget");
        let mut store = MemoryStore::load_from_path(&path);
        store.remember_turn("prompt", "first").unwrap();
        store.remember_turn("prompt", "second").unwrap();

        assert!(store.forget_latest_turn("prompt").unwrap());

        let loaded = MemoryStore::load_from_path(&path);
        assert_eq!(loaded.items().len(), 1);
        assert_eq!(
            loaded.turns()[0].assistant,
            "first",
            "should retain the earlier turn"
        );

        let _ = fs::remove_file(path);
    }

    fn unique_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();

        std::env::temp_dir().join(format!(
            "ai-suite-memory-{name}-{}-{unique}.json",
            std::process::id()
        ))
    }
}
