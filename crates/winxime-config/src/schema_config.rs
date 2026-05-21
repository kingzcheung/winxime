use crate::rime_deploy::init_rime_deployer;
use librime::levers::CustomSettings;
use serde::Deserialize;

pub struct SchemaConfigManager {
    settings: CustomSettings,
}

impl SchemaConfigManager {
    pub fn new(schema_id: &str) -> Result<Self, String> {
        init_rime_deployer()?;
        let settings = CustomSettings::new(schema_id, "Xime::SchemaConfigManager")
            .map_err(|e| e.to_string())?;
        Ok(Self { settings })
    }

    pub fn get_config(&self) -> SchemaConfig {
        SchemaConfig {
            speller: SpellerConfig {
                max_code_length: self.settings.get_int("speller/max_code_length"),
                auto_select: self.settings.get_bool("speller/auto_select"),
                auto_clear: self.settings.get_string("speller/auto_clear"),
                alphabet: self.settings.get_string("speller/alphabet"),
                delimiter: self.settings.get_string("speller/delimiter"),
            },
            translator: TranslatorConfig {
                enable_charset_filter: self.settings.get_bool("translator/enable_charset_filter"),
                enable_completion: self.settings.get_bool("translator/enable_completion"),
                enable_sentence: self.settings.get_bool("translator/enable_sentence"),
                enable_user_dict: self.settings.get_bool("translator/enable_user_dict"),
                enable_encoder: self.settings.get_bool("translator/enable_encoder"),
                encode_commit_history: self.settings.get_bool("translator/encode_commit_history"),
                max_phrase_length: self.settings.get_int("translator/max_phrase_length"),
            },
            reverse_lookup: ReverseLookupConfig {
                prefix: self.settings.get_string("reverse_lookup/prefix"),
                suffix: self.settings.get_string("reverse_lookup/suffix"),
                tips: self.settings.get_string("reverse_lookup/tips"),
            },
            tradition: TraditionConfig {
                opencc_config: self.settings.get_string("tradition/opencc_config"),
            },
        }
    }

    pub fn set_int(&self, key: &str, value: i32) -> Result<(), String> {
        self.settings.set_int(key, value).map_err(|e| e.to_string())
    }

    pub fn set_bool(&self, key: &str, value: bool) -> Result<(), String> {
        self.settings
            .set_bool(key, value)
            .map_err(|e| e.to_string())
    }

    pub fn set_string(&self, key: &str, value: &str) -> Result<(), String> {
        self.settings
            .set_string(key, value)
            .map_err(|e| e.to_string())
    }

    pub fn save(&self) -> Result<(), String> {
        self.settings.save().map_err(|e| e.to_string())
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SpellerConfig {
    pub max_code_length: Option<i32>,
    pub auto_select: Option<bool>,
    pub auto_clear: Option<String>,
    pub alphabet: Option<String>,
    pub delimiter: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct TranslatorConfig {
    pub enable_charset_filter: Option<bool>,
    pub enable_completion: Option<bool>,
    pub enable_sentence: Option<bool>,
    pub enable_user_dict: Option<bool>,
    pub enable_encoder: Option<bool>,
    pub encode_commit_history: Option<bool>,
    pub max_phrase_length: Option<i32>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ReverseLookupConfig {
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub tips: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct TraditionConfig {
    pub opencc_config: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SchemaConfig {
    pub speller: SpellerConfig,
    pub translator: TranslatorConfig,
    pub reverse_lookup: ReverseLookupConfig,
    pub tradition: TraditionConfig,
}
