use librime_sys2::*;
use librime::get_api;
use std::ffi::{CStr, CString};
use std::os::raw::c_int;
use std::ptr;
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
use windows_core::HSTRING;

pub struct UiConfig {
    pub font_family: HSTRING,
    pub font_size: f32,
    pub candidate_count: u32,
    pub show_code_hint: bool,
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
            color_scheme: "lavender_purple".to_string(),
            primary_color: 0x8F73E2,
        }
    }
}

impl UiConfig {
    pub fn load() -> Self {
        let api = get_api();
        if api.is_null() {
            return Self::default();
        }

        unsafe {
            let config_id = CString::new("xime").unwrap_or_default();
            let mut config = RimeConfig { ptr: ptr::null_mut() };

            if let Some(config_open) = (*api).config_open {
                if config_open(config_id.as_ptr(), &mut config) == FALSE {
                    return Self::default();
                }
            } else {
                return Self::default();
            }

            let result = Self::read_config(&mut config, api);

            if let Some(config_close) = (*api).config_close {
                config_close(&mut config);
            }

            result
        }
    }

    unsafe fn read_config(config: &mut RimeConfig, api: *mut RimeApi) -> Self {
        let mut ui_config = Self::default();

        if let Some(get_cstring) = (*api).config_get_cstring {
            let key = CString::new("style/font_family").unwrap_or_default();
            let value_ptr = get_cstring(config, key.as_ptr());
            if !value_ptr.is_null() {
                let value = CStr::from_ptr(value_ptr).to_string_lossy();
                ui_config.font_family = HSTRING::from(value.as_ref());
            }
        }

        if let Some(get_int) = (*api).config_get_int {
            let key = CString::new("style/font_size").unwrap_or_default();
            let mut value: c_int = 0;
            if get_int(config, key.as_ptr(), &mut value) != FALSE {
                ui_config.font_size = value as f32;
            }
        }

        if let Some(get_int) = (*api).config_get_int {
            let key = CString::new("style/candidate_count").unwrap_or_default();
            let mut value: c_int = 0;
            if get_int(config, key.as_ptr(), &mut value) != FALSE {
                ui_config.candidate_count = value as u32;
            }
        }

        if let Some(get_bool) = (*api).config_get_bool {
            let key = CString::new("style/show_code_hint").unwrap_or_default();
            let mut value: Bool = FALSE;
            if get_bool(config, key.as_ptr(), &mut value) != FALSE {
                ui_config.show_code_hint = value != FALSE;
            }
        }

        if let Some(get_cstring) = (*api).config_get_cstring {
            let key = CString::new("style/color_scheme").unwrap_or_default();
            let value_ptr = get_cstring(config, key.as_ptr());
            if !value_ptr.is_null() {
                ui_config.color_scheme = CStr::from_ptr(value_ptr).to_string_lossy().to_string();
            }
        }

        if !ui_config.color_scheme.is_empty() {
            if let Some(get_int) = (*api).config_get_int {
                let key = CString::new(format!("color_schemes/{}/primary_color", ui_config.color_scheme)).unwrap_or_default();
                let mut value: c_int = 0;
                if get_int(config, key.as_ptr(), &mut value) != FALSE {
                    ui_config.primary_color = value as u32;
                }
            }
        }

        ui_config
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

fn hex_to_rgb(hex: u32) -> (f32, f32, f32) {
    let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
    let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
    let b = (hex & 0xFF) as f32 / 255.0;
    (r, g, b)
}