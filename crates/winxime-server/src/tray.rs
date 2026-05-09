use std::sync::Arc;
use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM, HINSTANCE},
    System::LibraryLoader::GetModuleHandleW,
    System::Registry::{RegCloseKey, RegGetValueW, RegOpenKeyExW, HKEY_CURRENT_USER, KEY_READ, RRF_RT_DWORD},
    UI::{
        Shell::{NIM_ADD, NIM_DELETE, NIM_MODIFY, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIF_STATE, NIS_HIDDEN, Shell_NotifyIconW, NOTIFYICONDATAW, NOTIFY_ICON_STATE},
        WindowsAndMessaging::{
            AppendMenuW, CreateIconFromResource, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu,
            LoadIconW, PostMessageW, PostQuitMessage,
            RegisterClassW,
            WM_COMMAND, WM_DESTROY, WM_LBUTTONUP, WM_RBUTTONUP, WM_USER, WM_SETTINGCHANGE,
            WNDCLASSW, IDI_APPLICATION, IDC_ARROW, LoadCursorW,
            MF_SEPARATOR, MF_STRING,
            WS_POPUP, CW_USEDEFAULT, HMENU, HICON,
        },
    },
};

const ZH_LIGHT_ICO: &[u8] = include_bytes!("../../../resource/trayicon/zh_light.ico");
const ZH_DARK_ICO: &[u8] = include_bytes!("../../../resource/trayicon/zh_dark.ico");
const EN_LIGHT_ICO: &[u8] = include_bytes!("../../../resource/trayicon/en_light.ico");
const EN_DARK_ICO: &[u8] = include_bytes!("../../../resource/trayicon/en_dark.ico");

pub const WM_TRAY_EVENT: u32 = WM_USER + 100;
const WM_TRAY_UPDATE: u32 = WM_USER + 101;
const WM_TRAY_SHOW: u32 = WM_USER + 102;
const WM_TRAY_HIDE: u32 = WM_USER + 103;

const TRAY_ID: u32 = 1;
const MENU_ID_TOGGLE: u32 = 1001;
const MENU_ID_SETTINGS: u32 = 1002;
const MENU_ID_QUIT: u32 = 1003;

pub enum TrayAction {
    ToggleAsciiMode,
    OpenSettings,
    Quit,
}

static mut TRAY_HWND: Option<HWND> = None;
static mut TRAY_MENU: Option<HMENU> = None;
static mut TRAY_ON_ACTION: Option<Arc<dyn Fn(TrayAction) + Send + Sync>> = None;
static mut TRAY_IS_ASCII: bool = false;

pub struct TrayIcon;

impl TrayIcon {
    pub fn new(on_action: Arc<dyn Fn(TrayAction) + Send + Sync>) {
        let hwnd = Self::create_window();
        let menu = unsafe { CreatePopupMenu().unwrap() };
        
        unsafe {
            TRAY_HWND = Some(hwnd);
            TRAY_MENU = Some(menu);
            TRAY_ON_ACTION = Some(on_action);
            
            let toggle_text: Vec<u16> = "切换中/英\0".encode_utf16().collect();
            let settings_text: Vec<u16> = "设置\0".encode_utf16().collect();
            let quit_text: Vec<u16> = "退出\0".encode_utf16().collect();
            
            let _ = AppendMenuW(menu, MF_STRING, MENU_ID_TOGGLE as usize, windows_core::PCWSTR(toggle_text.as_ptr()));
            let _ = AppendMenuW(menu, MF_SEPARATOR, 0, windows_core::PCWSTR::null());
            let _ = AppendMenuW(menu, MF_STRING, MENU_ID_SETTINGS as usize, windows_core::PCWSTR(settings_text.as_ptr()));
            let _ = AppendMenuW(menu, MF_SEPARATOR, 0, windows_core::PCWSTR::null());
            let _ = AppendMenuW(menu, MF_STRING, MENU_ID_QUIT as usize, windows_core::PCWSTR(quit_text.as_ptr()));
        }
        
        Self::add_icon(hwnd);
    }
    
    fn create_window() -> HWND {
        unsafe {
            let hinst = GetModuleHandleW(None).unwrap();
            
            let class_name: Vec<u16> = "XimeTrayWindow\0".encode_utf16().collect();
            
            let wnd_class = WNDCLASSW {
                lpfnWndProc: Some(Self::window_proc),
                hInstance: HINSTANCE(hinst.0),
                lpszClassName: windows_core::PCWSTR(class_name.as_ptr()),
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
                ..Default::default()
            };
            
            RegisterClassW(&wnd_class);
            
            CreateWindowExW(
                windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE::default(),
                windows_core::PCWSTR(class_name.as_ptr()),
                windows_core::PCWSTR::null(),
                WS_POPUP,
                CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT,
                None,
                None,
                Some(HINSTANCE(hinst.0)),
                None,
            ).unwrap()
        }
    }
    
    fn add_icon(hwnd: HWND) {
        unsafe {
            let hicon = load_icon_from_ico(get_icon_for_mode(false));
            let mut nid = NOTIFYICONDATAW::default();
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = TRAY_ID;
            nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
            nid.uCallbackMessage = WM_TRAY_EVENT;
            nid.hIcon = hicon;
            let tip: Vec<u16> = "中\0".encode_utf16().collect();
            for (i, c) in tip.iter().enumerate() {
                if i < nid.szTip.len() {
                    nid.szTip[i] = *c;
                }
            }
            let result = Shell_NotifyIconW(NIM_ADD, &nid);
            crate::log::log(&format!("Shell_NotifyIconW NIM_ADD result: {:?}", result));
        }
    }
    
    fn remove_icon(hwnd: HWND) {
        unsafe {
            let mut nid = NOTIFYICONDATAW::default();
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = TRAY_ID;
            let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
        }
    }
    
    unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match msg {
            WM_TRAY_EVENT => {
                Self::handle_tray_event(lparam.0 as u32);
                LRESULT(0)
            }
            WM_TRAY_UPDATE => {
                do_update_icon(wparam.0 != 0);
                LRESULT(0)
            }
            WM_TRAY_SHOW => {
                do_show_icon();
                LRESULT(0)
            }
            WM_TRAY_HIDE => {
                do_hide_icon();
                LRESULT(0)
            }
            WM_COMMAND => {
                Self::handle_menu_command(wparam.0 as u32);
                LRESULT(0)
            }
            WM_SETTINGCHANGE => {
                Self::refresh_icon_for_theme_change();
                LRESULT(0)
            }
            WM_DESTROY => {
                Self::remove_icon(hwnd);
                TRAY_HWND = None;
                if let Some(menu) = TRAY_MENU {
                    let _ = DestroyMenu(menu);
                    TRAY_MENU = None;
                }
                TRAY_ON_ACTION = None;
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }
    
    fn refresh_icon_for_theme_change() {
        unsafe {
            if let Some(hwnd) = TRAY_HWND {
                let is_ascii = TRAY_IS_ASCII;
                let hicon = load_icon_from_ico(get_icon_for_mode(is_ascii));
                let tip_str = if is_ascii { "EN\0" } else { "中\0" };
                let tip: Vec<u16> = tip_str.encode_utf16().collect();
                let mut nid = NOTIFYICONDATAW::default();
                nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
                nid.hWnd = hwnd;
                nid.uID = TRAY_ID;
                nid.uFlags = NIF_ICON | NIF_TIP;
                nid.hIcon = hicon;
                for (i, c) in tip.iter().enumerate() {
                    if i < nid.szTip.len() {
                        nid.szTip[i] = *c;
                    }
                }
                let _ = Shell_NotifyIconW(NIM_MODIFY, &nid);
            }
        }
    }
    
    fn handle_tray_event(event: u32) {
        match event {
            WM_LBUTTONUP | WM_RBUTTONUP => {
                Self::handle_menu_command(MENU_ID_TOGGLE);
            }
            _ => {}
        }
    }
    
    fn handle_menu_command(cmd: u32) {
        unsafe {
            if let Some(ref on_action) = TRAY_ON_ACTION {
                match cmd {
                    MENU_ID_TOGGLE => on_action(TrayAction::ToggleAsciiMode),
                    MENU_ID_SETTINGS => on_action(TrayAction::OpenSettings),
                    MENU_ID_QUIT => on_action(TrayAction::Quit),
                    _ => {}
                }
            }
        }
    }
}

/// 更新托盘图标状态（中/英）
pub fn update_tray_icon(is_ascii: bool) {
    unsafe {
        TRAY_IS_ASCII = is_ascii;
        if let Some(hwnd) = TRAY_HWND {
            let _ = PostMessageW(Some(hwnd), WM_TRAY_UPDATE, WPARAM(if is_ascii { 1 } else { 0 }), LPARAM(0));
        }
    }
}

fn do_update_icon(is_ascii: bool) {
    unsafe {
        if let Some(hwnd) = TRAY_HWND {
            let hicon = load_icon_from_ico(get_icon_for_mode(is_ascii));
            let tip_str = if is_ascii { "EN\0" } else { "中\0" };
            let tip: Vec<u16> = tip_str.encode_utf16().collect();
            let mut nid = NOTIFYICONDATAW::default();
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = TRAY_ID;
            nid.uFlags = NIF_ICON | NIF_TIP;
            nid.hIcon = hicon;
            for (i, c) in tip.iter().enumerate() {
                if i < nid.szTip.len() {
                    nid.szTip[i] = *c;
                }
            }
            let _ = Shell_NotifyIconW(NIM_MODIFY, &nid);
        }
    }
}

/// 显示托盘图标（输入法激活时）
pub fn show_icon() {
    unsafe {
        if let Some(hwnd) = TRAY_HWND {
            let _ = PostMessageW(Some(hwnd), WM_TRAY_SHOW, WPARAM(0), LPARAM(0));
        }
    }
}

fn do_show_icon() {
    unsafe {
        if let Some(hwnd) = TRAY_HWND {
            let mut nid = NOTIFYICONDATAW::default();
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = TRAY_ID;
            nid.uFlags = NIF_STATE;
            nid.dwState = NOTIFY_ICON_STATE(0);
            nid.dwStateMask = NIS_HIDDEN;
            let _ = Shell_NotifyIconW(NIM_MODIFY, &nid);
        }
    }
}

/// 隐藏托盘图标（切换到其他输入法时）
pub fn hide_icon() {
    unsafe {
        if let Some(hwnd) = TRAY_HWND {
            let _ = PostMessageW(Some(hwnd), WM_TRAY_HIDE, WPARAM(0), LPARAM(0));
        }
    }
}

fn do_hide_icon() {
    unsafe {
        if let Some(hwnd) = TRAY_HWND {
            let mut nid = NOTIFYICONDATAW::default();
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = TRAY_ID;
            nid.uFlags = NIF_STATE;
            nid.dwState = NIS_HIDDEN;
            nid.dwStateMask = NIS_HIDDEN;
            let _ = Shell_NotifyIconW(NIM_MODIFY, &nid);
        }
    }
}

fn load_icon_from_ico(ico_data: &[u8]) -> HICON {
    unsafe {
        if ico_data.len() < 22 {
            return LoadIconW(None, IDI_APPLICATION).unwrap_or_default();
        }
        
        let image_size = u32::from_le_bytes([
            ico_data[14], ico_data[15], ico_data[16], ico_data[17],
        ]) as usize;
        
        let image_offset = u32::from_le_bytes([
            ico_data[18], ico_data[19], ico_data[20], ico_data[21],
        ]) as usize;
        
        if ico_data.len() < image_offset + image_size {
            return LoadIconW(None, IDI_APPLICATION).unwrap_or_default();
        }
        
        let icon_data = &ico_data[image_offset..image_offset + image_size];
        
        CreateIconFromResource(icon_data, true, 0x00030000)
            .unwrap_or_else(|_| LoadIconW(None, IDI_APPLICATION).unwrap_or_default())
    }
}

fn is_system_light_theme() -> bool {
    unsafe {
        let subkey: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0".encode_utf16().collect();
        let value: Vec<u16> = "SystemUsesLightTheme\0".encode_utf16().collect();
        let mut hkey = windows::Win32::System::Registry::HKEY::default();
        
        if RegOpenKeyExW(HKEY_CURRENT_USER, windows_core::PCWSTR(subkey.as_ptr()), Some(0), KEY_READ, &mut hkey).is_ok() {
            let mut data: u32 = 1;
            let mut data_size = std::mem::size_of::<u32>() as u32;
            
            let result = RegGetValueW(
                hkey,
                windows_core::PCWSTR::null(),
                windows_core::PCWSTR(value.as_ptr()),
                RRF_RT_DWORD,
                None,
                Some(&mut data as *mut _ as *mut _),
                Some(&mut data_size),
            );
            
            let _ = RegCloseKey(hkey);
            
            if result.is_ok() {
                return data != 0;
            }
        }
        true
    }
}

fn get_icon_for_mode(is_ascii: bool) -> &'static [u8] {
    let is_light = is_system_light_theme();
    match (is_ascii, is_light) {
        (false, true) => ZH_LIGHT_ICO,
        (false, false) => ZH_DARK_ICO,
        (true, true) => EN_LIGHT_ICO,
        (true, false) => EN_DARK_ICO,
    }
}