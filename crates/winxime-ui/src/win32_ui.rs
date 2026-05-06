#![allow(unused_must_use, dead_code)]
use std::sync::Arc;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;
use winxime_core::SharedInputContext;

const WINDOW_CLASS: &str = "XimeCandidateWindow";

#[derive(Clone, Copy)]
struct SendHwnd(HWND);
unsafe impl Send for SendHwnd {}

pub struct CandidateWindowInner {
    hwnd: SendHwnd,
    thread_handle: Option<std::thread::JoinHandle<()>>,
    context: &'static SharedInputContext,
}

impl CandidateWindowInner {
    pub fn start(context: &'static SharedInputContext) -> Arc<Self> {
        let (tx, rx) = std::sync::mpsc::channel::<SendHwnd>();

        let handle = std::thread::Builder::new()
            .name("XimeUI".into())
            .spawn(move || {
                Self::ui_thread(tx);
            })
            .expect("Failed to start UI thread");

        let hwnd = rx.recv().expect("Failed to get HWND from UI thread");

        Arc::new(Self {
            hwnd,
            thread_handle: Some(handle),
            context,
        })
    }

    fn ui_thread(tx: std::sync::mpsc::Sender<SendHwnd>) {
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
                WINDOW_EX_STYLE(
                    WS_EX_TOOLWINDOW.0 | WS_EX_TOPMOST.0 | WS_EX_NOACTIVATE.0 | WS_EX_LAYERED.0,
                ),
                windows::core::PCWSTR::from_raw(class_name.as_ptr()),
                windows::core::PCWSTR::null(),
                WS_POPUP,
                0, 0, 200, 140,
                None,
                None,
                hinstance,
                None,
            );

            let hwnd = hwnd.unwrap_or_default();
            let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0xFFFFFF), 240, LWA_ALPHA);
            tx.send(SendHwnd(hwnd)).ok();

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_PAINT => {
                Self::on_paint(hwnd);
                LRESULT(0)
            }
            WM_ERASEBKGND => LRESULT(1),
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    unsafe fn on_paint(hwnd: HWND) {
        let mut ps = PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &mut ps);
        let mut rect = RECT::default();
        let _ = GetClientRect(hwnd, &mut rect);
        let (w, h) = (rect.right - rect.left, rect.bottom - rect.top);

        if w == 0 || h == 0 {
            let _ = EndPaint(hwnd, &ps);
            return;
        }

        let bg = CreateSolidBrush(COLORREF(0xF0F0F0));
        FillRect(hdc, &rect, bg);
        DeleteObject(bg);

        let border = CreateSolidBrush(COLORREF(0xCCCCCC));
        let f = RECT { left: 0, top: 0, right: w, bottom: h };
        FrameRect(hdc, &f, border);
        DeleteObject(border);

        // Title
        win32_text(hdc, "Xime", 8, 4, w - 8, 5 + 18, 0x333333, true);

        // Candidate placeholder items
        let items: [(u32, &str, bool); 5] = [
            (0x0066CC, "1. \u{4e94}\u{7b14}", true),   // 五笔
            (0x333333, "2. \u{5019}\u{9009}", false),  // 候选
            (0x333333, "3. \u{6d4b}\u{8bd5}", false),  // 测试
            (0x333333, "4. \u{8f93}\u{5165}", false),  // 输入
            (0x333333, "5. \u{6cd5}\u{5f00}", false),  // 法开
        ];

        for (i, (color, label, selected)) in items.iter().enumerate() {
            let y: i32 = 28 + (i as i32) * 22;

            if *selected {
                let hl = CreateSolidBrush(COLORREF(0xE8F0FE));
                let s = RECT { left: 2, top: y - 1, right: w - 2, bottom: y + 21 };
                FillRect(hdc, &s, hl);
                DeleteObject(hl);
            }

            win32_text(hdc, label, 8, y, w - 8, y + 20, *color, false);
        }

        EndPaint(hwnd, &ps);
    }
}

fn win32_text(hdc: HDC, text: &str, x: i32, y: i32, right: i32, bottom: i32, color: u32, bold: bool) -> i32 {
    unsafe {
        let mut buf: Vec<u16> = text.encode_utf16().collect();
        let mut rect = RECT { left: x, top: y, right, bottom };

        SetTextColor(hdc, COLORREF(color));
        SetBkMode(hdc, TRANSPARENT);

        let weight = if bold { 700i32 } else { 400i32 };
        let font = CreateFontW(-13, 0, 0, 0, weight, 0, 0, 0, 1, 0, 0, 0, 0, windows::core::w!("Microsoft YaHei UI"));
        let old = SelectObject(hdc, font);
        let r = DrawTextW(hdc, &mut buf, &mut rect, DT_LEFT | DT_TOP | DT_SINGLELINE);
        SelectObject(hdc, old);
        DeleteObject(font);
        r
    }
}

impl CandidateWindowInner {
    pub fn show(&self, x: i32, y: i32) {
        unsafe {
            let hwnd = self.hwnd.0;
            let mut rect = RECT::default();
            GetClientRect(hwnd, &mut rect);
            let w = if rect.right == 0 { 200 } else { rect.right };
            let h = if rect.bottom == 0 { 140 } else { rect.bottom };

            SetWindowPos(
                hwnd, HWND_TOPMOST,
                x, y, w, h,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
        }
    }

    pub fn hide(&self) {
        unsafe { ShowWindow(self.hwnd.0, SW_HIDE); }
    }

    pub fn is_visible(&self) -> bool {
        unsafe { IsWindowVisible(self.hwnd.0).as_bool() }
    }

    pub fn request_redraw(&self) {
        unsafe { InvalidateRect(self.hwnd.0, None, false); }
    }
}
