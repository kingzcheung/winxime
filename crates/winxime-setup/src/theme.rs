use gpui::*;

#[derive(Clone)]
pub enum SystemTheme {
    Light,
    Dark,
}

impl SystemTheme {
    pub fn detect() -> Self {
        #[cfg(windows)]
        {
            use windows::Win32::System::Registry::*;
            use windows::core::w;

            let key_path = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
            let value_name = w!("AppsUseLightTheme");

            let mut handle = HKEY::default();
            
            if unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, key_path, Some(0), KEY_READ, &mut handle).is_ok() } {
                let mut data: u32 = 0;
                let mut data_size: u32 = 4;

                let result = unsafe {
                    RegQueryValueExW(
                        handle,
                        value_name,
                        None,
                        None,
                        Some(&mut data as *mut _ as *mut u8),
                        Some(&mut data_size),
                    )
                };

                unsafe { let _ = RegCloseKey(handle); };

                if result.is_ok() {
                    if data == 0 {
                        return SystemTheme::Dark;
                    }
                }
            }
        }

        SystemTheme::Light
    }

    pub fn is_dark(&self) -> bool {
        matches!(self, SystemTheme::Dark)
    }
}

#[derive(Clone)]
pub struct ThemeColors {
    pub background: Hsla,
    pub surface: Hsla,
    pub surface_variant: Hsla,
    pub primary: Hsla,
    pub primary_hover: Hsla,
    pub on_primary: Hsla,
    pub foreground: Hsla,
    pub foreground_muted: Hsla,
    pub border: Hsla,
    pub border_variant: Hsla,
    pub disabled: Hsla,
    pub error: Hsla,
    pub on_error: Hsla,
}

impl ThemeColors {
    pub fn from_theme(theme: &SystemTheme) -> Self {
        if theme.is_dark() {
            Self {
                background: hsla(0.0, 0.0, 0.05, 0.85),
                surface: rgb(0x1a1a1a).into(),
                surface_variant: rgb(0x262626).into(),
                primary: rgb(0x8F73E2).into(),
                primary_hover: rgb(0x7A5FD0).into(),
                on_primary: rgb(0xffffff).into(),
                foreground: rgb(0xe0e0e0).into(),
                foreground_muted: rgb(0x808080).into(),
                border: rgb(0x303030).into(),
                border_variant: rgb(0x404040).into(),
                disabled: rgb(0x4d4d4d).into(),
                error: rgb(0xc42b1c).into(),
                on_error: rgb(0xffffff).into(),
            }
        } else {
            Self {
                background: hsla(0.0, 0.0, 1.0, 0.95),
                surface: rgb(0xffffff).into(),
                surface_variant: rgb(0xf5f5f5).into(),
                primary: rgb(0x7B5DC7).into(),
                primary_hover: rgb(0x6A4DB5).into(),
                on_primary: rgb(0xffffff).into(),
                foreground: rgb(0x1a1a1a).into(),
                foreground_muted: rgb(0x666666).into(),
                border: rgb(0xe0e0e0).into(),
                border_variant: rgb(0xd0d0d0).into(),
                disabled: rgb(0xaaaaaa).into(),
                error: rgb(0xc42b1c).into(),
                on_error: rgb(0xffffff).into(),
            }
        }
    }
}