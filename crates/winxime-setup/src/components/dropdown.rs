use crate::theme::ThemeColors;
use gpui::*;

#[derive(Clone)]
pub struct Dropdown {
    selected_label: String,
    colors: Option<ThemeColors>,
}

impl Dropdown {
    pub fn new(options: Vec<(String, String)>, selected: String) -> Self {
        let selected_label = options
            .iter()
            .find(|(_, v)| v == &selected)
            .map(|(l, _)| l.clone())
            .unwrap_or_else(|| selected.clone());
        Self {
            selected_label,
            colors: None,
        }
    }

    pub fn theme(mut self, colors: ThemeColors) -> Self {
        self.colors = Some(colors);
        self
    }
}

impl IntoElement for Dropdown {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        let colors = self.colors.unwrap_or_else(|| {
            ThemeColors::from_theme(&crate::theme::SystemTheme::Light, 0x8F73E2)
        });

        div()
            .py(px(6.0))
            .px(px(12.0))
            .rounded(px(4.0))
            .bg(colors.surface_variant)
            .border_1()
            .border_color(colors.border_variant)
            .text_size(px(14.0))
            .text_color(colors.foreground)
            .cursor_pointer()
            .child(self.selected_label)
    }
}
