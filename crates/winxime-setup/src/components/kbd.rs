use gpui::*;

pub struct Kbd {
    key: String,
}

impl Kbd {
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into() }
    }
}

impl IntoElement for Kbd {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
            .py(px(2.0))
            .px(px(6.0))
            .rounded(px(4.0))
            .bg(rgb(0x2d2d2d))
            .border_1()
            .border_color(rgb(0x4a4a4a))
            .text_size(px(12.0))
            .text_color(rgb(0xe0e0e0))
            .font_weight(FontWeight::MEDIUM)
            .child(self.key)
    }
}