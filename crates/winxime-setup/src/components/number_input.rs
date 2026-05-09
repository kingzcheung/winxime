use gpui::*;

pub struct NumberInput {
    value: f64,
}

impl NumberInput {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

impl IntoElement for NumberInput {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        let primary = hsla(0.63, 0.65, 0.67, 1.0);
        
        div()
            .flex()
            .items_center()
            .gap(px(4.0))
            .w(px(140.0))
            .h(px(36.0))
            .rounded(px(12.0))
            .bg(hsla(0.0, 0.0, 0.15, 0.8))
            .border_1()
            .border_color(rgb(0x404040))
            .child(
                div()
                    .w(px(32.0))
                    .h(px(32.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(10.0))
                    .cursor_pointer()
                    .hover(|style| style.bg(hsla(0.0, 0.0, 0.25, 0.6)))
                    .text_size(px(16.0))
                    .text_color(rgb(0x808080))
                    .child("-")
            )
            .child(
                div()
                    .flex_1()
                    .h(px(32.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(14.0))
                    .text_color(primary)
                    .font_weight(FontWeight::MEDIUM)
                    .child(format!("{}", self.value as i32))
            )
            .child(
                div()
                    .w(px(32.0))
                    .h(px(32.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(10.0))
                    .cursor_pointer()
                    .hover(|style| style.bg(hsla(0.0, 0.0, 0.25, 0.6)))
                    .text_size(px(16.0))
                    .text_color(rgb(0x808080))
                    .child("+")
            )
    }
}