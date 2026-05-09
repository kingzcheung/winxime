use gpui::*;
use crate::components::{SettingsPage, SettingsGroup, SettingsItem};

pub fn render() -> AnyElement {
    SettingsPage::new("剪切板")
        .group(
            SettingsGroup::new("剪切板功能")
                .items(vec![
                    SettingsItem::new("启用剪切板", SettingsItem::switch(false))
                        .description("启用剪切板历史记录功能"),
                    SettingsItem::new("历史数量", SettingsItem::number_input(20.0))
                        .description("保存的剪切板历史条数"),
                    SettingsItem::new("保留天数", SettingsItem::number_input(7.0))
                        .description("剪切板历史保留天数"),
                ])
        )
        .into_any_element()
}