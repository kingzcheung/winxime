use gpui::*;
use crate::components::{SettingsPage, SettingsGroup, SettingsItem};

pub fn render() -> AnyElement {
    SettingsPage::new("快捷键")
        .group(
            SettingsGroup::new("基础快捷键")
                .items(vec![
                    SettingsItem::new("切换中/英文", SettingsItem::kbd("Shift"))
                        .description("按 Shift 切换中/英文输入模式"),
                    SettingsItem::new("翻页键", SettingsItem::label("PageDown / PageUp"))
                        .description("翻页查看更多候选词"),
                    SettingsItem::new("第二三候选", SettingsItem::label("[ / ]"))
                        .description("快速选择第二/第三个候选词"),
                ])
        )
        .group(
            SettingsGroup::new("编辑快捷键")
                .items(vec![
                    SettingsItem::new("清空编码", SettingsItem::kbd("Esc"))
                        .description("按 Esc 清空当前输入编码"),
                    SettingsItem::new("删除末字", SettingsItem::kbd("Backspace"))
                        .description("删除已输入的最后一个字"),
                    SettingsItem::new("回退编码", SettingsItem::kbd("-"))
                        .description("删除最后一个编码字母"),
                ])
        )
        .group(
            SettingsGroup::new("功能快捷键")
                .items(vec![
                    SettingsItem::new("打开设置", SettingsItem::label("Ctrl + Shift + S"))
                        .description("打开设置界面"),
                ])
        )
        .into_any_element()
}