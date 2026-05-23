mod rime_deploy;
mod schema_config;
mod schema_manager;
mod smart_suggestion;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub use rime_deploy::{
    deploy_all, deploy_all_schemas, get_data_dirs, init_rime_deployer, SchemaInfo,
};
pub use schema_config::{
    ReverseLookupConfig, SchemaConfig, SchemaConfigManager, SpellerConfig, TraditionConfig,
    TranslatorConfig,
};
pub use schema_manager::SchemaManager;
pub use smart_suggestion::{
    SmartSuggestionConfig, SmartSuggestionModelConfig, SmartSuggestionModelFile,
};

static LOG_GUARD: Mutex<Option<tracing_appender::non_blocking::WorkerGuard>> = Mutex::new(None);

struct LocalTimer;

impl tracing_subscriber::fmt::time::FormatTime for LocalTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = chrono::Local::now();
        write!(w, "{}", now.format("%Y-%m-%dT%H:%M:%S%.6f"))
    }
}

pub fn init_logging(component: &str) {
    let log_dir = get_log_dir();
    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = tracing_appender::rolling::RollingFileAppender::new(
        tracing_appender::rolling::Rotation::NEVER,
        log_dir,
        format!("{}.log", component),
    );

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    if let Ok(mut g) = LOG_GUARD.lock() {
        *g = Some(guard);
    }

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));

    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_target(false)
                .with_line_number(true)
                .with_timer(LocalTimer),
        )
        .try_init()
        .ok();
}

pub fn init_logging_with_console(component: &str) {
    let log_dir = get_log_dir();
    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = tracing_appender::rolling::RollingFileAppender::new(
        tracing_appender::rolling::Rotation::NEVER,
        log_dir,
        format!("{}.log", component),
    );

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    if let Ok(mut g) = LOG_GUARD.lock() {
        *g = Some(guard);
    }

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));

    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_target(false)
                .with_line_number(true)
                .with_timer(LocalTimer),
        )
        .with(fmt::layer().with_writer(std::io::stdout).with_ansi(true))
        .try_init()
        .ok();
}

fn get_log_dir() -> PathBuf {
    std::env::var("TEMP")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("winxime")
}

pub fn log_dir() -> PathBuf {
    get_log_dir()
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct XimeConfig {
    #[serde(default)]
    pub wubi_radicals: WubiRadicalsConfig,
    #[serde(default)]
    pub style: StyleConfig,
    #[serde(default)]
    pub color_schemes: HashMap<String, ColorScheme>,
    #[serde(default)]
    pub smart_suggestion: SmartSuggestionConfig,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct WubiRadicalsConfig {
    #[serde(default)]
    pub hotkeys: HotkeyConfig,
    #[serde(default)]
    pub schema_radicals: SchemaRadicalsConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct StyleConfig {
    #[serde(default)]
    pub font_family: String,
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    #[serde(default = "default_candidate_count")]
    pub candidate_count: i32,
    #[serde(default = "default_horizontal")]
    pub horizontal: bool,
    #[serde(default = "default_corner_radius")]
    pub corner_radius: f32,
    #[serde(default = "default_color_scheme")]
    pub color_scheme: String,
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            font_family: String::new(),
            font_size: default_font_size(),
            candidate_count: default_candidate_count(),
            horizontal: default_horizontal(),
            corner_radius: default_corner_radius(),
            color_scheme: default_color_scheme(),
        }
    }
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
fn default_corner_radius() -> f32 {
    8.0
}
fn default_color_scheme() -> String {
    "lavender_purple".to_string()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct ColorScheme {
    #[serde(default)]
    pub name: String,
    #[serde(
        deserialize_with = "deserialize_hex_color",
        serialize_with = "serialize_hex_color",
        default = "default_primary_color"
    )]
    pub primary_color: u32,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            name: String::new(),
            primary_color: default_primary_color(),
        }
    }
}

fn default_primary_color() -> u32 {
    0x8F73E2
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
    serializer.serialize_u32(*value)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HotkeyConfig {
    #[serde(default = "default_show_key")]
    pub show_key: String,
    #[serde(default)]
    pub show_all_keys: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            show_key: default_show_key(),
            show_all_keys: String::new(),
        }
    }
}

fn default_show_key() -> String {
    "Ctrl".to_string()
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct SchemaRadicalsConfig {
    #[serde(default, flatten)]
    pub schemas: HashMap<String, WubiRootConfig>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct WubiRootConfig {
    #[serde(default)]
    pub g: String,
    #[serde(default)]
    pub f: String,
    #[serde(default)]
    pub d: String,
    #[serde(default)]
    pub s: String,
    #[serde(default)]
    pub a: String,
    #[serde(default)]
    pub h: String,
    #[serde(default)]
    pub j: String,
    #[serde(default)]
    pub k: String,
    #[serde(default)]
    pub l: String,
    #[serde(default)]
    pub m: String,
    #[serde(default)]
    pub t: String,
    #[serde(default)]
    pub r: String,
    #[serde(default)]
    pub e: String,
    #[serde(default)]
    pub w: String,
    #[serde(default)]
    pub q: String,
    #[serde(default)]
    pub y: String,
    #[serde(default)]
    pub u: String,
    #[serde(default)]
    pub i: String,
    #[serde(default)]
    pub o: String,
    #[serde(default)]
    pub p: String,
    #[serde(default)]
    pub n: String,
    #[serde(default)]
    pub b: String,
    #[serde(default)]
    pub v: String,
    #[serde(default)]
    pub c: String,
    #[serde(default)]
    pub x: String,
}

impl XimeConfig {
    /// Parse XimeConfig from YAML via serde_json::Value intermediate
    /// to completely avoid serde_saphyr's broken nested map scoping.
    fn parse_yaml(content: &str) -> Option<Self> {
        let root: serde_json::Value = serde_saphyr::from_str(content).ok()?;
        let obj = root.as_object()?;

        let mut config = XimeConfig::default();

        for (k, v) in obj {
            match k.as_str() {
                "wubi_radicals" => {
                    config.wubi_radicals = Self::parse_wubi_radicals(v)?;
                }
                "style" => {
                    config.style = serde_json::from_value(v.clone()).ok()?;
                }
                "color_schemes" => {
                    config.color_schemes = serde_json::from_value(v.clone()).ok()?;
                }
                "smart_suggestion" => {
                    config.smart_suggestion = serde_json::from_value(v.clone()).ok()?;
                }
                _ => {}
            }
        }

        Some(config)
    }

    fn parse_wubi_radicals(value: &serde_json::Value) -> Option<WubiRadicalsConfig> {
        let obj = value.as_object()?;

        let mut hotkeys = HotkeyConfig::default();
        let mut schemas: HashMap<String, WubiRootConfig> = HashMap::new();

        for (k, v) in obj {
            match k.as_str() {
                "hotkeys" => {
                    hotkeys = serde_json::from_value(v.clone()).ok()?;
                }
                "schema_radicals" => {
                    let sr_obj = v.as_object()?;
                    for (sk, sv) in sr_obj {
                        let root: WubiRootConfig = serde_json::from_value(sv.clone()).ok()?;
                        schemas.insert(sk.clone(), root);
                    }
                }
                _ => {}
            }
        }

        Some(WubiRadicalsConfig {
            hotkeys,
            schema_radicals: SchemaRadicalsConfig { schemas },
        })
    }

    pub fn load() -> Self {
        // 1. Lowest: compiled-in defaults
        let mut config = Self::builtin_default();

        // 2. System config from exe dir
        let exe_path = std::env::current_exe().unwrap_or_default();
        let system_path = exe_path
            .parent()
            .map(|p| p.join("data").join("xime.yaml"))
            .unwrap_or_default();
        if system_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&system_path) {
                if let Some(system) = Self::parse_yaml(&content) {
                    config = Self::merge_configs(config, system);
                }
            }
        }

        // 3. User xime.yaml (overrides system)
        let user_data_dir = Self::rime_user_data_dir();
        let user_yaml = user_data_dir.join("xime.yaml");
        if user_yaml.exists() {
            if let Ok(content) = std::fs::read_to_string(&user_yaml) {
                if let Some(user) = Self::parse_yaml(&content) {
                    config = Self::merge_configs(config, user);
                }
            }
        }

        // 4. User xime.custom.yaml (Rime patch format)
        config = Self::apply_custom_config(config, &user_data_dir);

        #[cfg(debug_assertions)]
        {
            let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let workspace_dir = manifest_dir.parent().unwrap().parent().unwrap();
            let dev_custom = workspace_dir.join("rime-wubi").join("xime.custom.yaml");
            if dev_custom.exists() {
                if let Ok(content) = std::fs::read_to_string(&dev_custom) {
                    if let Ok(value) = serde_saphyr::from_str::<serde_json::Value>(&content) {
                        if let Some(patch_value) = value.get("patch") {
                            if let Ok(custom) =
                                serde_json::from_value::<XimeConfig>(patch_value.clone())
                            {
                                config = Self::merge_configs(config, custom);
                            }
                        }
                    }
                }
            }
        }

        config
    }

    fn builtin_default() -> Self {
        const DEFAULT_CONFIG: &[u8] = include_bytes!("../../../resources/xime.yaml");
        let content = std::str::from_utf8(DEFAULT_CONFIG).unwrap_or_default();
        Self::parse_yaml(content).unwrap_or_default()
    }

    fn apply_custom_config(config: Self, user_data_dir: &std::path::Path) -> Self {
        let custom_path = user_data_dir.join("xime.custom.yaml");
        if !custom_path.exists() {
            return config;
        }
        let content = match std::fs::read_to_string(&custom_path) {
            Ok(c) => c,
            Err(_) => return config,
        };
        if let Some(custom) = Self::read_config(&content) {
            return Self::merge_configs(config, custom);
        }
        config
    }

    fn merge_configs(base: Self, over: Self) -> Self {
        XimeConfig {
            wubi_radicals: WubiRadicalsConfig {
                hotkeys: over.wubi_radicals.hotkeys,
                schema_radicals: SchemaRadicalsConfig {
                    schemas: if over.wubi_radicals.schema_radicals.schemas.is_empty() {
                        base.wubi_radicals.schema_radicals.schemas
                    } else {
                        over.wubi_radicals.schema_radicals.schemas
                    },
                },
            },
            style: over.style,
            color_schemes: if over.color_schemes.is_empty() {
                base.color_schemes
            } else {
                over.color_schemes
            },
            smart_suggestion: SmartSuggestionConfig {
                enabled: over
                    .smart_suggestion
                    .enabled
                    .or(base.smart_suggestion.enabled),
                suggestion_count: over.smart_suggestion.suggestion_count,
                record_user_frequency: over.smart_suggestion.record_user_frequency,
                auto_adjust_frequency: over.smart_suggestion.auto_adjust_frequency,
                learning_threshold: over.smart_suggestion.learning_threshold,
                model: SmartSuggestionModelConfig {
                    provider: if over.smart_suggestion.model.provider.is_empty() {
                        base.smart_suggestion.model.provider
                    } else {
                        over.smart_suggestion.model.provider
                    },
                    name: if over.smart_suggestion.model.name.is_empty() {
                        base.smart_suggestion.model.name
                    } else {
                        over.smart_suggestion.model.name
                    },
                    auto_download: over.smart_suggestion.model.auto_download,
                    files: if over.smart_suggestion.model.files.is_empty() {
                        base.smart_suggestion.model.files
                    } else {
                        over.smart_suggestion.model.files
                    },
                },
            },
        }
    }

    fn rime_user_data_dir() -> PathBuf {
        let (_, user_data_dir) = crate::get_data_dirs();
        user_data_dir
    }

    pub fn config_path() -> PathBuf {
        #[cfg(debug_assertions)]
        {
            let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let workspace_dir = manifest_dir.parent().unwrap().parent().unwrap();
            workspace_dir.join("rime-wubi").join("xime.custom.yaml")
        }
        #[cfg(not(debug_assertions))]
        {
            Self::rime_user_data_dir().join("xime.custom.yaml")
        }
    }

    fn serialize_config(config: &XimeConfig) -> Result<String, String> {
        let value = serde_json::to_value(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        let root = serde_json::json!({"patch": value});
        serde_saphyr::to_string(&root)
            .map_err(|e| format!("Failed to serialize custom yaml: {}", e))
    }

    fn read_config(content: &str) -> Option<XimeConfig> {
        if let Ok(value) = serde_saphyr::from_str::<serde_json::Value>(content) {
            if let Some(patch_value) = value.get("patch") {
                if let Ok(config) = serde_json::from_value::<XimeConfig>(patch_value.clone()) {
                    return Some(config);
                }
            }
        }
        Self::parse_yaml(content)
    }

    pub fn save(&self) -> Result<(), String> {
        let config_path = Self::config_path();

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir: {}", e))?;
        }

        let content = Self::serialize_config(self)?;

        fs::write(&config_path, content).map_err(|e| format!("Failed to write config: {}", e))?;

        Ok(())
    }

    pub fn save_smart_suggestion(&self) -> Result<(), String> {
        let config_path = Self::config_path();

        let existing: XimeConfig = if config_path.exists() {
            fs::read_to_string(&config_path)
                .ok()
                .and_then(|c| Self::read_config(&c))
                .unwrap_or_default()
        } else {
            XimeConfig::default()
        };

        let merged = XimeConfig {
            wubi_radicals: existing.wubi_radicals,
            style: existing.style,
            color_schemes: existing.color_schemes,
            smart_suggestion: self.smart_suggestion.clone(),
        };

        merged.save()
    }

    pub fn get_last_key_root_binding(&self) -> String {
        self.wubi_radicals.hotkeys.show_key.clone()
    }

    pub fn get_primary_color(&self) -> (u8, u8, u8) {
        let scheme_name = &self.style.color_scheme;
        if let Some(scheme) = self.color_schemes.get(scheme_name) {
            (
                (scheme.primary_color >> 16) as u8,
                (scheme.primary_color >> 8) as u8,
                scheme.primary_color as u8,
            )
        } else {
            (0x8F, 0x73, 0xE2)
        }
    }

    pub fn get_primary_color_u32(&self) -> u32 {
        let scheme_name = &self.style.color_scheme;
        if let Some(scheme) = self.color_schemes.get(scheme_name) {
            scheme.primary_color
        } else {
            0x8F73E2
        }
    }

    pub fn get_root_for_key(&self, schema_id: &str, key: char) -> Option<String> {
        let radicals = self.wubi_radicals.schema_radicals.schemas.get(schema_id)?;
        let root = match key.to_lowercase().next()? {
            'g' => &radicals.g,
            'f' => &radicals.f,
            'd' => &radicals.d,
            's' => &radicals.s,
            'a' => &radicals.a,
            'h' => &radicals.h,
            'j' => &radicals.j,
            'k' => &radicals.k,
            'l' => &radicals.l,
            'm' => &radicals.m,
            't' => &radicals.t,
            'r' => &radicals.r,
            'e' => &radicals.e,
            'w' => &radicals.w,
            'q' => &radicals.q,
            'y' => &radicals.y,
            'u' => &radicals.u,
            'i' => &radicals.i,
            'o' => &radicals.o,
            'p' => &radicals.p,
            'n' => &radicals.n,
            'b' => &radicals.b,
            'v' => &radicals.v,
            'c' => &radicals.c,
            'x' => &radicals.x,
            _ => return None,
        };
        if root.is_empty() {
            None
        } else {
            Some(root.clone())
        }
    }

    pub fn user_data_dir() -> PathBuf {
        Self::rime_user_data_dir()
    }

    pub fn shared_data_dir() -> PathBuf {
        let exe_path = std::env::current_exe().unwrap_or_default();
        exe_path
            .parent()
            .map(|p| p.join("data"))
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_wubi_root_config() {
        let yaml = r#"
g: "王龶五一戋"
f: "土士二干"
d: "大犬三"
"#;
        let config: WubiRootConfig = serde_saphyr::from_str(yaml).unwrap();
        assert_eq!(config.g, "王龶五一戋");
        assert_eq!(config.f, "土士二干");
        assert_eq!(config.d, "大犬三");
        assert_eq!(config.s, ""); // default
    }

    #[test]
    fn test_parse_schema_radicals_with_flatten() {
        let yaml = r#"
wubi86:
  g: "王龶五一戋"
  f: "土士二干"
wubi98:
  g: "王王"
  f: "土士"
"#;
        let config: SchemaRadicalsConfig = serde_saphyr::from_str(yaml).unwrap();
        assert_eq!(config.schemas.len(), 2);
        assert_eq!(config.schemas.get("wubi86").unwrap().g, "王龶五一戋");
        assert_eq!(config.schemas.get("wubi86").unwrap().f, "土士二干");
        assert_eq!(config.schemas.get("wubi98").unwrap().g, "王王");
    }

    #[test]
    fn test_parse_schema_radicals_with_yaml_anchors() {
        let yaml = r#"
wubi86: &wubi86_radicals
  g: "王龶五一戋"
  f: "土士二干"
  d: "大犬三"
wubi86_pinyin: *wubi86_radicals
"#;
        let config: SchemaRadicalsConfig = serde_saphyr::from_str(yaml).unwrap();
        assert_eq!(config.schemas.len(), 2);
        assert_eq!(config.schemas.get("wubi86").unwrap().g, "王龶五一戋");
        assert_eq!(config.schemas.get("wubi86_pinyin").unwrap().g, "王龶五一戋");
        assert_eq!(config.schemas.get("wubi86_pinyin").unwrap().d, "大犬三");
    }

    #[test]
    fn test_parse_wubi_radicals_config() {
        let yaml = r#"
hotkeys:
  show_key: Ctrl
  show_all_keys: Shift+Ctrl
schema_radicals:
  wubi86:
    g: "王"
    f: "土"
"#;
        let config: WubiRadicalsConfig = serde_saphyr::from_str(yaml).unwrap();
        assert_eq!(config.hotkeys.show_key, "Ctrl");
        assert_eq!(config.hotkeys.show_all_keys, "Shift+Ctrl");
        assert_eq!(config.schema_radicals.schemas.len(), 1);
        assert_eq!(
            config.schema_radicals.schemas.get("wubi86").unwrap().g,
            "王"
        );
    }

    #[test]
    fn test_parse_wubi_root_config_25_fields() {
        // Parse only the WubiRootConfig (25 fields, no flatten)
        let yaml = r#"
g: "王龶五一戋"
f: "土士二干十寸雨"
d: "大犬三古石厂"
s: "木丁西"
a: "工匚戈艹廿"
h: "目丨卜上止"
j: "日曰早虫"
k: "口川"
l: "田甲囗四"
m: "山由贝"
t: "禾竹丿彳"
r: "白手扌斤"
e: "月彡乃用"
w: "人亻八"
q: "金钅犭"
y: "言讠文方"
u: "立辛冫"
i: "水氵小"
o: "火灬米"
p: "之辶宀"
n: "已己巳心"
b: "子耳了也"
v: "女刀九臼"
c: "又巴马"
x: "弓匕纟"
"#;
        let config: WubiRootConfig = serde_saphyr::from_str(yaml).unwrap();
        assert_eq!(config.g, "王龶五一戋");
        assert_eq!(config.x, "弓匕纟");
        assert_eq!(config.t, "禾竹丿彳");
    }

    #[test]
    fn test_parse_xime_config_wubi_only() {
        // Parse XimeConfig with wubi_radicals but no style/color_schemes
        let yaml = r#"
wubi_radicals:
  hotkeys:
    show_key: Ctrl
  schema_radicals:
    wubi86:
      g: "王龶五一戋"
      f: "土士二干十寸雨"
"#;
        let config: XimeConfig = serde_saphyr::from_str(yaml).unwrap();
        assert_eq!(config.wubi_radicals.hotkeys.show_key, "Ctrl");
        assert_eq!(config.wubi_radicals.schema_radicals.schemas.len(), 1);
    }

    #[test]
    fn test_parse_xime_config_wubi_and_style() {
        // Parse with wubi_radicals + style (cross-field boundary test)
        let yaml = r#"
wubi_radicals:
  hotkeys:
    show_key: Ctrl
  schema_radicals:
    wubi86:
      g: "王龶五一戋"
      f: "土士二干十寸雨"

style:
  font_size: 16
  horizontal: true
"#;
        let config: XimeConfig = serde_saphyr::from_str(yaml).unwrap();
        assert_eq!(config.wubi_radicals.hotkeys.show_key, "Ctrl");
        assert_eq!(config.wubi_radicals.schema_radicals.schemas.len(), 1);
        assert_eq!(config.style.font_size, 16.0);
        assert!(config.style.horizontal);
    }

    #[test]
    fn test_parse_xime_config_full_fields() {
        // Full 25 fields with style + color_schemes, no YAML anchors
        let yaml = r#"
wubi_radicals:
  hotkeys:
    show_key: Ctrl
  schema_radicals:
    wubi86:
      g: "王龶五一戋"
      f: "土士二干十寸雨"
      d: "大犬三古石厂"
      s: "木丁西"
      a: "工匚戈艹廿"
      h: "目丨卜上止"
      j: "日曰早虫"
      k: "口川"
      l: "田甲囗四"
      m: "山由贝"
      t: "禾竹丿彳"
      r: "白手扌斤"
      e: "月彡乃用"
      w: "人亻八"
      q: "金钅犭"
      y: "言讠文方"
      u: "立辛冫"
      i: "水氵小"
      o: "火灬米"
      p: "之辶宀"
      n: "已己巳心"
      b: "子耳了也"
      v: "女刀九臼"
      c: "又巴马"
      x: "弓匕纟"

style:
  font_family: "Microsoft YaHei UI"
  font_size: 16
  horizontal: true

color_schemes:
  test_theme:
    name: "测试主题"
    primary_color: 0xFF0000
"#;
        let config = XimeConfig::parse_yaml(yaml).unwrap();

        // Check wubi_radicals
        assert_eq!(config.wubi_radicals.hotkeys.show_key, "Ctrl");
        let schemas = &config.wubi_radicals.schema_radicals.schemas;
        let keys: Vec<&String> = schemas.keys().collect();
        assert_eq!(
            schemas.len(),
            1,
            "expected 1 schema entry, got {}: {:?}",
            schemas.len(),
            keys
        );
        let wubi86 = schemas.get("wubi86").unwrap();
        assert_eq!(wubi86.g, "王龶五一戋");
        assert_eq!(wubi86.f, "土士二干十寸雨");
        assert_eq!(wubi86.a, "工匚戈艹廿");
        assert_eq!(wubi86.x, "弓匕纟");

        // Check style
        assert_eq!(config.style.font_family, "Microsoft YaHei UI");
        assert_eq!(config.style.font_size, 16.0);
        assert!(config.style.horizontal);

        // Check color_schemes
        assert_eq!(config.color_schemes.len(), 1);
        let theme = config.color_schemes.get("test_theme").unwrap();
        assert_eq!(theme.name, "测试主题");
        assert_eq!(theme.primary_color, 0xFF0000);
    }

    #[test]
    fn test_parse_xime_config_with_yaml_anchors() {
        // This mimics the user's actual xime.yaml structure
        let yaml = r#"
wubi_radicals:
  hotkeys:
    show_key: Ctrl
    show_all_keys: ""
  schema_radicals:
    wubi86: &wubi86_radicals
      g: "王龶五一戋"
      f: "土士二干十寸雨"
      d: "大犬三古石厂"
    wubi86_pinyin: *wubi86_radicals

style:
  font_size: 14
  font_family: "Microsoft YaHei UI"
"#;
        let config: XimeConfig = serde_saphyr::from_str(yaml).unwrap();

        // This is the critical test: flatten with YAML anchors
        assert!(
            !config.wubi_radicals.schema_radicals.schemas.is_empty(),
            "schema_radicals.schemas should NOT be empty! Got schemas={:?}",
            config.wubi_radicals.schema_radicals.schemas
        );

        let schemas = &config.wubi_radicals.schema_radicals.schemas;
        assert_eq!(schemas.len(), 2);
        assert_eq!(schemas.get("wubi86").unwrap().g, "王龶五一戋");
        assert_eq!(schemas.get("wubi86_pinyin").unwrap().g, "王龶五一戋");
    }

    #[test]
    fn test_merge_configs_overrides_fields() {
        let base = XimeConfig {
            wubi_radicals: WubiRadicalsConfig {
                hotkeys: HotkeyConfig {
                    show_key: "Ctrl".into(),
                    show_all_keys: String::new(),
                },
                schema_radicals: SchemaRadicalsConfig {
                    schemas: {
                        let mut m = HashMap::new();
                        m.insert(
                            "wubi86".into(),
                            WubiRootConfig {
                                g: "王".into(),
                                ..Default::default()
                            },
                        );
                        m
                    },
                },
            },
            style: StyleConfig {
                font_size: 12.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let over = XimeConfig {
            wubi_radicals: WubiRadicalsConfig {
                hotkeys: HotkeyConfig {
                    show_key: "Shift".into(),
                    show_all_keys: String::new(),
                },
                schema_radicals: SchemaRadicalsConfig {
                    schemas: {
                        let mut m = HashMap::new();
                        m.insert(
                            "wubi86".into(),
                            WubiRootConfig {
                                g: "王王".into(),
                                ..Default::default()
                            },
                        );
                        m
                    },
                },
            },
            style: StyleConfig {
                font_size: 16.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let merged = XimeConfig::merge_configs(base, over);
        assert_eq!(merged.wubi_radicals.hotkeys.show_key, "Shift");
        assert_eq!(
            merged
                .wubi_radicals
                .schema_radicals
                .schemas
                .get("wubi86")
                .unwrap()
                .g,
            "王王"
        );
        assert_eq!(merged.style.font_size, 16.0);
    }

    #[test]
    fn test_merge_configs_preserves_base_when_over_empty() {
        let base = XimeConfig {
            wubi_radicals: WubiRadicalsConfig {
                hotkeys: HotkeyConfig {
                    show_key: "Ctrl".into(),
                    show_all_keys: String::new(),
                },
                schema_radicals: SchemaRadicalsConfig {
                    schemas: {
                        let mut m = HashMap::new();
                        m.insert(
                            "wubi86".into(),
                            WubiRootConfig {
                                g: "王".into(),
                                ..Default::default()
                            },
                        );
                        m
                    },
                },
            },
            ..Default::default()
        };

        let over = XimeConfig::default(); // empty schemas

        let merged = XimeConfig::merge_configs(base, over);
        // Since over.schema_radicals.schemas is empty, base should be preserved
        assert_eq!(merged.wubi_radicals.schema_radicals.schemas.len(), 1);
        assert_eq!(
            merged
                .wubi_radicals
                .schema_radicals
                .schemas
                .get("wubi86")
                .unwrap()
                .g,
            "王"
        );
    }

    #[test]
    fn test_parse_user_xime_yaml_from_file() {
        // Read the actual file from the user data directory
        let user_data_dir = XimeConfig::rime_user_data_dir();
        let user_yaml = user_data_dir.join("xime.yaml");
        if !user_yaml.exists() {
            eprintln!("Skipping: user xime.yaml not found at {:?}", user_yaml);
            return;
        }
        let content = std::fs::read_to_string(&user_yaml).unwrap();
        let config = XimeConfig::parse_yaml(&content).unwrap();
        eprintln!("Parsed config: {:#?}", config);
        assert!(
            !config.wubi_radicals.schema_radicals.schemas.is_empty(),
            "wubi_radicals.schema_radicals.schemas should NOT be empty after parsing {:?}",
            user_yaml
        );
    }

    #[test]
    fn test_serialize_roundtrip() {
        let config = XimeConfig {
            wubi_radicals: WubiRadicalsConfig {
                hotkeys: HotkeyConfig {
                    show_key: "Ctrl".into(),
                    show_all_keys: String::new(),
                },
                schema_radicals: SchemaRadicalsConfig {
                    schemas: {
                        let mut m = HashMap::new();
                        m.insert(
                            "wubi86".into(),
                            WubiRootConfig {
                                g: "王".into(),
                                ..Default::default()
                            },
                        );
                        m
                    },
                },
            },
            ..Default::default()
        };

        let yaml = serde_saphyr::to_string(&config).unwrap();
        let deserialized: XimeConfig = serde_saphyr::from_str(&yaml).unwrap();
        assert_eq!(deserialized.wubi_radicals.hotkeys.show_key, "Ctrl");
        assert_eq!(
            deserialized
                .wubi_radicals
                .schema_radicals
                .schemas
                .get("wubi86")
                .unwrap()
                .g,
            "王"
        );
    }
}
