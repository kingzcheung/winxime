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
        let primary = rgb(0x8F73E2);
        
        div()
            .id(self.label.clone())
            .py(px(8.0))
            .px(px(16.0))
            .rounded(px(12.0))
            .bg(primary)
            .text_color(white())
            .text_size(px(14.0))
            .cursor_pointer()
            .hover(|style| style.bg(rgb(0x7A5FD0)))
            .child(self.label)
    }
}