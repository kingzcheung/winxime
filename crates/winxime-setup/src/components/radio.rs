use gpui::{prelude::FluentBuilder, *};

pub struct Radio {
    checked: bool,
}

impl Radio {
    pub fn new(checked: bool) -> Self {
        Self { checked }
    }
}

impl IntoElement for Radio {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        let primary = rgb(0x8F73E2);
        let inactive = rgb(0x666666);
        let size = px(16.0);
        
        div()
            .w(size)
            .h(size)
            .rounded(size / 2.0)
            .border_1()
            .border_color(if self.checked { primary } else { inactive })
            .flex()
            .items_center()
            .justify_center()
            .when(self.checked, |this: Div| {
                this.child(
                    div()
                        .w(px(8.0))
                        .h(px(8.0))
                        .rounded(px(4.0))
                        .bg(primary)
                )
            })
    }
}