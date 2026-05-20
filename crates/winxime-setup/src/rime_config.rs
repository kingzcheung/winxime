pub use librime::levers::deploy_all;
pub use librime::levers::SchemaInfo;
use librime::{
    create_session, get_api, initialize, join_maintenance_thread, levers::CustomSettings, setup,
    start_maintenance, Traits,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::CString;
use std::io::Write;
use std::sync::Once;

static RIME_INIT: Once = Once::new();

pub fn init_rime_deployer() -> Result<(), String> {
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
                schemas
                    .iter()
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
"#,
            schema_list
        );

        std::fs::write(&default_custom, content).ok();
    }
}

fn download_and_extract_user_configs(user_data_dir: &std::path::Path) {
    use flate2::read::GzDecoder;
    use std::fs::File;
    use std::io::Read;
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
        let exe_path = std::env::current_exe()
            .ok()
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        let exe_dir = exe_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        let user_data_dir = std::env::var("APPDATA")
            .ok()
            .map(|p| std::path::PathBuf::from(p).join("Xime").join("rime"))
            .unwrap_or_else(|| exe_dir.join("user-data"));

        (exe_dir.join("data"), user_data_dir)
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
        let exe_path = std::env::current_exe()
            .ok()
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        exe_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("resources")
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
                            schemas.push(SchemaInfo {
                                schema_id: schema_id.clone(),
                                name: schema_name,
                            });
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
                            schemas.push(SchemaInfo {
                                schema_id,
                                name: schema_name,
                            });
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
"#,
            schema_list_yaml
        );

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
            let schema = line
                .split("schema:")
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
                return trimmed
                    .split(':')
                    .nth(1)
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
        Ok(Self {
            settings,
            schema_id: schema_id.to_string(),
        })
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
        let key_parts: Vec<&str> = key.split('/').collect();

        let mut yaml: serde_json::Map<String, serde_json::Value> = if xime_custom.exists() {
            let content = std::fs::read_to_string(&xime_custom)
                .map_err(|e| format!("Failed to read xime.custom.yaml: {}", e))?;
            let doc: serde_json::Value = serde_saphyr::from_str(&content)
                .map_err(|e| format!("Failed to parse xime.custom.yaml: {}", e))?;
            doc.as_object().map(|m| m.clone()).unwrap_or_default()
        } else {
            let mut m = serde_json::Map::new();
            m.insert(
                "customization".to_string(),
                serde_json::Value::Object(serde_json::Map::new()),
            );
            let mut customization = m
                .get("customization")
                .and_then(|v| v.as_object())
                .cloned()
                .unwrap_or_default();
            customization.insert(
                "distribution_code_name".to_string(),
                serde_json::Value::String("Xime".to_string()),
            );
            customization.insert(
                "distribution_version".to_string(),
                serde_json::Value::String("1.0".to_string()),
            );
            m.insert(
                "customization".to_string(),
                serde_json::Value::Object(customization),
            );
            m
        };

        let patch_key = "patch".to_string();
        let patch_mapping = yaml
            .get(&patch_key)
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_else(|| serde_json::Map::new());

        let mut final_patch = patch_mapping.clone();
        set_nested_value(
            &mut final_patch,
            &key_parts,
            serde_json::Value::String(value),
        );

        yaml.insert(patch_key, serde_json::Value::Object(final_patch));

        let doc = serde_json::Value::Object(yaml);
        let content =
            serde_saphyr::to_string(&doc).map_err(|e| format!("Failed to serialize: {}", e))?;

        std::fs::write(&xime_custom, content).map_err(|e| format!("Failed to write: {}", e))?;

        Ok(())
    }

    pub fn save(&self) -> Result<(), String> {
        Ok(())
    }
}

fn set_nested_value(
    mapping: &mut serde_json::Map<String, serde_json::Value>,
    keys: &[&str],
    value: serde_json::Value,
) {
    if keys.is_empty() {
        return;
    }

    let key = keys[0].to_string();

    if keys.len() == 1 {
        mapping.insert(key, value);
    } else {
        let mut nested = mapping
            .get(&key)
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_else(|| serde_json::Map::new());

        set_nested_value(&mut nested, &keys[1..], value);
        mapping.insert(key, serde_json::Value::Object(nested));
    }
}

fn get_yaml_value(content: &str, key: &str) -> Option<String> {
    let key_parts: Vec<&str> = key.split('/').collect();

    let yaml: serde_json::Value = serde_saphyr::from_str(content).ok()?;
    let mut current = &yaml;

    for part in &key_parts {
        current = current.get(part)?;
    }

    match current {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct XimeStyleConfig {
    #[serde(default)]
    pub color_scheme: String,
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    #[serde(default = "default_candidate_count")]
    pub candidate_count: i32,
    #[serde(default)]
    pub show_code_hint: bool,
    #[serde(default = "default_horizontal")]
    pub horizontal: bool,
}

fn default_font_size() -> f32 {
    14.0
}
fn default_candidate_count() -> i32 {
    5
}
fn default_horizontal() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ColorSchemeEntry {
    pub name: String,
    #[serde(
        deserialize_with = "deserialize_hex_color",
        serialize_with = "serialize_hex_color"
    )]
    pub primary_color: u32,
}

fn deserialize_hex_color<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};

    struct ColorVisitor;

    impl<'de> Visitor<'de> for ColorVisitor {
        type Value = u32;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a color value (hex string or number)")
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v as u32)
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let s = s.trim();
            if s.starts_with("0x") || s.starts_with("0X") {
                u32::from_str_radix(&s[2..], 16).map_err(de::Error::custom)
            } else if s.starts_with('#') {
                u32::from_str_radix(&s[1..], 16).map_err(de::Error::custom)
            } else {
                s.parse::<u32>().map_err(de::Error::custom)
            }
        }
    }

    deserializer.deserialize_any(ColorVisitor)
}

fn serialize_hex_color<S>(value: &u32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&format!("0x{:06X}", value))
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct XimeConfigFile {
    #[serde(default)]
    pub style: XimeStyleConfig,
    #[serde(default)]
    pub color_schemes: HashMap<String, ColorSchemeEntry>,
}

pub struct XimeStyleManager {
    base_config_path: std::path::PathBuf,
    custom_config_path: std::path::PathBuf,
    config: XimeConfigFile,
}

impl XimeStyleManager {
    pub fn load() -> Result<Self, String> {
        let (_, user_dir) = get_data_dirs();
        let base_config_path = user_dir.join("xime.yaml");
        let custom_config_path = user_dir.join("xime.custom.yaml");

        let mut config = XimeConfigFile::default();

        if base_config_path.exists() {
            let content = std::fs::read_to_string(&base_config_path)
                .map_err(|e| format!("Failed to read xime.yaml: {}", e))?;
            config = serde_saphyr::from_str(&content)
                .map_err(|e| format!("Failed to parse xime.yaml: {}", e))?;
        }

        if custom_config_path.exists() {
            let content = std::fs::read_to_string(&custom_config_path)
                .map_err(|e| format!("Failed to read xime.custom.yaml: {}", e))?;
            let custom_config: XimeConfigFile = serde_saphyr::from_str(&content)
                .map_err(|e| format!("Failed to parse xime.custom.yaml: {}", e))?;

            if !custom_config.style.color_scheme.is_empty() {
                config.style.color_scheme = custom_config.style.color_scheme;
            }
            if custom_config.style.font_size != 0.0 {
                config.style.font_size = custom_config.style.font_size;
            }
            if custom_config.style.candidate_count != 0 {
                config.style.candidate_count = custom_config.style.candidate_count;
            }
            config.style.show_code_hint = custom_config.style.show_code_hint;
            config.style.horizontal = custom_config.style.horizontal;
        }

        let system_config = Self::load_system_color_schemes().unwrap_or_default();
        config.color_schemes.extend(system_config.color_schemes);

        Ok(Self {
            base_config_path,
            custom_config_path,
            config,
        })
    }

    fn load_system_color_schemes() -> Option<XimeConfigFile> {
        let config_source_dir = get_config_source_dir();
        let system_path = config_source_dir.join("xime.yaml");

        if !system_path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&system_path).ok()?;
        let config: XimeConfigFile = serde_saphyr::from_str(&content).ok()?;

        Some(XimeConfigFile {
            color_schemes: config.color_schemes,
            ..Default::default()
        })
    }

    pub fn get_style(&self) -> XimeStyleConfig {
        self.config.style.clone()
    }

    pub fn get_color_schemes(&self) -> Vec<(String, String, u32)> {
        self.config
            .color_schemes
            .iter()
            .map(|(id, entry)| (id.clone(), entry.name.clone(), entry.primary_color))
            .collect()
    }

    pub fn set_color_scheme(&mut self, scheme_id: &str) -> Result<(), String> {
        self.config.style.color_scheme = scheme_id.to_string();
        self.save()
    }

    pub fn set_font_size(&mut self, size: f32) -> Result<(), String> {
        self.config.style.font_size = size;
        self.save()
    }

    pub fn set_candidate_count(&mut self, count: i32) -> Result<(), String> {
        self.config.style.candidate_count = count;
        self.save()
    }

    pub fn set_show_code_hint(&mut self, show: bool) -> Result<(), String> {
        self.config.style.show_code_hint = show;
        self.save()
    }

    pub fn set_horizontal(&mut self, horizontal: bool) -> Result<(), String> {
        self.config.style.horizontal = horizontal;
        self.save()
    }

    fn save(&self) -> Result<(), String> {
        let custom_config = XimeConfigFile {
            style: self.config.style.clone(),
            color_schemes: HashMap::new(),
        };

        let content = serde_saphyr::to_string(&custom_config)
            .map_err(|e| format!("Failed to serialize xime.custom.yaml: {}", e))?;

        std::fs::write(&self.custom_config_path, content)
            .map_err(|e| format!("Failed to write xime.custom.yaml: {}", e))?;

        Ok(())
    }
}
