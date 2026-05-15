use librime::{
    get_api, setup, initialize, Traits,
    create_session, start_maintenance, join_maintenance_thread,
    levers::CustomSettings,
};
use librime::levers::SwitcherSettings;
pub use librime::levers::SchemaInfo;
pub use librime::levers::deploy_all;
use std::ffi::CString;
use std::io::Write;
use std::sync::Once;
use serde::Deserialize;

static RIME_INIT: Once = Once::new();

fn init_rime_deployer() -> Result<(), String> {
    RIME_INIT.call_once(|| {
        let (shared_data_dir, user_data_dir) = get_data_dirs();
        
        ensure_user_config_files(&user_data_dir);
        
        let mut traits = Traits::new();
        traits
            .set_shared_data_dir(shared_data_dir.to_str().unwrap_or(""))
            .set_user_data_dir(user_data_dir.to_str().unwrap_or(""))
            .set_distribution_name("Xime")
            .set_distribution_code_name("Xime")
            .set_distribution_version("1.0")
            .set_app_name("rime.xime.setup")
            .set_min_log_level(2);

        setup(&mut traits);
        
        if initialize(&mut traits).is_err() {
            return;
        }

        if start_maintenance(true).is_ok() {
            join_maintenance_thread();
        }

        if let Ok(session) = create_session() {
            drop(session);
        }

        unsafe {
            let api = get_api();
            if !api.is_null() {
                if let Some(deploy_config) = (*api).deploy_config_file {
                    let config_file = CString::new("xime.yaml").unwrap_or_default();
                    let version_key = CString::new("config_version").unwrap_or_default();
                    deploy_config(config_file.as_ptr(), version_key.as_ptr());
                }
            }
        }
    });

    Ok(())
}

const DEFAULT_USER_CONFIG_URL: &str =
    "https://github.com/kingzcheung/rime-wubi/archive/refs/tags/2.0.0.tar.gz";

fn ensure_user_config_files(user_data_dir: &std::path::Path) {
    if !user_data_dir.exists() {
        std::fs::create_dir_all(user_data_dir).ok();
    }
    
    let default_custom = user_data_dir.join("default.custom.yaml");
    let wubi_schema = user_data_dir.join("wubi86.schema.yaml");
    let wubi_dict = user_data_dir.join("wubi86.dict.yaml");
    
    if !default_custom.exists() || !wubi_schema.exists() || !wubi_dict.exists() {
        eprintln!("User configs missing, downloading from {}", DEFAULT_USER_CONFIG_URL);
        download_and_extract_user_configs(user_data_dir);
    }
    
    let config_source_dir = get_config_source_dir();
    
    let xime_yaml = user_data_dir.join("xime.yaml");
    if !xime_yaml.exists() {
        let source = config_source_dir.join("xime.yaml");
        if source.exists() {
            std::fs::copy(&source, &xime_yaml).ok();
        }
    }
    
    let xime_custom = user_data_dir.join("xime.custom.yaml");
    if !xime_custom.exists() {
        std::fs::write(&xime_custom, 
r#"customization:
  distribution_code_name: Xime
  distribution_version: 1.0
  generator: "Xime::ConfigManager"
  rime_version: 1.16.1

patch: {}
"#).ok();
    }
}

fn download_and_extract_user_configs(user_data_dir: &std::path::Path) {
    use std::fs::File;
    use std::io::Read;
    use flate2::read::GzDecoder;
    use tar::Archive;
    
    let temp_dir = user_data_dir.join(".temp_download");
    std::fs::create_dir_all(&temp_dir).ok();
    
    let tar_path = temp_dir.join("rime-wubi.tar.gz");
    
    let response = ureq::get(DEFAULT_USER_CONFIG_URL).call();
    if response.is_err() {
        eprintln!("Failed to download: {:?}", response.err());
        std::fs::remove_dir_all(&temp_dir).ok();
        return;
    }
    let response = response.unwrap();
    
    let file = File::create(&tar_path).ok();
    if file.is_none() {
        std::fs::remove_dir_all(&temp_dir).ok();
        return;
    }
    let mut file = file.unwrap();
    
    let mut buffer = Vec::new();
    if response.into_reader().read_to_end(&mut buffer).is_err() {
        std::fs::remove_dir_all(&temp_dir).ok();
        return;
    }
    file.write_all(&buffer).ok();
    
    eprintln!("Extracting...");
    
    let file = File::open(&tar_path).ok();
    if file.is_none() {
        std::fs::remove_dir_all(&temp_dir).ok();
        return;
    }
    let file = file.unwrap();
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    
    if let Ok(entries) = archive.entries() {
        for entry_result in entries {
            if let Ok(mut entry) = entry_result {
                if let Ok(path) = entry.path() {
                    let path_str = path.to_string_lossy();
                    
                    if path_str.contains("rime-wubi-1.0.0/") {
                        let relative_path = path_str
                            .strip_prefix("rime-wubi-1.0.0/")
                            .unwrap_or(&path_str);
                        
                        if !relative_path.is_empty() && !relative_path.ends_with('/') {
                            let dest = user_data_dir.join(relative_path);
                            
                            if let Some(parent) = dest.parent() {
                                std::fs::create_dir_all(parent).ok();
                            }
                            
                            entry.unpack(dest).ok();
                        }
                    }
                }
            }
        }
    }
    
    std::fs::remove_dir_all(&temp_dir).ok();
    eprintln!("User configs installed");
}

fn get_data_dirs() -> (std::path::PathBuf, std::path::PathBuf) {
    #[cfg(debug_assertions)]
    {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_dir = manifest_dir.parent().unwrap().parent().unwrap();
        (
            workspace_dir.join("librime").join("data").join("minimal"),
            workspace_dir.join("target").join("debug").join("user-data"),
        )
    }

    #[cfg(not(debug_assertions))]
    {
        let exe_path = std::env::current_exe().ok().unwrap_or_else(|| std::path::PathBuf::from("."));
        let exe_dir = exe_path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| std::path::PathBuf::from("."));
        
        let user_data_dir = std::env::var("APPDATA")
            .ok()
            .map(|p| std::path::PathBuf::from(p).join("Xime").join("rime"))
            .unwrap_or_else(|| exe_dir.join("user-data"));
        
        (
            exe_dir.join("data"),
            user_data_dir,
        )
    }
}

fn get_config_source_dir() -> std::path::PathBuf {
    #[cfg(debug_assertions)]
    {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_dir = manifest_dir.parent().unwrap().parent().unwrap();
        workspace_dir.join("resources")
    }

    #[cfg(not(debug_assertions))]
    {
        let exe_path = std::env::current_exe().ok().unwrap_or_else(|| std::path::PathBuf::from("."));
        exe_path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| std::path::PathBuf::from(".")).join("resources")
    }
}

pub struct RimeConfigManager {
    settings: CustomSettings,
}

impl RimeConfigManager {
    pub fn new() -> Result<Self, String> {
        init_rime_deployer()?;
        let settings = CustomSettings::new("xime", "Xime::ConfigManager")
            .map_err(|e| e.to_string())?;
        Ok(Self { settings })
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.settings.get_string(key)
    }

    pub fn get_int(&self, key: &str) -> Option<i32> {
        self.settings.get_int(key)
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.settings.get_bool(key)
    }

    pub fn get_double(&self, key: &str) -> Option<f64> {
        self.settings.get_double(key)
    }

    pub fn set_string(&self, key: &str, value: &str) -> Result<(), String> {
        self.settings.set_string(key, value).map_err(|e| e.to_string())
    }

    pub fn set_int(&self, key: &str, value: i32) -> Result<(), String> {
        self.settings.set_int(key, value).map_err(|e| e.to_string())
    }

    pub fn set_bool(&self, key: &str, value: bool) -> Result<(), String> {
        self.settings.set_bool(key, value).map_err(|e| e.to_string())
    }

    pub fn set_double(&self, key: &str, value: f64) -> Result<(), String> {
        self.settings.set_double(key, value).map_err(|e| e.to_string())
    }

    pub fn save(&self) -> Result<(), String> {
        self.settings.save().map_err(|e| e.to_string())
    }
}

pub struct SchemaManager {
    settings: SwitcherSettings,
}

impl SchemaManager {
    pub fn new() -> Result<Self, String> {
        init_rime_deployer()?;
        let settings = SwitcherSettings::new()
            .map_err(|e| e.to_string())?;
        Ok(Self { settings })
    }
    
    pub fn get_schema_list(&self) -> Vec<SchemaInfo> {
        self.settings.get_schema_list()
    }
    
    pub fn set_schema_list(&self, schema_ids: &[&str]) -> Result<(), String> {
        self.settings.set_schema_list(schema_ids).map_err(|e| e.to_string())
    }
    
    pub fn save(&self) -> Result<(), String> {
        self.settings.save().map_err(|e| e.to_string())
    }
    
    pub fn get_selected_schema(&self) -> Option<String> {
        self.settings.get_selected_schema()
    }
}

pub fn deploy_all_schemas() -> Result<(), String> {
    init_rime_deployer()?;
    deploy_all().map_err(|e| e.to_string())
}

pub struct SchemaConfigManager {
    settings: CustomSettings,
    schema_id: String,
}

#[derive(Clone, Debug, Default)]
pub struct SpellerConfig {
    pub max_code_length: Option<i32>,
    pub auto_select: Option<bool>,
    pub auto_clear: Option<String>,
    pub alphabet: Option<String>,
    pub delimiter: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct TranslatorConfig {
    pub enable_charset_filter: Option<bool>,
    pub enable_completion: Option<bool>,
    pub enable_sentence: Option<bool>,
    pub enable_user_dict: Option<bool>,
    pub enable_encoder: Option<bool>,
    pub encode_commit_history: Option<bool>,
    pub max_phrase_length: Option<i32>,
}

#[derive(Clone, Debug, Default)]
pub struct ReverseLookupConfig {
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub tips: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct TraditionConfig {
    pub opencc_config: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct SchemaConfig {
    pub speller: SpellerConfig,
    pub translator: TranslatorConfig,
    pub reverse_lookup: ReverseLookupConfig,
    pub tradition: TraditionConfig,
}

impl SchemaConfigManager {
    pub fn new(schema_id: &str) -> Result<Self, String> {
        init_rime_deployer()?;
        let settings = CustomSettings::new(schema_id, "Xime::SchemaConfigManager")
            .map_err(|e| e.to_string())?;
        Ok(Self { settings, schema_id: schema_id.to_string() })
    }
    
    fn get_patch_value_string(&self, key: &str) -> Option<String> {
        let (_, user_data_dir) = get_data_dirs();
        let custom_yaml_path = user_data_dir.join(format!("{}.custom.yaml", self.schema_id));
        
        if !custom_yaml_path.exists() {
            return None;
        }
        
        let content = std::fs::read_to_string(&custom_yaml_path).ok()?;
        
        #[derive(Deserialize)]
        struct CustomYaml {
            patch: Option<serde_yaml::Value>,
        }
        
        let yaml: CustomYaml = serde_yaml::from_str(&content).ok()?;
        let patch = yaml.patch?;
        
        if let Some(value) = patch.get(key) {
            if let serde_yaml::Value::String(s) = value {
                return Some(s.clone());
            } else if let serde_yaml::Value::Bool(b) = value {
                return Some(b.to_string());
            } else if let serde_yaml::Value::Number(n) = value {
                return Some(n.to_string());
            }
        }
        
        let key_parts: Vec<&str> = key.split('/').collect();
        if key_parts.len() > 1 {
            let mut current = patch;
            for part in &key_parts {
                if let Some(next) = current.get(part) {
                    current = next.clone();
                } else {
                    return None;
                }
            }
            if let serde_yaml::Value::String(s) = current {
                return Some(s);
            } else if let serde_yaml::Value::Bool(b) = current {
                return Some(b.to_string());
            } else if let serde_yaml::Value::Number(n) = current {
                return Some(n.to_string());
            }
        }
        
        None
    }
    
    fn get_patch_value_int(&self, key: &str) -> Option<i32> {
        let (_, user_data_dir) = get_data_dirs();
        let custom_yaml_path = user_data_dir.join(format!("{}.custom.yaml", self.schema_id));
        
        if !custom_yaml_path.exists() {
            return None;
        }
        
        let content = std::fs::read_to_string(&custom_yaml_path).ok()?;
        
        #[derive(Deserialize)]
        struct CustomYaml {
            patch: Option<serde_yaml::Value>,
        }
        
        let yaml: CustomYaml = serde_yaml::from_str(&content).ok()?;
        let patch = yaml.patch?;
        
        if let Some(value) = patch.get(key) {
            if let serde_yaml::Value::Number(n) = value {
                return n.as_i64().map(|v| v as i32);
            }
        }
        
        let key_parts: Vec<&str> = key.split('/').collect();
        if key_parts.len() > 1 {
            let mut current = patch;
            for part in &key_parts {
                if let Some(next) = current.get(part) {
                    current = next.clone();
                } else {
                    return None;
                }
            }
            if let serde_yaml::Value::Number(n) = current {
                return n.as_i64().map(|v| v as i32);
            }
        }
        
        None
    }
    
    fn get_patch_value_bool(&self, key: &str) -> Option<bool> {
        let (_, user_data_dir) = get_data_dirs();
        let custom_yaml_path = user_data_dir.join(format!("{}.custom.yaml", self.schema_id));
        
        if !custom_yaml_path.exists() {
            return None;
        }
        
        let content = std::fs::read_to_string(&custom_yaml_path).ok()?;
        
        #[derive(Deserialize)]
        struct CustomYaml {
            patch: Option<serde_yaml::Value>,
        }
        
        let yaml: CustomYaml = serde_yaml::from_str(&content).ok()?;
        let patch = yaml.patch?;
        
        if let Some(value) = patch.get(key) {
            if let serde_yaml::Value::Bool(b) = value {
                return Some(*b);
            }
        }
        
        let key_parts: Vec<&str> = key.split('/').collect();
        if key_parts.len() > 1 {
            let mut current = patch;
            for part in &key_parts {
                if let Some(next) = current.get(part) {
                    current = next.clone();
                } else {
                    return None;
                }
            }
            if let serde_yaml::Value::Bool(b) = current {
                return Some(b);
            }
        }
        
        None
    }
    
    fn get_merged_string(&self, key: &str) -> Option<String> {
        self.get_patch_value_string(key).or_else(|| self.settings.get_string(key))
    }
    
    fn get_merged_int(&self, key: &str) -> Option<i32> {
        self.get_patch_value_int(key).or_else(|| self.settings.get_int(key))
    }
    
    fn get_merged_bool(&self, key: &str) -> Option<bool> {
        self.get_patch_value_bool(key).or_else(|| self.settings.get_bool(key))
    }
    
    pub fn get_config(&self) -> SchemaConfig {
        SchemaConfig {
            speller: SpellerConfig {
                max_code_length: self.get_merged_int("speller/max_code_length"),
                auto_select: self.get_merged_bool("speller/auto_select"),
                auto_clear: self.get_merged_string("speller/auto_clear"),
                alphabet: self.get_merged_string("speller/alphabet"),
                delimiter: self.get_merged_string("speller/delimiter"),
            },
            translator: TranslatorConfig {
                enable_charset_filter: self.get_merged_bool("translator/enable_charset_filter"),
                enable_completion: self.get_merged_bool("translator/enable_completion"),
                enable_sentence: self.get_merged_bool("translator/enable_sentence"),
                enable_user_dict: self.get_merged_bool("translator/enable_user_dict"),
                enable_encoder: self.get_merged_bool("translator/enable_encoder"),
                encode_commit_history: self.get_merged_bool("translator/encode_commit_history"),
                max_phrase_length: self.get_merged_int("translator/max_phrase_length"),
            },
            reverse_lookup: ReverseLookupConfig {
                prefix: self.get_merged_string("reverse_lookup/prefix"),
                suffix: self.get_merged_string("reverse_lookup/suffix"),
                tips: self.get_merged_string("reverse_lookup/tips"),
            },
            tradition: TraditionConfig {
                opencc_config: self.get_merged_string("tradition/opencc_config"),
            },
        }
    }
    
    pub fn set_string(&self, key: &str, value: &str) -> Result<(), String> {
        self.settings.set_string(key, value).map_err(|e| e.to_string())
    }
    
    pub fn set_int(&self, key: &str, value: i32) -> Result<(), String> {
        self.settings.set_int(key, value).map_err(|e| e.to_string())
    }
    
    pub fn set_bool(&self, key: &str, value: bool) -> Result<(), String> {
        self.settings.set_bool(key, value).map_err(|e| e.to_string())
    }
    
    pub fn save(&self) -> Result<(), String> {
        self.settings.save().map_err(|e| e.to_string())
    }
    
    pub fn schema_id(&self) -> &str {
        &self.schema_id
    }
}