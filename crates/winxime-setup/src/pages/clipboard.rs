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
            Label::new("剪切板")
                .text_size(px(18.0))
                .font_weight(FontWeight::SEMIBOLD)
        )
        .child(
            SettingGroup::new()
                .title("剪切板功能")
                .items(vec![
                    SettingItem::new(
                        "启用剪切板",
                        SettingField::switch(
                            |_| false,
                            |val, _| {
                                println!("剪切板启用: {}", val);
                            },
                        )
                    )
                    .description("启用剪切板历史记录功能"),
                    SettingItem::new(
                        "历史数量",
                        SettingField::number_input(
                            NumberFieldOptions {
                                min: 10.0,
                                max: 100.0,
                                step: 10.0,
                                ..Default::default()
                            },
                            |_| 20.0,
                            |val, _| {
                                println!("历史数量: {}", val);
                            },
                        )
                    )
                    .description("保存的剪切板历史条数"),
                    SettingItem::new(
                        "保留天数",
                        SettingField::number_input(
                            NumberFieldOptions {
                                min: 1.0,
                                max: 30.0,
                                step: 1.0,
                                ..Default::default()
                            },
                            |_| 7.0,
                            |val, _| {
                                println!("保留天数: {}", val);
                            },
                        )
                    )
                    .description("剪切板历史保留天数"),
                ])
        )
        .child(
            SettingGroup::new()
                .title("快捷操作")
                .items(vec![
                    SettingItem::new(
                        "自动粘贴",
                        SettingField::switch(
                            |_| true,
                            |val, _| {
                                println!("自动粘贴: {}", val);
                            },
                        )
                    )
                    .description("选中候选词后自动粘贴到输入位置"),
                    SettingItem::new(
                        "纯文本模式",
                        SettingField::switch(
                            |_| true,
                            |val, _| {
                                println!("纯文本: {}", val);
                            },
                        )
                    )
                    .description("只保存纯文本内容，忽略格式"),
                ])
        )
}