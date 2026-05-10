use gpui::*;

#[derive(Clone)]
pub struct Label {
    text: String,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl IntoElement for Label {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
            .text_size(px(12.0))
            .text_color(rgb(0xc0c0c0))
            .child(self.text)
    }
}