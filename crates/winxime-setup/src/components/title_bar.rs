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
            .child(
                div()
                    .id("drag-region")
                    .flex()
                    .flex_1()
                    .h_full()
                    .window_control_area(WindowControlArea::Drag)
            )
            .child(Self::close_button())
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