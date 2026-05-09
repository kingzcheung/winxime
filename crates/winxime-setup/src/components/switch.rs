use gpui::{prelude::FluentBuilder, *};

pub struct Switch {
    checked: bool,
}

impl Switch {
    pub fn new(checked: bool) -> Self {
        Self { checked }
    }
}

impl IntoElement for Switch {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        let checked = self.checked;
        let primary = hsla(0.63, 0.65, 0.67, 1.0);
        let toggle_width = px(44.0);
        let toggle_height = px(24.0);
        let knob_size = px(18.0);
        let padding = px(3.0);

        div()
            .w(toggle_width)
            .h(toggle_height)
            .rounded(px(12.0))
            .flex()
            .items_center()
            .when(checked, |this: Div| {
                this.bg(primary)
                    .justify_end()
                    .pr(padding)
            })
            .when(!checked, |this: Div| {
                this.bg(hsla(0.0, 0.0, 0.3, 1.0))
                    .justify_start()
                    .pl(padding)
            })
            .cursor_pointer()
            .child(
                div()
                    .w(knob_size)
                    .h(knob_size)
                    .rounded(px(10.0))
                    .bg(white())
            )
    }
}