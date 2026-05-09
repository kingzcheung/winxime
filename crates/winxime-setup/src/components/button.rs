use gpui::*;

pub struct Button {
    label: String,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into() }
    }
}

impl IntoElement for Button {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let primary = hsla(0.63, 0.65, 0.67, 1.0);
        
        div()
            .id(self.label.clone())
            .py(px(8.0))
            .px(px(16.0))
            .rounded(px(12.0))
            .bg(primary)
            .text_color(white())
            .text_size(px(14.0))
            .cursor_pointer()
            .hover(|style| style.bg(hsla(0.63, 0.65, 0.55, 1.0)))
            .child(self.label)
    }
}