use crate::theme::ThemeColors;
use gpui::*;

#[derive(Clone)]
pub struct Kbd {
    key: String,
    colors: Option<ThemeColors>,
}

impl Kbd {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            colors: None,
        }
    }

    pub fn theme(mut self, colors: ThemeColors) -> Self {
        self.colors = Some(colors);
        self
    }
}

impl IntoElement for Kbd {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        let colors = self.colors.unwrap_or_else(|| {
            ThemeColors::from_theme(&crate::theme::SystemTheme::Light, 0x8F73E2)
        });

        div()
            .py(px(2.0))
            .px(px(6.0))
            .rounded(px(4.0))
            .bg(colors.surface_variant)
            .border_1()
            .border_color(colors.border_variant)
            .text_size(px(12.0))
            .text_color(colors.foreground)
            .font_weight(FontWeight::MEDIUM)
            .child(self.key)
    }
}
