use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct XimeConfig {
    #[serde(default)]
    pub hotkeys: HotkeyConfig,
    #[serde(default)]
    pub wubi_root: WubiRootConfig,
    #[serde(default)]
    pub style: StyleConfig,
    #[serde(default)]
    pub color_schemes: HashMap<String, ColorScheme>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
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
            show_code_hint: false,
            horizontal: default_horizontal(),
            corner_radius: default_corner_radius(),
            color_scheme: default_color_scheme(),
        }
    }
}

fn default_font_size() -> f32 { 14.0 }
fn default_candidate_count() -> i32 { 5 }
fn default_horizontal() -> bool { true }
fn default_corner_radius() -> f32 { 8.0 }
fn default_color_scheme() -> String { "lavender_purple".to_string() }

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
pub struct ColorScheme {
    #[serde(default)]
    pub name: String,
    #[serde(deserialize_with = "deserialize_hex_color", default = "default_primary_color")]
    pub primary_color: u32,
}

fn default_primary_color() -> u32 { 0x8F73E2 }

fn deserialize_hex_color<'de, D>(deserializer: D) -> Result<u32, D::Error>
where D: serde::Deserializer<'de>,
{
    let value: serde_yaml::Value = serde::Deserialize::deserialize(deserializer)?;
    match value {
        serde_yaml::Value::Number(n) => n.as_u64().map(|num| num as u32).or_else(|| Some(0x8F73E2)),
        serde_yaml::Value::String(s) => {
            let s = s.trim();
            if s.starts_with("0x") || s.starts_with("0X") {
                u32::from_str_radix(&s[2..], 16).ok()
            } else if s.starts_with('#') {
                u32::from_str_radix(&s[1..], 16).ok()
            } else {
                s.parse::<u32>().ok()
            }
        }
        _ => Some(0x8F73E2),
    }.ok_or_else(|| serde::de::Error::custom("Invalid color"))
}

#[derive(Debug, Deserialize)]
pub struct HotkeyConfig {
    #[serde(default = "default_show_last_key_root")]
    pub show_last_key_root: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self { show_last_key_root: default_show_last_key_root() }
    }
}

fn default_show_last_key_root() -> String { "Ctrl".to_string() }

#[derive(Debug, Deserialize, Default)]
pub struct WubiRootConfig {
    #[serde(default)] pub g: String,
    #[serde(default)] pub f: String,
    #[serde(default)] pub d: String,
    #[serde(default)] pub s: String,
    #[serde(default)] pub a: String,
    #[serde(default)] pub h: String,
    #[serde(default)] pub j: String,
    #[serde(default)] pub k: String,
    #[serde(default)] pub l: String,
    #[serde(default)] pub m: String,
    #[serde(default)] pub t: String,
    #[serde(default)] pub r: String,
    #[serde(default)] pub e: String,
    #[serde(default)] pub w: String,
    #[serde(default)] pub q: String,
    #[serde(default)] pub y: String,
    #[serde(default)] pub u: String,
    #[serde(default)] pub i: String,
    #[serde(default)] pub o: String,
    #[serde(default)] pub p: String,
    #[serde(default)] pub n: String,
    #[serde(default)] pub b: String,
    #[serde(default)] pub v: String,
    #[serde(default)] pub c: String,
    #[serde(default)] pub x: String,
}

impl XimeConfig {
    pub fn load() -> Self {
        let system_config = Self::load_system_config();
        let user_config = Self::load_user_config();
        Self::merge_configs(system_config, user_config)
    }
    
    fn load_system_config() -> Self {
        let exe_path = std::env::current_exe().unwrap_or_default();
        let system_path = exe_path.parent().map(|p| p.join("data").join("xime.yaml")).unwrap_or_default();
        if system_path.exists() {
            if let Ok(content) = fs::read_to_string(&system_path) {
                if let Ok(config) = serde_yaml::from_str::<XimeConfig>(&content) {
                    return config;
                }
            }
        }
        Self::builtin_default()
    }
    
    fn builtin_default() -> Self {
        const DEFAULT_CONFIG: &[u8] = include_bytes!("../resources/xime.yaml");
        serde_yaml::from_slice(DEFAULT_CONFIG).unwrap_or_default()
    }
    
    fn load_user_config() -> Option<Self> {
        let config_path = Self::user_config_path();
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                return serde_yaml::from_str::<XimeConfig>(&content).ok();
            }
        }
        None
    }
    
    fn merge_configs(system: Self, user: Option<Self>) -> Self {
        match user {
            Some(user) => Self {
                hotkeys: user.hotkeys,
                wubi_root: if user.wubi_root.g.is_empty() { system.wubi_root } else { user.wubi_root },
                style: user.style,
                color_schemes: if user.color_schemes.is_empty() { system.color_schemes } else { user.color_schemes },
            },
            None => system,
        }
    }
    
    fn user_config_path() -> PathBuf {
        std::env::var("APPDATA")
            .map(|p| PathBuf::from(p).join("Xime").join("rime").join("xime.yaml"))
            .unwrap_or_else(|_| PathBuf::from("xime.yaml"))
    }

    pub fn config_path() -> PathBuf { Self::user_config_path() }

    pub fn get_last_key_root_binding(&self) -> String {
        self.hotkeys.show_last_key_root.clone()
    }

    pub fn get_primary_color(&self) -> (u8, u8, u8) {
        let scheme_name = &self.style.color_scheme;
        if let Some(scheme) = self.color_schemes.get(scheme_name) {
            ((scheme.primary_color >> 16) as u8, (scheme.primary_color >> 8) as u8, scheme.primary_color as u8)
        } else {
            (0x8F, 0x73, 0xE2)
        }
    }

    pub fn get_root_for_key(&self, key: char) -> Option<String> {
        let root = match key.to_lowercase().next()? {
            'g' => &self.wubi_root.g, 'f' => &self.wubi_root.f, 'd' => &self.wubi_root.d,
            's' => &self.wubi_root.s, 'a' => &self.wubi_root.a, 'h' => &self.wubi_root.h,
            'j' => &self.wubi_root.j, 'k' => &self.wubi_root.k, 'l' => &self.wubi_root.l,
            'm' => &self.wubi_root.m, 't' => &self.wubi_root.t, 'r' => &self.wubi_root.r,
            'e' => &self.wubi_root.e, 'w' => &self.wubi_root.w, 'q' => &self.wubi_root.q,
            'y' => &self.wubi_root.y, 'u' => &self.wubi_root.u, 'i' => &self.wubi_root.i,
            'o' => &self.wubi_root.o, 'p' => &self.wubi_root.p, 'n' => &self.wubi_root.n,
            'b' => &self.wubi_root.b, 'v' => &self.wubi_root.v, 'c' => &self.wubi_root.c,
            'x' => &self.wubi_root.x, _ => return None,
        };
        if root.is_empty() { None } else { Some(root.clone()) }
    }

    pub fn user_data_dir() -> PathBuf {
        std::env::var("APPDATA")
            .map(|p| PathBuf::from(p).join("Xime").join("rime"))
            .unwrap_or_else(|_| PathBuf::from("."))
    }

    pub fn shared_data_dir() -> PathBuf {
        let exe_path = std::env::current_exe().unwrap_or_default();
        exe_path.parent().map(|p| p.join("data")).unwrap_or_default()
    }
}