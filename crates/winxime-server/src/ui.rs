use std::sync::Arc;
use winxime_ipc::Context;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{GWLP_USERDATA, WM_USER, *};

const WINDOW_CLASS: &str = "WinximeCandidateWindow";

pub const WM_SHOW_CANDIDATE: u32 = WM_USER + 1;
pub const WM_HIDE_CANDIDATE: u32 = WM_USER + 2;
pub const WM_UPDATE_CANDIDATE: u32 = WM_USER + 3;
pub const WM_SET_POSITION: u32 = WM_USER + 4;

pub struct CandidateWindow {
    hwnd: HWND,
}

unsafe impl Send for CandidateWindow {}
unsafe impl Sync for CandidateWindow {}

impl CandidateWindow {
    pub fn new() -> Arc<Self> {
        let hwnd = Self::create_window();
        Arc::new(Self { hwnd })
    }
    
    fn create_window() -> HWND {
        unsafe {
            let hinstance = GetModuleHandleW(None).unwrap_or_default();
            let class_name = windows::core::HSTRING::from(WINDOW_CLASS);
            
            let wc = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::wnd_proc),
                hInstance: hinstance.into(),
                lpszClassName: windows::core::PCWSTR::from_raw(class_name.as_ptr()),
                hbrBackground: HBRUSH(GetStockObject(WHITE_BRUSH).0),
                ..Default::default()
            };
            
            RegisterClassW(&wc);
            
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(WS_EX_TOOLWINDOW.0 | WS_EX_TOPMOST.0 | WS_EX_NOACTIVATE.0),
                windows::core::PCWSTR::from_raw(class_name.as_ptr()),
                windows::core::PCWSTR::null(),
                WS_POPUP | WS_BORDER,
                0, 0, 200, 160,
                None,
                None,
                hinstance,
                None,
            ).unwrap_or_default();
            
            hwnd
        }
    }
    
    pub fn show(&self, x: i32, y: i32) {
        unsafe {
            PostMessageW(self.hwnd, WM_SET_POSITION, WPARAM(x as usize), LPARAM(y as isize));
            PostMessageW(self.hwnd, WM_SHOW_CANDIDATE, WPARAM(0), LPARAM(0));
        }
    }
    
    pub fn hide(&self) {
        unsafe {
            PostMessageW(self.hwnd, WM_HIDE_CANDIDATE, WPARAM(0), LPARAM(0));
        }
    }
    
    pub fn update(&self, ctx: &Context) {
        unsafe {
            let ctx_ptr = Box::into_raw(Box::new(ctx.clone()));
            PostMessageW(self.hwnd, WM_UPDATE_CANDIDATE, WPARAM(ctx_ptr as usize), LPARAM(0));
        }
    }
    
    unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match msg {
            WM_SHOW_CANDIDATE => {
                let _ = ShowWindow(hwnd, SW_SHOWNA);
                LRESULT(0)
            }
            WM_HIDE_CANDIDATE => {
                let _ = ShowWindow(hwnd, SW_HIDE);
                LRESULT(0)
            }
            WM_UPDATE_CANDIDATE => {
                let ctx_ptr = wparam.0 as *mut Context;
                if !ctx_ptr.is_null() {
                    let ctx = Box::from_raw(ctx_ptr);
                    let old_ptr = SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(ctx) as isize);
                    if old_ptr != 0 {
                        let _ = Box::from_raw(old_ptr as *mut Context);
                    }
                    let _ = InvalidateRect(hwnd, None, true);
                    let _ = UpdateWindow(hwnd);
                }
                LRESULT(0)
            }
            WM_SET_POSITION => {
                let x = wparam.0 as i32;
                let y = lparam.0 as i32;
                let _ = SetWindowPos(hwnd, HWND_TOPMOST, x, y + 24, 200, 160, SWP_NOACTIVATE | SWP_NOCOPYBITS);
                LRESULT(0)
            }
            WM_PAINT => {
                Self::on_paint(hwnd);
                LRESULT(0)
            }
            WM_DESTROY => {
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
                if ptr != 0 {
                    let _ = Box::from_raw(ptr as *mut Context);
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                }
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
    
    unsafe fn on_paint(hwnd: HWND) {
        let mut ps = PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &mut ps);
        
        let bg = CreateSolidBrush(COLORREF(0xF0F0F0));
        let mut rect = RECT::default();
        let _ = GetClientRect(hwnd, &mut rect);
        FillRect(hdc, &rect, bg);
        let _ = DeleteObject(bg);
        
        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
        if ptr != 0 {
            let ctx = &*(ptr as *const Context);
            
            let preedit = &ctx.preedit.str;
            if !preedit.is_empty() {
                win32_text(hdc, preedit, 8, 8, 192, 28, 0x333333);
            }
            
            for (i, cand) in ctx.candidates.candies.iter().enumerate() {
                if i >= 5 { break; }
                let text = format!("{}. {}", i + 1, cand.str);
                win32_text(hdc, &text, 8, 32 + i as i32 * 24, 192, 56 + i as i32 * 24, 0x000000);
            }
        } else {
            win32_text(hdc, "曦码五笔", 8, 8, 192, 28, 0x333333);
        }
        
        let _ = EndPaint(hwnd, &ps);
    }
}

fn win32_text(hdc: HDC, text: &str, x: i32, y: i32, right: i32, bottom: i32, color: u32) {
    unsafe {
        let mut buf: Vec<u16> = text.encode_utf16().collect();
        let mut rect = RECT { left: x, top: y, right, bottom };
        
        SetTextColor(hdc, COLORREF(color));
        SetBkMode(hdc, TRANSPARENT);
        
        let font = CreateFontW(-14, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 0, 0, windows::core::w!("Microsoft YaHei UI"));
        let old = SelectObject(hdc, font);
        DrawTextW(hdc, &mut buf, &mut rect, DT_LEFT | DT_VCENTER | DT_SINGLELINE);
        SelectObject(hdc, old);
        let _ = DeleteObject(font);
    }
}