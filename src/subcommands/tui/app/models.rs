use super::App;
use crate::llm::LanguageModel;
use crate::router::Router;

impl App {
    /// Return the list of models that can be displayed by the UI.
    pub fn models(&self) -> &[LanguageModel] {
        self.routing.router.models()
    }

    /// All models the picker is allowed to offer for pinning.
    pub fn pickable_models(&self) -> Vec<&LanguageModel> {
        self.routing
            .router
            .models()
            .iter()
            .filter(|model| model.enabled)
            .collect()
    }

    /// Total number of rows shown by the picker, including the leading "Auto".
    pub fn models_picker_total(&self) -> usize {
        self.pickable_models().len() + 1
    }

    /// Currently highlighted picker row, clamped to the live entry list.
    pub fn models_picker_index(&self) -> usize {
        let total = self.models_picker_total();
        if total == 0 {
            0
        } else {
            self.ui.models_picker_index.min(total - 1)
        }
    }

    /// True when the given model is the one currently pinned.
    pub fn is_pinned(&self, model: &LanguageModel) -> bool {
        match &self.routing.pinned_model {
            Some(pinned) => pinned.provider == model.provider && pinned.name == model.name,
            None => false,
        }
    }

    /// Open the interactive `/model` picker overlay.
    pub fn open_models_picker(&mut self) {
        self.ui.show_models_picker = true;
        self.commands.dismiss_suggestions();
        self.ui.models_picker_index = match &self.routing.pinned_model {
            None => 0,
            Some(pinned) => self
                .pickable_models()
                .iter()
                .position(|model| model.provider == pinned.provider && model.name == pinned.name)
                .map(|i| i + 1)
                .unwrap_or(0),
        };
        self.ui.status =
            "Pick a model. Up/Down to navigate, Enter to select, Esc to cancel.".to_string();
    }

    /// Close the picker without applying any change.
    pub fn close_models_picker(&mut self) {
        if !self.ui.show_models_picker {
            return;
        }
        self.ui.show_models_picker = false;
        self.ui.status = "Model picker cancelled.".to_string();
    }

    /// Move the picker highlight up by one row, wrapping at the top.
    pub fn select_previous_model(&mut self) {
        let total = self.models_picker_total();
        if total == 0 {
            return;
        }
        let current = self.ui.models_picker_index.min(total - 1);
        self.ui.models_picker_index = if current == 0 { total - 1 } else { current - 1 };
    }

    /// Move the picker highlight down by one row, wrapping at the bottom.
    pub fn select_next_model(&mut self) {
        let total = self.models_picker_total();
        if total == 0 {
            return;
        }
        let current = self.ui.models_picker_index.min(total - 1);
        self.ui.models_picker_index = (current + 1) % total;
    }

    /// Apply the highlighted picker row.
    pub fn accept_model_selection(&mut self) {
        let total = self.models_picker_total();
        if total == 0 {
            return;
        }
        let index = self.ui.models_picker_index.min(total - 1);

        if index == 0 {
            self.routing.pinned_model = None;
            self.ui.status = "Routing reset to Auto. Router will pick per prompt.".to_string();
        } else {
            let chosen = self.pickable_models()[index - 1].clone();
            let label = chosen.display_label();
            self.routing.pinned_model = Some(chosen);
            self.ui.status = format!("Pinned to {label}. New prompts will skip the router.");
        }

        self.ui.show_models_picker = false;
    }
}
