use gpui::*;
use crate::components::{SettingsPage, SettingsGroup, SettingsItem};

pub fn render() -> AnyElement {
    SettingsPage::new("智能联想")
        .group(
            SettingsGroup::new("联想功能")
                .items(vec![
                    SettingsItem::new("启用智能联想", SettingsItem::switch(true))
                        .description("根据输入自动联想可能的词语"),
                    SettingsItem::new("联想词数量", SettingsItem::number_input(3.0))
                        .description("显示的联想词数量"),
                    SettingsItem::new("优先常用词", SettingsItem::switch(true))
                        .description("优先显示常用词"),
                ])
        )
        .group(
            SettingsGroup::new("学习功能")
                .items(vec![
                    SettingsItem::new("记录用户词频", SettingsItem::switch(true))
                        .description("记录用户输入习惯，优化词序"),
                    SettingsItem::new("自动调频", SettingsItem::switch(true))
                        .description("根据输入频率自动调整候选词顺序"),
                    SettingsItem::new("学习阈值", SettingsItem::number_input(3.0))
                        .description("输入次数达到阈值后开始调整词序"),
                ])
        )
        .into_any_element()
}