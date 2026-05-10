use gpui::*;
use crate::theme::ThemeColors;

pub struct TitleBar;

impl TitleBar {
    pub fn render(_window: &mut Window, colors: &ThemeColors) -> impl IntoElement {
        div()
            .flex()
            .w_full()
            .h(px(40.0))
            .child(
                div()
                    .w(px(213.0))
                    .h(px(40.0))
                    .bg(rgb(0x2d1f3d))
                    .flex()
                    .items_center()
                    .pl(px(12.0))
                    .child(Self::logo())
                    .child(Self::title_text())
                    .window_control_area(WindowControlArea::Drag)
            )
            .child(
                div()
                    .id("drag-region")
                    .flex_1()
                    .h(px(40.0))
                    .bg(colors.background)
                    .window_control_area(WindowControlArea::Drag)
            )
            .child(Self::close_button(colors.foreground, colors.background))
    }

    fn logo() -> impl IntoElement {
        img("image/icon.png")
            .w(px(24.0))
            .h(px(24.0))
            .flex_none()
            .mr_2()
    }

    fn title_text() -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap(px(4.0))
            .child(
                div()
                    .text_size(px(14.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(0xe0e0e0))
                    .child("Xime")
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(0x808080))
                    .child("曦码输入法")
            )
    }

    fn close_button(text_color: Hsla, bg: Hsla) -> impl IntoElement {
        div()
            .id("close-btn")
            .flex()
            .items_center()
            .justify_center()
            .w(px(46.0))
            .h(px(40.0))
            .bg(bg)
            .text_size(px(14.0))
            .text_color(text_color)
            .cursor_pointer()
            .hover(|style| style.bg(rgb(0xc42b1c)))
            .on_click(|_event, window, _cx| {
                window.remove_window();
            })
            .child("✕")
    }
}