use crate::theme::ThemeColors;
use gpui::prelude::FluentBuilder;
use gpui::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct Button {
    label: String,
    on_click: Option<Arc<dyn Fn(&mut Window, &mut App) + 'static>>,
    colors: Option<ThemeColors>,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            on_click: None,
            colors: None,
        }
    }

    pub fn on_click(mut self, callback: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Arc::new(callback));
        self
    }

    pub fn theme(mut self, colors: ThemeColors) -> Self {
        self.colors = Some(colors);
        self
    }
}

impl IntoElement for Button {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let colors = self.colors.unwrap_or_else(|| {
            ThemeColors::from_theme(&crate::theme::SystemTheme::Light, 0x8F73E2)
        });
        let on_click = self.on_click;

        let hover_color = hsla(
            colors.primary.h,
            colors.primary.s,
            colors.primary.l * 0.9,
            colors.primary.a,
        );

        div()
            .id(self.label.clone())
            .py(px(8.0))
            .px(px(16.0))
            .rounded(px(12.0))
            .bg(colors.primary)
            .text_color(colors.on_primary)
            .text_size(px(14.0))
            .cursor_pointer()
            .hover(|style| style.bg(hover_color))
            .when_some(on_click, |this: Stateful<Div>, cb| {
                this.on_click(move |_, window, cx| {
                    cb(window, cx);
                })
            })
            .child(self.label)
    }
}
