use crate::theme::{SystemTheme, ThemeColors};
use gpui::*;
use winxime_config::{
    deploy_all, get_data_dirs, SchemaConfig, SchemaConfigManager, SchemaInfo, SchemaManager,
    SmartSuggestionConfig, XimeConfig,
};
use winxime_ipc::IpcClient;
use winxime_predict::check_model_exists;

pub struct SettingsState {
    pub appearance: AppearanceState,
    pub input_schema: InputSchemaState,
    pub clipboard: ClipboardState,
    pub smart_suggestion: SmartSuggestionState,
    pub system_theme: SystemTheme,
    pub deploy_message: Option<String>,
    pub deploy_message_time: Option<std::time::Instant>,
    pub schemas_loaded: bool,
}

impl SettingsState {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut state = Self {
            appearance: AppearanceState::default(),
            input_schema: InputSchemaState::default(),
            clipboard: ClipboardState::default(),
            smart_suggestion: Self::load_smart_suggestion_config(),
            system_theme: SystemTheme::detect(),
            deploy_message: None,
            deploy_message_time: None,
            schemas_loaded: false,
        };
        state.load_color_schemes(cx);
        state
    }

    fn load_smart_suggestion_config() -> SmartSuggestionState {
        let config = XimeConfig::load();
        let model_name = config.smart_suggestion.model.name.clone();

        SmartSuggestionState {
            enabled: config.smart_suggestion.enabled.unwrap_or(false),
            suggestion_count: config.smart_suggestion.suggestion_count,
            record_user_frequency: config.smart_suggestion.record_user_frequency,
            auto_adjust_frequency: config.smart_suggestion.auto_adjust_frequency,
            learning_threshold: config.smart_suggestion.learning_threshold,
            auto_download: config.smart_suggestion.model.auto_download,
            model_name: model_name.clone(),
            model_downloaded: check_model_exists(Some(&model_name)),
            downloading: false,
        }
    }

    pub fn load_schemas(&mut self, cx: &mut Context<Self>) {
        if self.schemas_loaded {
            return;
        }
        if let Ok(manager) = SchemaManager::new() {
            let schemas = manager.get_schema_list();
            self.input_schema.available_schemas = schemas;
            self.schemas_loaded = true;
            cx.notify();
        }
    }

    pub fn load_schema_config(&mut self, cx: &mut Context<Self>) {
        if self.input_schema.config_loaded {
            return;
        }
        if self.input_schema.selected_schema >= self.input_schema.available_schemas.len() {
            return;
        }
        let schema_id =
            &self.input_schema.available_schemas[self.input_schema.selected_schema].schema_id;
        if let Ok(manager) = SchemaConfigManager::new(schema_id) {
            self.input_schema.schema_config = Some(manager.get_config());
            self.input_schema.config_loaded = true;
            cx.notify();
        }
    }

    fn load_appearance_config_safe() -> AppearanceState {
        AppearanceState::default()
    }

    fn load_schema_config_safe() -> InputSchemaState {
        InputSchemaState::default()
    }

    fn load_appearance_config() -> AppearanceState {
        let config = XimeConfig::load();
        AppearanceState {
            font_size: config.style.font_size as f64,
            candidate_count: config.style.candidate_count,
            horizontal: config.style.horizontal,
            color_scheme: config.style.color_scheme,
            available_color_schemes: Vec::new(),
            color_schemes_loaded: false,
        }
    }

    fn load_input_schema_config() -> InputSchemaState {
        if let Ok(manager) = SchemaManager::new() {
            let schemas = manager.get_schema_list();
            let selected_schema = manager
                .get_selected_schema()
                .and_then(|selected_id| schemas.iter().position(|s| s.schema_id == selected_id))
                .unwrap_or(0);
            InputSchemaState {
                selected_schema,
                available_schemas: schemas,
                schema_config: None,
                config_loaded: false,
            }
        } else {
            InputSchemaState::default()
        }
    }

    pub fn colors(&self) -> ThemeColors {
        let primary_color = self.get_primary_color();
        ThemeColors::from_theme(&self.system_theme, primary_color)
    }

    fn get_primary_color(&self) -> u32 {
        self.appearance
            .available_color_schemes
            .iter()
            .find(|(id, _, _)| id == &self.appearance.color_scheme)
            .map(|(_, _, color)| *color)
            .unwrap_or(0x8F73E2)
    }

    pub fn load_color_schemes(&mut self, cx: &mut Context<Self>) {
        if self.appearance.color_schemes_loaded {
            return;
        }
        let config = XimeConfig::load();
        self.appearance.color_scheme = config.style.color_scheme;
        self.appearance.available_color_schemes = config
            .color_schemes
            .iter()
            .map(|(id, scheme)| (id.clone(), scheme.name.clone(), scheme.primary_color))
            .collect();
        self.appearance.font_size = config.style.font_size as f64;
        self.appearance.candidate_count = config.style.candidate_count;
        self.appearance.horizontal = config.style.horizontal;
        self.appearance.color_schemes_loaded = true;
        cx.notify();
    }

    pub fn save_color_scheme(&self) -> Result<(), String> {
        let config = XimeConfig::load();
        let updated = XimeConfig {
            wubi_radicals: config.wubi_radicals,
            style: winxime_config::StyleConfig {
                font_family: config.style.font_family,
                font_size: config.style.font_size,
                candidate_count: config.style.candidate_count,
                horizontal: config.style.horizontal,
                corner_radius: config.style.corner_radius,
                color_scheme: self.appearance.color_scheme.clone(),
            },
            color_schemes: config.color_schemes,
            smart_suggestion: config.smart_suggestion,
        };
        updated.save()
    }

    pub fn save_appearance(&self) -> Result<(), String> {
        let config = XimeConfig::load();
        let updated = XimeConfig {
            wubi_radicals: config.wubi_radicals,
            style: winxime_config::StyleConfig {
                font_family: config.style.font_family,
                font_size: self.appearance.font_size as f32,
                candidate_count: self.appearance.candidate_count,
                horizontal: self.appearance.horizontal,
                corner_radius: config.style.corner_radius,
                color_scheme: self.appearance.color_scheme.clone(),
            },
            color_schemes: config.color_schemes,
            smart_suggestion: config.smart_suggestion,
        };
        updated.save()?;

        if !IpcClient::reload_config() {
            eprintln!("Server not running, appearance will apply on next start");
        }
        Ok(())
    }

    pub fn save_schema(&self) -> Result<(), String> {
        if self.input_schema.selected_schema < self.input_schema.available_schemas.len() {
            let selected_id =
                &self.input_schema.available_schemas[self.input_schema.selected_schema].schema_id;

            let schema_manager = SchemaManager::new()?;
            schema_manager.set_schema_list(&[selected_id])?;
            schema_manager.save()?;

            deploy_all().map_err(|e| e.to_string())?;

            if !IpcClient::reload_config() {
                eprintln!("Server not running, config will apply on next start");
            }
        }
        Ok(())
    }

    pub fn save_schema_config(&self) -> Result<(), String> {
        if self.input_schema.selected_schema >= self.input_schema.available_schemas.len() {
            return Ok(());
        }

        let schema_id =
            &self.input_schema.available_schemas[self.input_schema.selected_schema].schema_id;
        let manager = SchemaConfigManager::new(schema_id)?;

        if let Some(config) = &self.input_schema.schema_config {
            if let Some(v) = config.speller.max_code_length {
                manager.set_int("speller/max_code_length", v)?;
            }
            if let Some(v) = config.speller.auto_select {
                manager.set_bool("speller/auto_select", v)?;
            }
            if let Some(v) = &config.speller.auto_clear {
                if !v.is_empty() {
                    manager.set_string("speller/auto_clear", v)?;
                }
            }

            if let Some(v) = config.translator.enable_charset_filter {
                manager.set_bool("translator/enable_charset_filter", v)?;
            }
            if let Some(v) = config.translator.enable_completion {
                manager.set_bool("translator/enable_completion", v)?;
            }
            if let Some(v) = config.translator.enable_sentence {
                manager.set_bool("translator/enable_sentence", v)?;
            }
            if let Some(v) = config.translator.enable_user_dict {
                manager.set_bool("translator/enable_user_dict", v)?;
            }
            if let Some(v) = config.translator.enable_encoder {
                manager.set_bool("translator/enable_encoder", v)?;
            }
            if let Some(v) = config.translator.encode_commit_history {
                manager.set_bool("translator/encode_commit_history", v)?;
            }
            if let Some(v) = config.translator.max_phrase_length {
                manager.set_int("translator/max_phrase_length", v)?;
            }

            if let Some(v) = &config.reverse_lookup.prefix {
                manager.set_string("reverse_lookup/prefix", v)?;
            }
            if let Some(v) = &config.reverse_lookup.suffix {
                manager.set_string("reverse_lookup/suffix", v)?;
            }

            if let Some(v) = &config.tradition.opencc_config {
                manager.set_string("tradition/opencc_config", v)?;
            }

            manager.save()?;
        }

        Ok(())
    }

    pub fn save_clipboard(&self) -> Result<(), String> {
        Ok(())
    }

    pub fn save_smart_suggestion(&self) -> Result<(), String> {
        let config = XimeConfig::load();

        let updated = XimeConfig {
            wubi_radicals: config.wubi_radicals,
            style: config.style,
            color_schemes: config.color_schemes,
            smart_suggestion: SmartSuggestionConfig {
                enabled: Some(self.smart_suggestion.enabled),
                suggestion_count: self.smart_suggestion.suggestion_count,
                record_user_frequency: self.smart_suggestion.record_user_frequency,
                auto_adjust_frequency: self.smart_suggestion.auto_adjust_frequency,
                learning_threshold: self.smart_suggestion.learning_threshold,
                model: winxime_config::SmartSuggestionModelConfig {
                    provider: "modelscope".to_string(),
                    name: self.smart_suggestion.model_name.clone(),
                    auto_download: self.smart_suggestion.auto_download,
                    files: vec![],
                },
            },
        };

        updated.save()
    }

    pub fn deploy(&mut self) -> Result<(), String> {
        let result = deploy_all().map_err(|e| e.to_string());
        match &result {
            Ok(_) => {
                if IpcClient::reload_config() {
                    self.deploy_message = Some("部署成功！配置已重载。".to_string());
                } else {
                    self.deploy_message =
                        Some("部署成功！(服务器未运行，配置将在下次启动时生效)".to_string());
                }
            }
            Err(e) => {
                self.deploy_message = Some(format!("部署失败: {}", e));
            }
        }
        self.deploy_message_time = Some(std::time::Instant::now());
        result
    }

    pub fn clear_deploy_message(&mut self) {
        self.deploy_message = None;
        self.deploy_message_time = None;
    }
}

#[derive(Clone, Default)]
pub struct AppearanceState {
    pub font_size: f64,
    pub candidate_count: i32,
    pub horizontal: bool,
    pub color_scheme: String,
    pub available_color_schemes: Vec<(String, String, u32)>,
    pub color_schemes_loaded: bool,
}

#[derive(Clone, Default)]
pub struct InputSchemaState {
    pub selected_schema: usize,
    pub available_schemas: Vec<SchemaInfo>,
    pub schema_config: Option<SchemaConfig>,
    pub config_loaded: bool,
}

#[derive(Clone, Default)]
pub struct ClipboardState {
    pub enabled: bool,
    pub history_count: i32,
    pub retention_days: i32,
}

#[derive(Clone)]
pub struct SmartSuggestionState {
    pub enabled: bool,
    pub suggestion_count: i32,
    pub record_user_frequency: bool,
    pub auto_adjust_frequency: bool,
    pub learning_threshold: i32,
    pub auto_download: bool,
    pub model_name: String,
    pub model_downloaded: bool,
    pub downloading: bool,
}

impl Default for SmartSuggestionState {
    fn default() -> Self {
        Self {
            enabled: false,
            suggestion_count: 5,
            record_user_frequency: true,
            auto_adjust_frequency: true,
            learning_threshold: 3,
            auto_download: true,
            model_name: "predictive-text-small".to_string(),
            model_downloaded: false,
            downloading: false,
        }
    }
}
