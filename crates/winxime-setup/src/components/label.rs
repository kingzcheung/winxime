use crate::theme::ThemeColors;
use gpui::*;

#[derive(Clone)]
pub struct Label {
    text: String,
    colors: Option<ThemeColors>,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            colors: None,
        }
    }

    pub fn theme(mut self, colors: ThemeColors) -> Self {
        self.colors = Some(colors);
        self
    }
}

impl IntoElement for Label {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        let colors = self.colors.unwrap_or_else(|| {
            ThemeColors::from_theme(&crate::theme::SystemTheme::Light, 0x8F73E2)
        });

        div()
            .text_size(px(14.0))
            .text_color(colors.foreground_muted)
            .child(self.text)
    }
}
