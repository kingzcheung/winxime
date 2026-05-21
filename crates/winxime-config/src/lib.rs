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
    pub fn load() -> Self {
        let system_config = Self::load_system_config();
        let user_config = Self::load_user_config();
        Self::merge_configs(system_config, user_config)
    }

    pub fn save(&self) -> Result<(), String> {
        let config_path = Self::user_config_path();

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir: {}", e))?;
        }

        let content = serde_saphyr::to_string(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&config_path, content).map_err(|e| format!("Failed to write config: {}", e))?;

        Ok(())
    }

    pub fn save_smart_suggestion(&self) -> Result<(), String> {
        let config_path = Self::user_config_path();

        let existing: XimeConfig = if config_path.exists() {
            fs::read_to_string(&config_path)
                .ok()
                .and_then(|c| serde_saphyr::from_str(&c).ok())
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

    fn load_system_config() -> Self {
        let exe_path = std::env::current_exe().unwrap_or_default();
        let system_path = exe_path
            .parent()
            .map(|p| p.join("data").join("xime.yaml"))
            .unwrap_or_default();
        if system_path.exists() {
            if let Ok(content) = fs::read_to_string(&system_path) {
                if let Ok(config) = serde_saphyr::from_str::<XimeConfig>(&content) {
                    return config;
                }
            }
        }
        Self::builtin_default()
    }

    fn builtin_default() -> Self {
        const DEFAULT_CONFIG: &[u8] = include_bytes!("../resources/xime.yaml");
        serde_saphyr::from_slice(DEFAULT_CONFIG).unwrap_or_default()
    }

    fn load_user_config() -> Option<Self> {
        let config_path = Self::user_config_path();
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                return serde_saphyr::from_str::<XimeConfig>(&content).ok();
            }
        }
        None
    }

    fn merge_configs(system: Self, user: Option<Self>) -> Self {
        match user {
            Some(user) => Self {
                wubi_radicals: WubiRadicalsConfig {
                    hotkeys: user.wubi_radicals.hotkeys,
                    schema_radicals: SchemaRadicalsConfig {
                        schemas: if user.wubi_radicals.schema_radicals.schemas.is_empty() {
                            system.wubi_radicals.schema_radicals.schemas
                        } else {
                            user.wubi_radicals.schema_radicals.schemas
                        },
                    },
                },
                style: user.style,
                color_schemes: if user.color_schemes.is_empty() {
                    system.color_schemes
                } else {
                    user.color_schemes
                },
                smart_suggestion: SmartSuggestionConfig {
                    enabled: user.smart_suggestion.enabled,
                    suggestion_count: user.smart_suggestion.suggestion_count,
                    prefer_common_words: user.smart_suggestion.prefer_common_words,
                    record_user_frequency: user.smart_suggestion.record_user_frequency,
                    auto_adjust_frequency: user.smart_suggestion.auto_adjust_frequency,
                    learning_threshold: user.smart_suggestion.learning_threshold,
                    model: SmartSuggestionModelConfig {
                        provider: if user.smart_suggestion.model.provider.is_empty() {
                            system.smart_suggestion.model.provider
                        } else {
                            user.smart_suggestion.model.provider
                        },
                        name: if user.smart_suggestion.model.name.is_empty() {
                            system.smart_suggestion.model.name
                        } else {
                            user.smart_suggestion.model.name
                        },
                        auto_download: user.smart_suggestion.model.auto_download,
                        files: if user.smart_suggestion.model.files.is_empty() {
                            system.smart_suggestion.model.files
                        } else {
                            user.smart_suggestion.model.files
                        },
                    },
                },
            },
            None => system,
        }
    }

    fn user_config_path() -> PathBuf {
        std::env::var("APPDATA")
            .map(|p| PathBuf::from(p).join("Xime").join("rime").join("xime.yaml"))
            .unwrap_or_else(|_| PathBuf::from("xime.yaml"))
    }

    pub fn config_path() -> PathBuf {
        Self::user_config_path()
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
        std::env::var("APPDATA")
            .map(|p| PathBuf::from(p).join("Xime").join("rime"))
            .unwrap_or_else(|_| PathBuf::from("."))
    }

    pub fn shared_data_dir() -> PathBuf {
        let exe_path = std::env::current_exe().unwrap_or_default();
        exe_path
            .parent()
            .map(|p| p.join("data"))
            .unwrap_or_default()
    }
}
