use gpui::*;

pub struct SettingsState {
    pub appearance: AppearanceState,
    pub input_schema: InputSchemaState,
    pub clipboard: ClipboardState,
    pub smart_suggestion: SmartSuggestionState,
}

impl SettingsState {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            appearance: AppearanceState::default(),
            input_schema: InputSchemaState::default(),
            clipboard: ClipboardState::default(),
            smart_suggestion: SmartSuggestionState::default(),
        }
    }
}

#[derive(Clone, Default)]
pub struct AppearanceState {
    pub font_size: f64,
    pub candidate_count: i32,
    pub show_code_hint: bool,
    pub corner_radius: f64,
}

#[derive(Clone, Default)]
pub struct InputSchemaState {
    pub selected_schema: usize,
}

#[derive(Clone, Default)]
pub struct ClipboardState {
    pub enabled: bool,
    pub history_count: i32,
    pub retention_days: i32,
}

#[derive(Clone, Default)]
pub struct SmartSuggestionState {
    pub enabled: bool,
    pub suggestion_count: i32,
    pub prefer_common_words: bool,
    pub record_user_frequency: bool,
    pub auto_adjust_frequency: bool,
    pub learning_threshold: i32,
}