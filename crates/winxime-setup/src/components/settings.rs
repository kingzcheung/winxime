use gpui::{prelude::FluentBuilder, *};
use crate::components::{Switch, Dropdown, NumberInput, Button, Kbd, Label};

pub struct SettingsItem {
    label: String,
    control: SettingsControl,
    description: Option<String>,
}

pub enum SettingsControl {
    Switch(Switch),
    Dropdown(Dropdown),
    NumberInput(NumberInput),
    Button(Button),
    Kbd(Kbd),
    Label(Label),
    Custom(AnyElement),
}

impl SettingsItem {
    pub fn new(label: impl Into<String>, control: SettingsControl) -> Self {
        Self {
            label: label.into(),
            control,
            description: None,
        }
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn switch(checked: bool) -> SettingsControl {
        SettingsControl::Switch(Switch::new(checked))
    }

    pub fn dropdown(options: Vec<(String, String)>, selected: String) -> SettingsControl {
        SettingsControl::Dropdown(Dropdown::new(options, selected))
    }

    pub fn number_input(value: f64) -> SettingsControl {
        SettingsControl::NumberInput(NumberInput::new(value))
    }

    pub fn button(label: impl Into<String>) -> SettingsControl {
        SettingsControl::Button(Button::new(label))
    }

    pub fn kbd(key: impl Into<String>) -> SettingsControl {
        SettingsControl::Kbd(Kbd::new(key))
    }

    pub fn label(text: impl Into<String>) -> SettingsControl {
        SettingsControl::Label(Label::new(text))
    }

    pub fn custom(element: AnyElement) -> SettingsControl {
        SettingsControl::Custom(element)
    }
}

impl IntoElement for SettingsItem {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        let control_element: AnyElement = match self.control {
            SettingsControl::Switch(s) => s.into_any_element(),
            SettingsControl::Dropdown(d) => d.into_any_element(),
            SettingsControl::NumberInput(n) => n.into_any_element(),
            SettingsControl::Button(b) => b.into_any_element(),
            SettingsControl::Kbd(k) => k.into_any_element(),
            SettingsControl::Label(l) => l.into_any_element(),
            SettingsControl::Custom(e) => e,
        };

        div()
            .flex()
            .items_center()
            .justify_between()
            .py(px(12.0))
            .px(px(16.0))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_size(px(14.0))
                            .text_color(rgb(0xe0e0e0))
                            .child(self.label)
                    )
                    .when_some(self.description, |this: Div, desc| {
                        this.child(
                            div()
                                .text_size(px(12.0))
                                .text_color(rgb(0x808080))
                                .child(desc)
                        )
                    })
            )
            .child(control_element)
    }
}

pub struct SettingsGroup {
    title: String,
    items: Vec<SettingsItem>,
}

impl SettingsGroup {
    pub fn new(title: impl Into<String>) -> Self {
        Self { title: title.into(), items: vec![] }
    }

    pub fn items(mut self, items: Vec<SettingsItem>) -> Self {
        self.items = items;
        self
    }
}

impl IntoElement for SettingsGroup {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .py(px(16.0))
            .px(px(16.0))
            .rounded(px(8.0))
            .bg(rgb(0x1e1e1e))
            .child(
                div()
                    .text_size(px(16.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0xe0e0e0))
                    .pb(px(8.0))
                    .child(self.title)
            )
            .children(self.items)
    }
}

pub struct SettingsPage {
    title: String,
    groups: Vec<SettingsGroup>,
}

impl SettingsPage {
    pub fn new(title: impl Into<String>) -> Self {
        Self { title: title.into(), groups: vec![] }
    }

    pub fn group(mut self, group: SettingsGroup) -> Self {
        self.groups.push(group);
        self
    }

    pub fn groups(mut self, groups: Vec<SettingsGroup>) -> Self {
        self.groups = groups;
        self
    }
}

impl IntoElement for SettingsPage {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .p(px(16.0))
            .w_full()
            .child(
                div()
                    .text_size(px(20.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0xe0e0e0))
                    .pb(px(8.0))
                    .child(self.title)
            )
            .children(self.groups)
    }
}