use std::cell::RefCell;
use std::sync::Arc;

use windows::Win32::{
    Foundation::*,
    Graphics::{
        Direct2D::Common::{D2D1_COLOR_F, D2D_RECT_F, D2D1_PIXEL_FORMAT, D2D1_ALPHA_MODE_PREMULTIPLIED, D2D_SIZE_F, D2D1_COMPOSITE_MODE_SOURCE_OVER},
        Direct2D::{D2D1_BITMAP_OPTIONS_TARGET, D2D1_BITMAP_OPTIONS_CANNOT_DRAW, D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_ROUNDED_RECT, D2D1CreateFactory, ID2D1DeviceContext, ID2D1Factory1, D2D1_BITMAP_PROPERTIES1, D2D1_DEVICE_CONTEXT_OPTIONS_NONE, D2D1_COMPATIBLE_RENDER_TARGET_OPTIONS_NONE, D2D1_INTERPOLATION_MODE_LINEAR, CLSID_D2D1GaussianBlur, ID2D1BitmapRenderTarget},
        Direct3D::D3D_DRIVER_TYPE_WARP,
        Direct3D11::{D3D11CreateDevice, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION},
        DirectComposition::{DCompositionCreateDevice, IDCompositionDevice, IDCompositionTarget},
        DirectWrite::{
            DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL,
            DWRITE_FONT_WEIGHT_NORMAL, DWRITE_MEASURING_MODE_NATURAL, DWRITE_TEXT_METRICS,
            DWriteCreateFactory, IDWriteFactory1,
        },
        Dxgi::Common::{DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
        Dxgi::{IDXGIDevice, IDXGIFactory2, IDXGISwapChain1, DXGI_SWAP_CHAIN_DESC1, DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL, DXGI_USAGE_RENDER_TARGET_OUTPUT, DXGI_PRESENT, DXGI_SWAP_CHAIN_FLAG},
        Gdi::{BeginPaint, EndPaint, GetMonitorInfoW, MonitorFromPoint, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST, PAINTSTRUCT},
    },
    System::LibraryLoader::GetModuleHandleW,
    UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI},
    UI::WindowsAndMessaging::{
        CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, DefWindowProcW, GetWindowLongPtrW,
        LoadCursorW, PostMessageW, RegisterClassW, SetWindowLongPtrW, SetWindowPos,
        ShowWindow, IDC_ARROW, HWND_TOPMOST, SW_HIDE, SW_SHOWNA, SWP_NOACTIVATE,
        SWP_NOSIZE, SWP_NOMOVE, SWP_NOCOPYBITS, WM_DESTROY, WM_NCCREATE, WM_PAINT, WM_USER, WM_WINDOWPOSCHANGING, 
        WINDOWPOS, WNDCLASSW, CREATESTRUCTW,
        WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP, GWLP_USERDATA,
    },
};
use windows_core::{w, HSTRING, Interface, PCWSTR};
use windows_numerics::Vector2;

use winxime_ipc::Context;

pub const WM_SHOW_CANDIDATE: u32 = WM_USER + 1;
pub const WM_HIDE_CANDIDATE: u32 = WM_USER + 2;
pub const WM_UPDATE_CANDIDATE: u32 = WM_USER + 3;
pub const WM_SET_POSITION: u32 = WM_USER + 4;

const WINDOW_CLASS: &str = "WinximeCandidateWindow";
const ROW_SPACING: f32 = 4.0;
const COL_SPACING: f32 = 8.0;
const MARGIN: f32 = 6.0;
const MIN_WIDTH: f32 = 120.0;
const BLUR_RADIUS: f32 = 8.0;

#[derive(Debug, Clone)]
struct RenderedMetrics {
    width: f32,
    height: f32,
    hw_width: f32,
    hw_height: f32,
    item_height: f32,
    item_widths: Vec<f32>,
    selkey_widths: Vec<f32>,
    text_widths: Vec<f32>,
    comment_widths: Vec<f32>,
}

#[derive(Debug, Clone, Default)]
pub struct CandidateModel {
    pub items: Vec<String>,
    pub comments: Vec<String>,
    pub selkeys: Vec<u16>,
    pub total_pages: u32,
    pub current_page: u32,
    pub font_family: HSTRING,
    pub font_size: f32,
    pub cand_per_row: u32,
    pub use_cursor: bool,
    pub current_sel: usize,
    pub selkey_color: D2D1_COLOR_F,
    pub fg_color: D2D1_COLOR_F,
    pub comment_color: D2D1_COLOR_F,
    pub bg_color: D2D1_COLOR_F,
    pub highlight_fg_color: D2D1_COLOR_F,
    pub highlight_bg_color: D2D1_COLOR_F,
    pub border_color: D2D1_COLOR_F,
}

impl From<&Context> for CandidateModel {
    fn from(ctx: &Context) -> Self {
        Self {
            items: ctx.candidates.candies.iter().map(|c| c.str.clone()).collect(),
            comments: ctx.candidates.comments.iter().map(|c| c.str.clone()).collect(),
            selkeys: vec!['1' as u16, '2' as u16, '3' as u16, '4' as u16, '5' as u16],
            total_pages: ctx.candidates.total_pages,
            current_page: ctx.candidates.current_page + 1,
            current_sel: ctx.candidates.highlighted as usize,
            font_family: HSTRING::from("Microsoft YaHei UI"),
            font_size: 14.0,
            cand_per_row: 5,
            use_cursor: true,
            selkey_color: D2D1_COLOR_F { r: 0.4, g: 0.4, b: 0.4, a: 1.0 },
            fg_color: D2D1_COLOR_F { r: 0.2, g: 0.2, b: 0.2, a: 1.0 },
            comment_color: D2D1_COLOR_F { r: 0.5, g: 0.5, b: 0.5, a: 0.7 },
            bg_color: D2D1_COLOR_F { r: 0.95, g: 0.95, b: 0.95, a: 0.85 },
            highlight_fg_color: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
            highlight_bg_color: D2D1_COLOR_F { r: 0.56, g: 0.45, b: 0.89, a: 0.9 },
            border_color: D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 0.05 },
        }
    }
}

struct RenderedView {
    hwnd: HWND,
    _d2d_factory: ID2D1Factory1,
    dwrite_factory: IDWriteFactory1,
    d2d_context: ID2D1DeviceContext,
    swapchain: IDXGISwapChain1,
    _dcomp_target: IDCompositionTarget,
}

impl RenderedView {
    fn new(user_data: *const CandidateWindow) -> Result<Self, String> {
        unsafe {
            let hinstance = GetModuleHandleW(None).unwrap_or_default();
            let class_name = HSTRING::from(WINDOW_CLASS);

            let wc = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::wnd_proc),
                hInstance: HINSTANCE(hinstance.0),
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
                lpszClassName: PCWSTR::from_raw(class_name.as_ptr()),
                ..Default::default()
            };
            RegisterClassW(&wc);

            let hwnd = windows::Win32::UI::WindowsAndMessaging::CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_NOREDIRECTIONBITMAP,
                PCWSTR::from_raw(class_name.as_ptr()),
                PCWSTR::null(),
                WS_POPUP,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                200,
                160,
                None,
                None,
                Some(HINSTANCE(hinstance.0)),
                Some(user_data.cast()),
            )
            .map_err(|e| format!("CreateWindowExW failed: {:?}", e))?;

            let dwrite_factory: IDWriteFactory1 = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)
                .map_err(|e| format!("DWriteCreateFactory failed: {:?}", e))?;

            let mut device = None;
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_WARP,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                None,
            )
            .map_err(|e| format!("D3D11CreateDevice failed: {:?}", e))?;
            let device = device.ok_or("D3D11 device is None")?;

            let dxgi_device: IDXGIDevice = device
                .cast()
                .map_err(|e| format!("IDXGIDevice cast failed: {:?}", e))?;
            let adapter = dxgi_device
                .GetAdapter()
                .map_err(|e| format!("GetAdapter failed: {:?}", e))?;
            let factory: IDXGIFactory2 = adapter
                .GetParent()
                .map_err(|e| format!("GetParent failed: {:?}", e))?;

            let swapchain_desc = DXGI_SWAP_CHAIN_DESC1 {
                Width: 10,
                Height: 10,
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: 2,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
                AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
                ..Default::default()
            };

            let swapchain = factory
                .CreateSwapChainForComposition(&device, &swapchain_desc, None)
                .map_err(|e| format!("CreateSwapChainForComposition failed: {:?}", e))?;

            let d2d_factory: ID2D1Factory1 = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)
                .map_err(|e| format!("D2D1CreateFactory failed: {:?}", e))?;
            let d2d_device = d2d_factory
                .CreateDevice(&dxgi_device)
                .map_err(|e| format!("CreateDevice failed: {:?}", e))?;
            let d2d_context = d2d_device
                .CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)
                .map_err(|e| format!("CreateDeviceContext failed: {:?}", e))?;

            Self::create_swapchain_bitmap(&swapchain, &d2d_context)?;

            let dcomp_device: IDCompositionDevice = DCompositionCreateDevice(&dxgi_device)
                .map_err(|e| format!("DCompositionCreateDevice failed: {:?}", e))?;
            let dcomp_target = dcomp_device
                .CreateTargetForHwnd(hwnd, true)
                .map_err(|e| format!("CreateTargetForHwnd failed: {:?}", e))?;
            let visual = dcomp_device
                .CreateVisual()
                .map_err(|e| format!("CreateVisual failed: {:?}", e))?;
            visual
                .SetContent(&swapchain)
                .map_err(|e| format!("SetContent failed: {:?}", e))?;
            dcomp_target
                .SetRoot(&visual)
                .map_err(|e| format!("SetRoot failed: {:?}", e))?;
            dcomp_device
                .Commit()
                .map_err(|e| format!("Commit failed: {:?}", e))?;

            Ok(Self {
                hwnd,
                _d2d_factory: d2d_factory,
                dwrite_factory,
                d2d_context,
                swapchain,
                _dcomp_target: dcomp_target,
            })
        }
    }

    unsafe fn create_swapchain_bitmap(swapchain: &IDXGISwapChain1, target: &ID2D1DeviceContext) -> Result<(), String> {
        let surface: windows::Win32::Graphics::Dxgi::IDXGISurface =
            swapchain.GetBuffer(0).map_err(|e| format!("GetBuffer failed: {:?}", e))?;

        let bitmap_props = D2D1_BITMAP_PROPERTIES1 {
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_B8G8R8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
            },
            bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
            ..Default::default()
        };

        let bitmap = target
            .CreateBitmapFromDxgiSurface(&surface, Some(&bitmap_props))
            .map_err(|e| format!("CreateBitmapFromDxgiSurface failed: {:?}", e))?;
        target.SetTarget(&bitmap);
        Ok(())
    }

    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_NCCREATE => {
                let cs = lparam.0 as *const CREATESTRUCTW;
                if !cs.is_null() {
                    let user_data = unsafe { (*cs).lpCreateParams };
                    unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, user_data as isize) };
                }
                unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
            }
            WM_SHOW_CANDIDATE => {
                println!("WM_SHOW_CANDIDATE received, hwnd={:?}", hwnd.0);
                let result = ShowWindow(hwnd, SW_SHOWNA);
                println!("  ShowWindow(SW_SHOWNA) result: {:?}", result);
                LRESULT(0)
            }
            WM_HIDE_CANDIDATE => {
                println!("WM_HIDE_CANDIDATE received");
                let _ = ShowWindow(hwnd, SW_HIDE);
                LRESULT(0)
            }
            WM_UPDATE_CANDIDATE => {
                println!("WM_UPDATE_CANDIDATE received");
                let ctx_ptr = wparam.0 as *mut Context;
                if !ctx_ptr.is_null() {
                    let ctx = Box::from_raw(ctx_ptr);
                    println!("  ctx.candidates: {} items", ctx.candidates.candies.len());
                    let model = CandidateModel::from(&*ctx);
                    println!("  model.items: {:?}", model.items);
                    
                    let this_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const CandidateWindow;
                    if !this_ptr.is_null() {
                        (*this_ptr).model.replace(model.clone());
                        
                        let view = (*this_ptr).view.borrow();
                        if let Some(view) = view.as_ref() {
                            let dpi = RenderedView::get_dpi_for_window(hwnd);
                            println!("  DPI: {}", dpi);
                            if let Ok(metrics) = view.calculate_client_rect(&model, dpi) {
                                println!("  metrics.width: {}, metrics.height: {}", metrics.width, metrics.height);
                                println!("  metrics.hw_width: {}, metrics.hw_height: {}", metrics.hw_width, metrics.hw_height);
                                println!("  metrics.item_widths: {:?}", metrics.item_widths);
                                let _ = SetWindowPos(
                                    hwnd,
                                    Some(HWND_TOPMOST),
                                    0,
                                    0,
                                    metrics.hw_width as i32,
                                    metrics.hw_height as i32,
                                    SWP_NOMOVE | SWP_NOACTIVATE | SWP_NOCOPYBITS,
                                );
                                if !model.items.is_empty() {
                                    let _ = view.on_paint_with_metrics(&model, dpi, &metrics);
                                }
                            }
                        }
                    }
                    println!("  update complete");
                }
                LRESULT(0)
            }
            WM_SET_POSITION => {
                let x = wparam.0 as i32;
                let y = lparam.0 as i32;
                let _ = SetWindowPos(hwnd, Some(HWND_TOPMOST), x, y + 24, 0, 0, SWP_NOSIZE | SWP_NOACTIVATE);
                LRESULT(0)
            }
            WM_PAINT => {
                println!("WM_PAINT received");
                let mut ps = PAINTSTRUCT::default();
                BeginPaint(hwnd, &mut ps);

                let this_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const CandidateWindow;
                if !this_ptr.is_null() {
                    let model = (*this_ptr).model.borrow();
                    println!("  painting {} items", model.items.len());
                    if !model.items.is_empty() {
                        let view = (*this_ptr).view.borrow();
                        if let Some(view) = view.as_ref() {
                            let _ = view.on_paint(&model);
                        }
                    }
                }

                let _ = EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_WINDOWPOSCHANGING => {
                let pos = lparam.0 as *mut WINDOWPOS;
                if let Some(pos) = pos.as_mut() {
                    let dpi = RenderedView::get_dpi_for_point(POINT { x: pos.x, y: pos.y });
                    
                    let this_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const CandidateWindow;
                    if !this_ptr.is_null() {
                        let model = (*this_ptr).model.borrow();
                        let view = (*this_ptr).view.borrow();
                        if let Some(view) = view.as_ref() {
                            if let Ok(metrics) = view.calculate_client_rect(&model, dpi) {
                                pos.cx = metrics.hw_width as i32;
                                pos.cy = metrics.hw_height as i32;
                                (pos.x, pos.y) = RenderedView::clamp_point_to_monitor(pos.x, pos.y, pos.cx, pos.cy);
                            }
                        }
                    }
                }
                LRESULT(0)
            }
            WM_DESTROY => {
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    fn get_dpi_for_window(hwnd: HWND) -> f32 {
        unsafe {
            let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
            let mut dpi_x = 96u32;
            let mut dpi_y = 96u32;
            let _ = GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);
            dpi_x as f32
        }
    }

    fn get_dpi_for_point(point: POINT) -> f32 {
        unsafe {
            let monitor = MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST);
            let mut dpi_x = 96u32;
            let mut dpi_y = 96u32;
            let _ = GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);
            dpi_x as f32
        }
    }

    fn clamp_point_to_monitor(x: i32, y: i32, w: i32, h: i32) -> (i32, i32) {
        unsafe {
            let monitor = MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONEAREST);
            let mut mi = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };

            if GetMonitorInfoW(monitor, &mut mi).as_bool() {
                let rc = mi.rcWork;
                (x.clamp(rc.left, rc.right - w), y.clamp(rc.top, rc.bottom - h))
            } else {
                (x, y)
            }
        }
    }

    fn calculate_client_rect(&self, model: &CandidateModel, dpi: f32) -> Result<RenderedMetrics, String> {
        unsafe {
            let scale = dpi / 96.0;

            let text_format = self.dwrite_factory
                .CreateTextFormat(
                    &model.font_family,
                    None,
                    DWRITE_FONT_WEIGHT_NORMAL,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    model.font_size,
                    w!("zh-CN"),
                )
                .map_err(|e| format!("CreateTextFormat failed: {:?}", e))?;

            let mut item_height = 0.0f32;
            let mut item_widths = Vec::new();
            let mut selkey_widths = Vec::new();
            let mut text_widths = Vec::new();
            let mut comment_widths = Vec::new();
            let mut selkey_buf = "?.".encode_utf16().collect::<Vec<_>>();

            for (i, item) in model.items.iter().enumerate() {
                let selkey = model.selkeys.get(i).copied().unwrap_or('?' as u16);
                selkey_buf[0] = selkey;

                let mut selkey_metrics = DWRITE_TEXT_METRICS::default();
                let mut item_metrics = DWRITE_TEXT_METRICS::default();
                let mut comment_metrics = DWRITE_TEXT_METRICS::default();

                self.dwrite_factory
                    .CreateTextLayout(&selkey_buf, &text_format, f32::MAX, f32::MAX)
                    .map_err(|e| format!("CreateTextLayout for selkey failed: {:?}", e))?
                    .GetMetrics(&mut selkey_metrics)
                    .map_err(|e| format!("GetMetrics for selkey failed: {:?}", e))?;

                let item_hstring = HSTRING::from(item);
                self.dwrite_factory
                    .CreateTextLayout(&item_hstring, &text_format, f32::MAX, f32::MAX)
                    .map_err(|e| format!("CreateTextLayout for item failed: {:?}", e))?
                    .GetMetrics(&mut item_metrics)
                    .map_err(|e| format!("GetMetrics for item failed: {:?}", e))?;

                let comment = model.comments.get(i).cloned().unwrap_or_default();
                let comment_hstring = HSTRING::from(&comment);
                self.dwrite_factory
                    .CreateTextLayout(&comment_hstring, &text_format, f32::MAX, f32::MAX)
                    .map_err(|e| format!("CreateTextLayout for comment failed: {:?}", e))?
                    .GetMetrics(&mut comment_metrics)
                    .map_err(|e| format!("GetMetrics for comment failed: {:?}", e))?;

                let padding_x = 6.0;
                let padding_y = 4.0;
                let selkey_width = selkey_metrics.widthIncludingTrailingWhitespace;
                let text_width = item_metrics.widthIncludingTrailingWhitespace;
                let comment_width = if comment.is_empty() { 0.0 } else { comment_metrics.widthIncludingTrailingWhitespace + 4.0 };
                selkey_widths.push(selkey_width);
                text_widths.push(text_width);
                comment_widths.push(comment_width);

                let item_width = selkey_width + text_width + comment_width + 2.0 * padding_x;
                item_widths.push(item_width);
                item_height = item_height.max(item_metrics.height + 2.0 * padding_y).max(selkey_metrics.height + 2.0 * padding_y);
            }

            let items_len = model.items.len() as f32;
            if items_len == 0.0 {
                return Ok(RenderedMetrics {
                    width: 100.0,
                    height: 30.0,
                    hw_width: ((100.0 + BLUR_RADIUS * 2.0) * scale).ceil(),
                    hw_height: ((30.0 + BLUR_RADIUS * 2.0) * scale).ceil(),
                    item_height: 20.0,
                    item_widths: Vec::new(),
                    selkey_widths: Vec::new(),
                    text_widths: Vec::new(),
                    comment_widths: Vec::new(),
                });
            }

            let cand_per_row = model.cand_per_row as usize;
            
            let mut max_row_width = MIN_WIDTH;
            for row_start in (0..model.items.len()).step_by(cand_per_row) {
                let row_end = std::cmp::min(row_start + cand_per_row, model.items.len());
                let row_width: f32 = item_widths[row_start..row_end].iter().copied().sum::<f32>()
                    + (row_end - row_start - 1) as f32 * COL_SPACING
                    + 2.0 * MARGIN;
                max_row_width = max_row_width.max(row_width);
            }

            let rows = (items_len / cand_per_row as f32).ceil().max(1.0);
            let height = rows * item_height + (rows - 1.0) * ROW_SPACING + 2.0 * MARGIN;

            let hw_width = ((max_row_width + BLUR_RADIUS * 2.0) * scale).ceil();
            let hw_height = ((height + BLUR_RADIUS * 2.0) * scale).ceil();

            Ok(RenderedMetrics {
                width: max_row_width,
                height,
                hw_width,
                hw_height,
                item_height,
                item_widths,
                selkey_widths,
                text_widths,
                comment_widths,
            })
        }
    }

    fn on_paint(&self, model: &CandidateModel) -> Result<(), String> {
        let dpi = Self::get_dpi_for_window(self.hwnd);
        let metrics = self.calculate_client_rect(model, dpi)?;
        self.on_paint_with_metrics(model, dpi, &metrics)
    }

    fn on_paint_with_metrics(&self, model: &CandidateModel, dpi: f32, metrics: &RenderedMetrics) -> Result<(), String> {
        unsafe {
            println!("on_paint: dpi={}, width={}, height={}, hw_width={}, hw_height={}", 
                dpi, metrics.width, metrics.height, metrics.hw_width, metrics.hw_height);
            println!("on_paint: item_widths={:?}", metrics.item_widths);

            self.d2d_context.SetTarget(None);
            self.swapchain
                .ResizeBuffers(
                    0,
                    metrics.hw_width as u32,
                    metrics.hw_height as u32,
                    DXGI_FORMAT_B8G8R8A8_UNORM,
                    DXGI_SWAP_CHAIN_FLAG(0),
                )
                .map_err(|e| format!("ResizeBuffers failed: {:?}", e))?;

            self.d2d_context.SetDpi(dpi, dpi);
            Self::create_swapchain_bitmap(&self.swapchain, &self.d2d_context)?;

            let text_format = self.dwrite_factory
                .CreateTextFormat(
                    &model.font_family,
                    None,
                    DWRITE_FONT_WEIGHT_NORMAL,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    model.font_size,
                    w!("zh-CN"),
                )
                .map_err(|e| format!("CreateTextFormat failed: {:?}", e))?;

            self.d2d_context.BeginDraw();

            let blur_radius = BLUR_RADIUS;
            let corner_radius = 10.0;

            let bg_brush = self.d2d_context
                .CreateSolidColorBrush(&model.bg_color, None)
                .map_err(|e| format!("CreateSolidColorBrush bg failed: {:?}", e))?;

            let border_brush = self.d2d_context
                .CreateSolidColorBrush(&model.border_color, None)
                .map_err(|e| format!("CreateSolidColorBrush border failed: {:?}", e))?;

            let selkey_brush = self.d2d_context
                .CreateSolidColorBrush(&model.selkey_color, None)
                .map_err(|e| format!("CreateSolidColorBrush selkey failed: {:?}", e))?;

            let text_brush = self.d2d_context
                .CreateSolidColorBrush(&model.fg_color, None)
                .map_err(|e| format!("CreateSolidColorBrush text failed: {:?}", e))?;

            let highlight_brush = self.d2d_context
                .CreateSolidColorBrush(&model.highlight_bg_color, None)
                .map_err(|e| format!("CreateSolidColorBrush highlight failed: {:?}", e))?;

            let selected_text_brush = self.d2d_context
                .CreateSolidColorBrush(&model.highlight_fg_color, None)
                .map_err(|e| format!("CreateSolidColorBrush selected_text failed: {:?}", e))?;

            let shadow_render_target: ID2D1BitmapRenderTarget = self.d2d_context
                .CreateCompatibleRenderTarget(
                    Some(&D2D_SIZE_F {
                        width: metrics.width + blur_radius * 2.0,
                        height: metrics.height + blur_radius * 2.0,
                    }),
                    None,
                    None,
                    D2D1_COMPATIBLE_RENDER_TARGET_OPTIONS_NONE,
                )
                .map_err(|e| format!("CreateCompatibleRenderTarget failed: {:?}", e))?;

            shadow_render_target.BeginDraw();
            shadow_render_target.Clear(None);

            let shadow_brush = shadow_render_target
                .CreateSolidColorBrush(
                    &D2D1_COLOR_F {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.15,
                    },
                    None,
                )
                .map_err(|e| format!("CreateSolidColorBrush shadow failed: {:?}", e))?;

            let shadow_rect = D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F {
                    left: blur_radius,
                    top: blur_radius,
                    right: metrics.width + blur_radius,
                    bottom: metrics.height + blur_radius,
                },
                radiusX: corner_radius,
                radiusY: corner_radius,
            };
            shadow_render_target.FillRoundedRectangle(&shadow_rect, &shadow_brush);
            shadow_render_target.EndDraw(None, None)
                .map_err(|e| format!("shadow EndDraw failed: {:?}", e))?;

            let shadow_bitmap = shadow_render_target
                .GetBitmap()
                .map_err(|e| format!("GetBitmap failed: {:?}", e))?;

            let gaussian_blur_effect = self.d2d_context
                .CreateEffect(&CLSID_D2D1GaussianBlur)
                .map_err(|e| format!("CreateEffect failed: {:?}", e))?;
            gaussian_blur_effect.SetInput(0, &shadow_bitmap, false);
            let blur_output = gaussian_blur_effect
                .GetOutput()
                .map_err(|e| format!("GetOutput failed: {:?}", e))?;

            self.d2d_context.DrawImage(
                &blur_output,
                Some(&Vector2 { X: 0.0, Y: 0.0 }),
                None,
                D2D1_INTERPOLATION_MODE_LINEAR,
                D2D1_COMPOSITE_MODE_SOURCE_OVER,
            );

            let bg_rounded_rect = D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F {
                    left: blur_radius,
                    top: blur_radius,
                    right: metrics.width + blur_radius,
                    bottom: metrics.height + blur_radius,
                },
                radiusX: corner_radius,
                radiusY: corner_radius,
            };
            self.d2d_context.FillRoundedRectangle(&bg_rounded_rect, &bg_brush);

            let border_rounded_rect = D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F {
                    left: blur_radius + 0.5,
                    top: blur_radius + 0.5,
                    right: metrics.width + blur_radius - 0.5,
                    bottom: metrics.height + blur_radius - 0.5,
                },
                radiusX: corner_radius,
                radiusY: corner_radius,
            };
            self.d2d_context.DrawRoundedRectangle(&border_rounded_rect, &border_brush, 0.5, None);

let comment_brush = self.d2d_context
                .CreateSolidColorBrush(&model.comment_color, None)
                .map_err(|e| format!("CreateSolidColorBrush comment failed: {:?}", e))?;

            let mut col = 0usize;
            let mut x = MARGIN + blur_radius;
            let mut y = MARGIN + blur_radius;
            let padding_x = 6.0;
            let padding_y = 4.0;

            for (i, item) in model.items.iter().enumerate() {
                let selkey = model.selkeys.get(i).copied().unwrap_or('?' as u16);
                let mut selkey_buf = [0u16; 3];
                selkey_buf[0] = selkey;
                selkey_buf[1] = '.' as u16;

                let item_width = metrics.item_widths.get(i).copied().unwrap_or(60.0);
                let selkey_width = metrics.selkey_widths.get(i).copied().unwrap_or(20.0);
                let text_width = metrics.text_widths.get(i).copied().unwrap_or(40.0);
                let comment_width = metrics.comment_widths.get(i).copied().unwrap_or(0.0);
                
                println!("  item {}: x={}, item_width={}, text='{}'", i, x, item_width, item);
                println!("  bg_rect: left={}, right={}", blur_radius, metrics.width + blur_radius);
                println!("  item_right={}", x + item_width);

                let selkey_rect = D2D_RECT_F {
                    left: x + padding_x,
                    top: y + padding_y,
                    right: x + selkey_width + padding_x,
                    bottom: y + metrics.item_height - padding_y,
                };

                let text_rect = D2D_RECT_F {
                    left: x + selkey_width + padding_x,
                    top: y + padding_y,
                    right: x + selkey_width + text_width + padding_x,
                    bottom: y + metrics.item_height - padding_y,
                };

                let comment_rect = D2D_RECT_F {
                    left: x + selkey_width + text_width + padding_x + 4.0,
                    top: y + padding_y,
                    right: x + item_width - padding_x,
                    bottom: y + metrics.item_height - padding_y,
                };

                let item_hstring = HSTRING::from(item);
                let comment = model.comments.get(i).cloned().unwrap_or_default();
                let comment_hstring = HSTRING::from(&comment);

                if model.use_cursor && i == model.current_sel {
                    let highlight_rounded_rect = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: x,
                            top: y,
                            right: x + item_width,
                            bottom: y + metrics.item_height,
                        },
                        radiusX: 6.0,
                        radiusY: 6.0,
                    };
                    self.d2d_context.FillRoundedRectangle(&highlight_rounded_rect, &highlight_brush);

                    self.d2d_context.DrawText(
                        &selkey_buf[..2],
                        &text_format,
                        &selkey_rect,
                        &selected_text_brush,
                        D2D1_DRAW_TEXT_OPTIONS_NONE,
                        DWRITE_MEASURING_MODE_NATURAL,
                    );

                    self.d2d_context.DrawText(
                        &item_hstring,
                        &text_format,
                        &text_rect,
                        &selected_text_brush,
                        D2D1_DRAW_TEXT_OPTIONS_NONE,
                        DWRITE_MEASURING_MODE_NATURAL,
                    );

                    if !comment.is_empty() {
                        self.d2d_context.DrawText(
                            &comment_hstring,
                            &text_format,
                            &comment_rect,
                            &selected_text_brush,
                            D2D1_DRAW_TEXT_OPTIONS_NONE,
                            DWRITE_MEASURING_MODE_NATURAL,
                        );
                    }
                } else {
                    self.d2d_context.DrawText(
                        &selkey_buf[..2],
                        &text_format,
                        &selkey_rect,
                        &selkey_brush,
                        D2D1_DRAW_TEXT_OPTIONS_NONE,
                        DWRITE_MEASURING_MODE_NATURAL,
                    );

                    self.d2d_context.DrawText(
                        &item_hstring,
                        &text_format,
                        &text_rect,
                        &text_brush,
                        D2D1_DRAW_TEXT_OPTIONS_NONE,
                        DWRITE_MEASURING_MODE_NATURAL,
                    );

                    if !comment.is_empty() {
                        self.d2d_context.DrawText(
                            &comment_hstring,
                            &text_format,
                            &comment_rect,
                            &comment_brush,
                            D2D1_DRAW_TEXT_OPTIONS_NONE,
                            DWRITE_MEASURING_MODE_NATURAL,
                        );
                    }
                }

                col += 1;
                if col >= model.cand_per_row as usize {
                    col = 0;
                    x = MARGIN + blur_radius;
                    y += metrics.item_height + ROW_SPACING;
                } else {
                    x += item_width + COL_SPACING;
                }
            }

            self.d2d_context.EndDraw(None, None)
                .map_err(|e| format!("EndDraw failed: {:?}", e))?;

            let _ = self.swapchain.Present(1, DXGI_PRESENT(0)).ok();
        }

        Ok(())
    }
}

pub struct CandidateWindow {
    model: RefCell<CandidateModel>,
    view: RefCell<Option<RenderedView>>,
}

unsafe impl Send for CandidateWindow {}
unsafe impl Sync for CandidateWindow {}

impl CandidateWindow {
    pub fn new() -> Arc<Self> {
        let window = Arc::new(Self {
            model: RefCell::new(CandidateModel::default()),
            view: RefCell::new(None),
        });
        
        // Initialize UI immediately in the thread that will run message loop
        window.ensure_view_initialized();
        
        window
    }

    fn ensure_view_initialized(&self) {
        if self.view.borrow().is_none() {
            let user_data_ptr = self as *const Self;
            match RenderedView::new(user_data_ptr.cast()) {
                Ok(view) => {
                    println!("UI initialized successfully");
                    *self.view.borrow_mut() = Some(view);
                }
                Err(e) => {
                    eprintln!("Failed to initialize UI: {}", e);
                }
            }
        }
    }

    pub fn show(&self, x: i32, y: i32) {
        self.ensure_view_initialized();
        if let Some(view) = self.view.borrow().as_ref() {
            println!("  show: hwnd={:?}, moving to ({}, {})", view.hwnd.0, x, y + 24);
            unsafe {
                let _ = SetWindowPos(view.hwnd, Some(HWND_TOPMOST), x, y + 24, 0, 0, SWP_NOSIZE | SWP_NOACTIVATE);
                println!("  show: posting WM_SHOW_CANDIDATE");
                let result = PostMessageW(Some(view.hwnd), WM_SHOW_CANDIDATE, WPARAM(0), LPARAM(0));
                println!("  show: PostMessageW result: {:?}", result);
            }
        } else {
            println!("  show: view is None!");
        }
    }

    pub fn hide(&self) {
        if let Some(view) = self.view.borrow().as_ref() {
            unsafe {
                let _ = PostMessageW(Some(view.hwnd), WM_HIDE_CANDIDATE, WPARAM(0), LPARAM(0));
            }
        }
    }

    pub fn update(&self, ctx: &Context) {
        if let Some(view) = self.view.borrow().as_ref() {
            println!("  update: hwnd={:?}, posting WM_UPDATE_CANDIDATE", view.hwnd.0);
            unsafe {
                let ctx_ptr = Box::into_raw(Box::new(ctx.clone()));
                let result = PostMessageW(
                    Some(view.hwnd),
                    WM_UPDATE_CANDIDATE,
                    WPARAM(ctx_ptr as usize),
                    LPARAM(0),
                );
                println!("  update: PostMessageW result: {:?}", result);
            }
        } else {
            println!("  update: view is None!");
        }
    }
}