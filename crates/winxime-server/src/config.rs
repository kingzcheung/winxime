use std::fs;
use std::path::PathBuf;
use serde::Deserialize;
use std::collections::HashMap;
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
use windows_core::HSTRING;

#[derive(Debug, Deserialize, Default)]
pub struct XimeConfig {
    #[serde(default)]
    pub style: StyleConfig,
    #[serde(default)]
    pub color_schemes: HashMap<String, ColorScheme>,
}

#[derive(Debug, Deserialize)]
pub struct StyleConfig {
    #[serde(default)]
    pub font_family: String,
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    #[serde(default = "default_candidate_count")]
    pub candidate_count: i32,
    #[serde(default)]
    pub show_code_hint: bool,
    #[serde(default = "default_horizontal")]
    pub horizontal: bool,
    #[serde(default = "default_color_scheme")]
    pub color_scheme: String,
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            font_family: String::new(),
            font_size: default_font_size(),
            candidate_count: default_candidate_count(),
            show_code_hint: false,
            horizontal: default_horizontal(),
            color_scheme: default_color_scheme(),
        }
    }
}

fn default_font_size() -> f32 { 14.0 }
fn default_candidate_count() -> i32 { 5 }
fn default_horizontal() -> bool { true }
fn default_color_scheme() -> String { "lavender_purple".to_string() }

#[derive(Debug, Deserialize)]
pub struct ColorScheme {
    #[serde(default)]
    pub name: String,
    #[serde(deserialize_with = "deserialize_hex_color", default = "default_primary_color")]
    pub primary_color: u32,
}

fn default_primary_color() -> u32 { 0x8F73E2 }

fn deserialize_hex_color<'de, D>(deserializer: D) -> Result<u32, D::Error>
where D: serde::Deserializer<'de> {
    let value: serde_yaml::Value = serde::Deserialize::deserialize(deserializer)?;
    match value {
        serde_yaml::Value::Number(n) => {
            if let Some(num) = n.as_u64() {
                Ok(num as u32)
            } else {
                Ok(0x8F73E2)
            }
        }
        serde_yaml::Value::String(s) => {
            let s = s.trim();
            if s.starts_with("0x") || s.starts_with("0X") {
                u32::from_str_radix(&s[2..], 16).map_err(|_| serde::de::Error::custom("Invalid hex color"))
            } else if s.starts_with('#') {
                u32::from_str_radix(&s[1..], 16).map_err(|_| serde::de::Error::custom("Invalid hex color"))
            } else {
                s.parse::<u32>().map_err(|_| serde::de::Error::custom("Invalid color number"))
            }
        }
        _ => Ok(0x8F73E2),
    }
}

pub struct UiConfig {
    pub font_family: HSTRING,
    pub font_size: f32,
    pub candidate_count: u32,
    pub show_code_hint: bool,
    pub horizontal: bool,
    pub color_scheme: String,
    pub primary_color: u32,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            font_family: HSTRING::from("Microsoft YaHei UI"),
            font_size: 14.0,
            candidate_count: 5,
            show_code_hint: false,
            horizontal: true,
            color_scheme: "lavender_purple".to_string(),
            primary_color: 0x8F73E2,
        }
    }
}

impl UiConfig {
    pub fn load() -> Self {
        let config = XimeConfig::load();
        
        let font_family = if config.style.font_family.is_empty() {
            HSTRING::from("Microsoft YaHei UI")
        } else {
            HSTRING::from(config.style.font_family.as_str())
        };
        
        let primary_color = config.get_primary_color();
        
        Self {
            font_family,
            font_size: config.style.font_size,
            candidate_count: config.style.candidate_count as u32,
            show_code_hint: config.style.show_code_hint,
            horizontal: config.style.horizontal,
            color_scheme: config.style.color_scheme,
            primary_color,
        }
    }
    
    pub fn get_colors(&self) -> (D2D1_COLOR_F, D2D1_COLOR_F, D2D1_COLOR_F, D2D1_COLOR_F, D2D1_COLOR_F, D2D1_COLOR_F, D2D1_COLOR_F) {
        let primary = hex_to_rgb(self.primary_color);

        let bg_color = D2D1_COLOR_F { r: 0.95, g: 0.95, b: 0.95, a: 0.85 };
        let border_color = D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 0.05 };
        let fg_color = D2D1_COLOR_F { r: 0.2, g: 0.2, b: 0.2, a: 1.0 };
        let selkey_color = D2D1_COLOR_F { r: 0.4, g: 0.4, b: 0.4, a: 1.0 };
        let comment_color = D2D1_COLOR_F { r: 0.5, g: 0.5, b: 0.5, a: 0.7 };
        let highlight_bg_color = D2D1_COLOR_F {
            r: primary.0,
            g: primary.1,
            b: primary.2,
            a: 0.9,
        };
        let highlight_fg_color = D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

        (bg_color, border_color, fg_color, selkey_color, comment_color, highlight_bg_color, highlight_fg_color)
    }
}

impl XimeConfig {
    pub fn load() -> Self {
        let (system_path, user_path) = Self::get_config_paths();
        
        let system_config = Self::load_from_path(&system_path).unwrap_or_default();
        let user_config = Self::load_from_path(&user_path);
        
        Self::merge_configs(system_config, user_config)
    }
    
    fn get_config_paths() -> (PathBuf, PathBuf) {
        #[cfg(debug_assertions)]
        {
            let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let workspace_dir = manifest_dir.parent().unwrap().parent().unwrap();
            (
                workspace_dir.join("resources").join("xime.yaml"),
                workspace_dir.join("target").join("debug").join("user-data").join("xime.custom.yaml"),
            )
        }
        #[cfg(not(debug_assertions))]
        {
            let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::new());
            let install_dir = exe_path.parent().unwrap_or_else(|| PathBuf::new().as_path()).parent().unwrap_or_else(|| PathBuf::new().as_path());
            let appdata = std::env::var("APPDATA").unwrap_or_else(|_| String::new());
            (
                install_dir.join("data").join("xime.yaml"),
                PathBuf::from(&appdata).join("Xime").join("rime").join("xime.custom.yaml"),
            )
        }
    }
    
    fn load_from_path(path: &PathBuf) -> Option<Self> {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(config) = serde_yaml::from_str::<XimeConfig>(&content) {
                    tracing::info!("Loaded config from {:?}: color_scheme={}", path, config.style.color_scheme);
                    return Some(config);
                }
            }
        }
        None
    }
    
    fn merge_configs(system: Self, user: Option<Self>) -> Self {
        match user {
            Some(user) => Self {
                style: StyleConfig {
                    font_family: if user.style.font_family.is_empty() { system.style.font_family } else { user.style.font_family },
                    font_size: if user.style.font_size == 0.0 { system.style.font_size } else { user.style.font_size },
                    candidate_count: if user.style.candidate_count == 0 { system.style.candidate_count } else { user.style.candidate_count },
                    show_code_hint: user.style.show_code_hint,
                    horizontal: user.style.horizontal,
                    color_scheme: if user.style.color_scheme.is_empty() { system.style.color_scheme } else { user.style.color_scheme },
                },
                color_schemes: if user.color_schemes.is_empty() { system.color_schemes } else { user.color_schemes },
            },
            None => system,
        }
    }
    
    pub fn get_primary_color(&self) -> u32 {
        if let Some(scheme) = self.color_schemes.get(&self.style.color_scheme) {
            scheme.primary_color
        } else {
            0x8F73E2
        }
    }
}

fn hex_to_rgb(hex: u32) -> (f32, f32, f32) {
    let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
    let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
    let b = (hex & 0xFF) as f32 / 255.0;
    (r, g, b)
}