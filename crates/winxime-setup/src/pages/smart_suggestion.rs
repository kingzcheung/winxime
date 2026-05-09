use gpui::*;
use gpui_component::{
    v_flex,
    label::Label,
    setting::{SettingGroup, SettingItem, SettingField, NumberFieldOptions},
};

pub fn render(cx: &mut Context<super::SettingsApp>) -> impl IntoElement {
    v_flex()
        .w_full()
        .p_4()
        .gap_4()
        .child(
            Label::new("智能联想")
                .text_size(px(18.0))
                .font_weight(FontWeight::SEMIBOLD)
        )
        .child(
            SettingGroup::new()
                .title("联想功能")
                .items(vec![
                    SettingItem::new(
                        "启用智能联想",
                        SettingField::switch(
                            |_| true,
                            |val, _| {
                                println!("智能联想启用: {}", val);
                            },
                        )
                    )
                    .description("根据输入自动联想可能的词语"),
                    SettingItem::new(
                        "联想词数量",
                        SettingField::number_input(
                            NumberFieldOptions {
                                min: 1.0,
                                max: 5.0,
                                step: 1.0,
                                ..Default::default()
                            },
                            |_| 3.0,
                            |val, _| {
                                println!("联想词数量: {}", val);
                            },
                        )
                    )
                    .description("显示的联想词数量"),
                    SettingItem::new(
                        "优先常用词",
                        SettingField::switch(
                            |_| true,
                            |val, _| {
                                println!("优先常用词: {}", val);
                            },
                        )
                    )
                    .description("优先显示常用词"),
                ])
        )
        .child(
            SettingGroup::new()
                .title("学习功能")
                .items(vec![
                    SettingItem::new(
                        "记录用户词频",
                        SettingField::switch(
                            |_| true,
                            |val, _| {
                                println!("记录词频: {}", val);
                            },
                        )
                    )
                    .description("记录用户输入习惯，优化词序"),
                    SettingItem::new(
                        "自动调频",
                        SettingField::switch(
                            |_| true,
                            |val, _| {
                                println!("自动调频: {}", val);
                            },
                        )
                    )
                    .description("根据输入频率自动调整候选词顺序"),
                    SettingItem::new(
                        "学习阈值",
                        SettingField::number_input(
                            NumberFieldOptions {
                                min: 1.0,
                                max: 10.0,
                                step: 1.0,
                                ..Default::default()
                            },
                            |_| 3.0,
                            |val, _| {
                                println!("学习阈值: {}", val);
                            },
                        )
                    )
                    .description("输入次数达到阈值后开始调整词序"),
                ])
        )
}