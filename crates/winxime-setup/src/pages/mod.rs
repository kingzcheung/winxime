mod input_schema;
mod clipboard;
mod hotkeys;
mod smart_suggestion;
mod dictionary;

use gpui::*;
use gpui_component::{
    h_flex, v_flex,
    button::Button,
    scrollable::Scrollable,
    setting::{
        Settings, SettingPage, SettingGroup, SettingItem, SettingField,
        NumberFieldOptions,
    },
    sidebar::{
        Sidebar, SidebarHeader, SidebarGroup, SidebarMenu, SidebarMenuItem,
    },
    Icon, IconName,
};

pub struct SettingsApp {
    current_page: usize,
}

impl SettingsApp {
    pub fn new() -> Self {
        Self { current_page: 0 }
    }
}

impl Render for SettingsApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pages = vec![
            ("输入方案", IconName::Keyboard),
            ("剪切板", IconName::ClipboardList),
            ("快捷键", IconName::Keyboard),
            ("智能联想", IconName::Brain),
            ("词库管理", IconName::BookOpen),
        ];

        h_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(
                Sidebar::new()
                    .width(200)
                    .header(
                        SidebarHeader::new()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(Icon::new(IconName::Settings))
                                    .child("Xime 设置")
                            )
                    )
                    .child(
                        SidebarGroup::new()
                            .child(
                                SidebarMenu::new()
                                    .children(pages.iter().enumerate().map(|(i, (name, icon))| {
                                        SidebarMenuItem::new(name)
                                            .icon(*icon)
                                            .active(i == self.current_page)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.current_page = i;
                                                cx.notify();
                                            }))
                                    }))
                            )
                    )
            )
            .child(self.render_content(cx))
    }

    fn render_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        Scrollable::new(v_flex().flex_1().size_full())
            .child(match self.current_page {
                0 => input_schema::render(cx),
                1 => clipboard::render(cx),
                2 => hotkeys::render(cx),
                3 => smart_suggestion::render(cx),
                4 => dictionary::render(cx),
                _ => input_schema::render(cx),
            })
    }
}