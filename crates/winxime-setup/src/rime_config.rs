use librime::{
    get_api, setup, initialize, Traits,
    create_session, start_maintenance, join_maintenance_thread,
    levers::CustomSettings,
};
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
        ensure_schemas_in_user_dir(&shared_data_dir, &user_data_dir);
        
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
    
    let config_source_dir = get_config_source_dir();
    
    let xime_yaml = user_data_dir.join("xime.yaml");
    if !xime_yaml.exists() {
        let source = config_source_dir.join("xime.yaml");
        if source.exists() {
            std::fs::copy(&source, &xime_yaml).ok();
        }
    }
}

fn ensure_schemas_in_user_dir(shared_data_dir: &std::path::Path, user_data_dir: &std::path::Path) {
    let default_custom = user_data_dir.join("default.custom.yaml");
    if !default_custom.exists() {
        let schema_list = if let Ok(entries) = std::fs::read_dir(shared_data_dir) {
            let schemas: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.ends_with(".schema.yaml") {
                        Some(name.replace(".schema.yaml", ""))
                    } else {
                        None
                    }
                })
                .collect();
            
            if schemas.is_empty() {
                "    - schema: wubi86\n".to_string()
            } else {
                schemas.iter()
                    .map(|s| format!("    - schema: {}\n", s))
                    .collect::<Vec<_>>()
                    .join("")
            }
        } else {
            "    - schema: wubi86\n".to_string()
        };
        
        let content = format!(
r#"customization:
  distribution_code_name: Xime
  distribution_version: 1.0

patch:
  schema_list:
{}
"#, schema_list);
        
        std::fs::write(&default_custom, content).ok();
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

pub struct SchemaManager {
    user_dir: std::path::PathBuf,
}

impl SchemaManager {
    pub fn new() -> Result<Self, String> {
        init_rime_deployer()?;
        let (_, user_dir) = get_data_dirs();
        Ok(Self { user_dir })
    }
    
    pub fn get_schema_list(&self) -> Vec<SchemaInfo> {
        let (shared_data_dir, user_data_dir) = get_data_dirs();
        let mut schemas = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();
        
        if let Ok(entries) = std::fs::read_dir(&user_data_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".schema.yaml") {
                        let schema_id = name.replace(".schema.yaml", "");
                        
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            let schema_name = extract_schema_name(&content, &schema_id);
                            schemas.push(SchemaInfo { schema_id: schema_id.clone(), name: schema_name });
                            seen_ids.insert(schema_id);
                        }
                    }
                }
            }
        }
        
        if let Ok(entries) = std::fs::read_dir(&shared_data_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".schema.yaml") {
                        let schema_id = name.replace(".schema.yaml", "");
                        
                        if seen_ids.contains(&schema_id) {
                            continue;
                        }
                        
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            let schema_name = extract_schema_name(&content, &schema_id);
                            schemas.push(SchemaInfo { schema_id, name: schema_name });
                        }
                    }
                }
            }
        }
        
        schemas.sort_by(|a, b| a.name.cmp(&b.name));
        schemas
    }
    
    pub fn set_schema_list(&self, schema_ids: &[&str]) -> Result<(), String> {
        let default_custom = self.user_dir.join("default.custom.yaml");
        
        let schema_list_yaml = schema_ids
            .iter()
            .map(|id| format!("    - schema: {}", id))
            .collect::<Vec<_>>()
            .join("\n");
        
        let content = format!(
r#"customization:
  distribution_code_name: Xime
  distribution_version: 1.0

patch:
  schema_list:
{}
"#, schema_list_yaml);
        
        std::fs::write(&default_custom, content)
            .map_err(|e| format!("Failed to write default.custom.yaml: {}", e))?;
        
        Ok(())
    }
    
    pub fn save(&self) -> Result<(), String> {
        Ok(())
    }
    
    pub fn get_selected_schema(&self) -> Option<String> {
        let default_custom = self.user_dir.join("default.custom.yaml");
        if !default_custom.exists() {
            return None;
        }
        
        let content = std::fs::read_to_string(&default_custom).ok()?;
        extract_selected_schema(&content)
    }
}

fn extract_selected_schema(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.contains("schema:") {
            let schema = line.split("schema:")
                .nth(1)
                .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string());
            if let Some(s) = schema {
                if !s.is_empty() {
                    return Some(s);
                }
            }
        }
    }
    None
}

fn extract_schema_name(content: &str, schema_id: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_schema_block = false;
    let mut indent_level = 0;
    
    for line in &lines {
        let trimmed = line.trim();
        
        if trimmed == "schema:" {
            in_schema_block = true;
            indent_level = line.len() - line.trim_start().len();
            continue;
        }
        
        if in_schema_block {
            let current_indent = line.len() - line.trim_start().len();
            
            if current_indent <= indent_level && !trimmed.is_empty() && !trimmed.starts_with('#') {
                break;
            }
            
            if trimmed.starts_with("name:") {
                return trimmed.split(':').nth(1)
                    .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                    .unwrap_or_else(|| schema_id.to_string());
            }
        }
    }
    
    schema_id.to_string()
}

pub fn deploy_all_schemas() -> Result<(), String> {
    init_rime_deployer()?;
    deploy_all().map_err(|e| e.to_string())
}

pub struct SchemaConfigManager {
    settings: CustomSettings,
    schema_id: String,
}

impl SchemaConfigManager {
    pub fn new(schema_id: &str) -> Result<Self, String> {
        init_rime_deployer()?;
        let settings = CustomSettings::new(schema_id, "Xime::SchemaConfigManager")
            .map_err(|e| e.to_string())?;
        Ok(Self { settings, schema_id: schema_id.to_string() })
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
        self.settings.set_bool(key, value).map_err(|e| e.to_string())
    }
    
    pub fn set_string(&self, key: &str, value: &str) -> Result<(), String> {
        self.settings.set_string(key, value).map_err(|e| e.to_string())
    }
    
    pub fn save(&self) -> Result<(), String> {
        self.settings.save().map_err(|e| e.to_string())
    }
    
    pub fn schema_id(&self) -> &str {
        &self.schema_id
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

pub struct RimeConfigManager {
    user_dir: std::path::PathBuf,
}

impl RimeConfigManager {
    pub fn new() -> Result<Self, String> {
        let (_, user_dir) = get_data_dirs();
        Ok(Self { user_dir })
    }
    
    pub fn get_double(&self, key: &str) -> Option<f64> {
        self.get_value(key).and_then(|v| v.parse::<f64>().ok())
    }
    
    pub fn get_int(&self, key: &str) -> Option<i32> {
        self.get_value(key).and_then(|v| v.parse::<i32>().ok())
    }
    
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get_value(key).and_then(|v| v.parse::<bool>().ok())
    }
    
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get_value(key)
    }
    
    fn get_value(&self, key: &str) -> Option<String> {
        let xime_yaml = self.user_dir.join("xime.yaml");
        let xime_custom = self.user_dir.join("xime.custom.yaml");
        
        if xime_custom.exists() {
            let content = std::fs::read_to_string(&xime_custom).ok()?;
            if let Some(v) = get_yaml_value(&content, key) {
                return Some(v);
            }
        }
        
        if xime_yaml.exists() {
            let content = std::fs::read_to_string(&xime_yaml).ok()?;
            get_yaml_value(&content, key)
        } else {
            None
        }
    }
    
    pub fn set_double(&self, key: &str, value: f64) -> Result<(), String> {
        self.set_value(key, value.to_string())
    }
    
    pub fn set_int(&self, key: &str, value: i32) -> Result<(), String> {
        self.set_value(key, value.to_string())
    }
    
    pub fn set_bool(&self, key: &str, value: bool) -> Result<(), String> {
        self.set_value(key, value.to_string())
    }
    
    pub fn set_string(&self, key: &str, value: &str) -> Result<(), String> {
        self.set_value(key, value.to_string())
    }
    
    fn set_value(&self, key: &str, value: String) -> Result<(), String> {
        let xime_custom = self.user_dir.join("xime.custom.yaml");
        
        let existing_content = if xime_custom.exists() {
            std::fs::read_to_string(&xime_custom).ok()
        } else {
            None
        };
        
        let mut lines: Vec<String> = existing_content
            .map(|c| c.lines().map(|l| l.to_string()).collect())
            .unwrap_or_else(|| {
                vec![
                    "customization:".to_string(),
                    "  distribution_code_name: Xime".to_string(),
                    "  distribution_version: 1.0".to_string(),
                    "".to_string(),
                    "patch:".to_string(),
                ]
            });
        
        let key_parts: Vec<&str> = key.split('/').collect();
        let formatted_key = if key_parts.len() > 1 {
            format!("{}{}", "  ".repeat(key_parts.len()), key_parts.join("_"))
        } else {
            format!("  {}", key)
        };
        
        let new_line = format!("{}: {}", formatted_key, value);
        
        let patch_idx = lines.iter().position(|l| l.trim() == "patch:");
        if let Some(idx) = patch_idx {
            let key_prefix = format!("{}:", formatted_key);
            let existing_idx = lines.iter().skip(idx + 1)
                .position(|l| l.starts_with(&key_prefix));
            
            if let Some(e_idx) = existing_idx {
                lines[idx + 1 + e_idx] = new_line;
            } else {
                lines.insert(idx + 1, new_line);
            }
        }
        
        std::fs::write(&xime_custom, lines.join("\n") + "\n")
            .map_err(|e| format!("Failed to write: {}", e))?;
        
        Ok(())
    }
    
    pub fn save(&self) -> Result<(), String> {
        Ok(())
    }
}

fn get_yaml_value(content: &str, key: &str) -> Option<String> {
    let key_parts: Vec<&str> = key.split('/').collect();
    
    let yaml: serde_yaml::Value = serde_yaml::from_str(content).ok()?;
    let mut current = &yaml;
    
    for part in &key_parts {
        current = current.get(part)?;
    }
    
    match current {
        serde_yaml::Value::String(s) => Some(s.clone()),
        serde_yaml::Value::Number(n) => Some(n.to_string()),
        serde_yaml::Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}