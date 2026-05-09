use gpui::*;

pub struct TitleBar;

impl TitleBar {
    pub fn render(_window: &mut Window) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .h(px(40.0))
            .pl(px(12.0))
            .child(
                div()
                    .id("drag-region")
                    .flex()
                    .flex_1()
                    .h_full()
                    .items_center()
                    .gap(px(10.0))
                    .window_control_area(WindowControlArea::Drag)
                    .child(Self::logo())
                    .child(Self::title_text())
            )
            .child(Self::close_button())
    }

    fn logo() -> impl IntoElement {
        img("image/icon.png")
            .w(px(24.0))
            .h(px(24.0))
            .flex_none()
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

    fn close_button() -> impl IntoElement {
        div()
            .id("close-btn")
            .flex()
            .items_center()
            .justify_center()
            .w(px(46.0))
            .h(px(40.0))
            .text_size(px(14.0))
            .text_color(rgb(0xe0e0e0))
            .cursor_pointer()
            .hover(|style| style.bg(rgb(0xc42b1c)))
            .on_click(|_event, window, _cx| {
                window.remove_window();
            })
            .child("✕")
    }
}