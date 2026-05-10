use gpui::prelude::FluentBuilder;
use gpui::*;
use std::sync::Arc;

pub struct NumberInput {
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    on_change: Option<Arc<dyn Fn(f64, &mut Window, &mut App) + 'static>>,
}

impl NumberInput {
    pub fn new(value: f64) -> Self {
        Self {
            value,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            on_change: None,
        }
    }
    
    pub fn min(mut self, min: f64) -> Self {
        self.min = min;
        self
    }
    
    pub fn max(mut self, max: f64) -> Self {
        self.max = max;
        self
    }
    
    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }
    
    pub fn on_change(mut self, callback: impl Fn(f64, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Arc::new(callback));
        self
    }
}

impl IntoElement for NumberInput {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        let primary = rgb(0x8F73E2);
        let value = self.value;
        let min = self.min;
        let max = self.max;
        let step = self.step;
        let on_change = self.on_change.clone();
        
        div()
            .flex()
            .items_center()
            .gap(px(4.0))
            .w(px(140.0))
            .h(px(36.0))
            .rounded(px(12.0))
            .bg(rgb(0x262626))
            .border_1()
            .border_color(rgb(0x404040))
            .child(
                div()
                    .id("dec-btn")
                    .w(px(32.0))
                    .h(px(32.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(10.0))
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(0x404040)))
                    .text_size(px(16.0))
                    .when(value > min, |this: Stateful<Div>| {
                        this.text_color(rgb(0xe0e0e0))
                    })
                    .when(value <= min, |this: Stateful<Div>| {
                        this.text_color(rgb(0x808080))
                    })
                    .when_some(on_change.clone(), |this: Stateful<Div>, cb| {
                        this.on_click(move |_, window, cx| {
                            let new_value = (value - step).max(min);
                            cb(new_value, window, cx);
                        })
                    })
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
                    .child(format!("{}", value as i32))
            )
            .child(
                div()
                    .id("inc-btn")
                    .w(px(32.0))
                    .h(px(32.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(10.0))
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(0x404040)))
                    .text_size(px(16.0))
                    .when(value < max, |this: Stateful<Div>| {
                        this.text_color(rgb(0xe0e0e0))
                    })
                    .when(value >= max, |this: Stateful<Div>| {
                        this.text_color(rgb(0x808080))
                    })
                    .when_some(on_change, |this: Stateful<Div>, cb| {
                        this.on_click(move |_, window, cx| {
                            let new_value = (value + step).min(max);
                            cb(new_value, window, cx);
                        })
                    })
                    .child("+")
            )
    }
}