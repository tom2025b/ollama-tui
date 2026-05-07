use ai_suite::ConversationTurn;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    User,
    Assistant,
    Local,
}

impl Role {
    pub fn label(self) -> &'static str {
        match self {
            Role::User => "You",
            Role::Assistant => "Assistant",
            Role::Local => "ai-suite",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
    pub complete: bool,
    pub is_error: bool,
    pub model_label: Option<String>,
}

impl ChatMessage {
    pub fn user(content: String) -> Self {
        Self {
            role: Role::User,
            content,
            complete: true,
            is_error: false,
            model_label: None,
        }
    }

    pub fn assistant(model_label: String) -> Self {
        Self {
            role: Role::Assistant,
            content: String::new(),
            complete: false,
            is_error: false,
            model_label: Some(model_label),
        }
    }

    pub fn local(content: String) -> Self {
        Self {
            role: Role::Local,
            content,
            complete: true,
            is_error: false,
            model_label: None,
        }
    }
}

pub fn conversation_context(messages: &[ChatMessage]) -> Vec<ConversationTurn> {
    let mut turns = Vec::new();
    let mut pending_user: Option<String> = None;

    for message in messages
        .iter()
        .filter(|message| message.complete && !message.is_error)
    {
        match message.role {
            Role::User => pending_user = Some(message.content.clone()),
            Role::Assistant => {
                if let Some(user) = pending_user.take() {
                    turns.push(ConversationTurn {
                        user,
                        assistant: message.content.clone(),
                    });
                }
            }
            Role::Local => {}
        }
    }

    turns
}
