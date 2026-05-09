use gpui::*;
use gpui_component::{
    h_flex, v_flex,
    label::Label,
    setting::{SettingGroup, SettingItem, SettingField, NumberFieldOptions},
};

pub fn render(cx: &mut Context<super::SettingsApp>) -> impl IntoElement {
    v_flex()
        .w_full()
        .p_4()
        .gap_4()
        .child(
            Label::new("输入方案")
                .text_size(px(18.0))
                .font_weight(FontWeight::SEMIBOLD)
        )
        .child(
            SettingGroup::new()
                .title("基础设置")
                .items(vec![
                    SettingItem::new(
                        "当前方案",
                        SettingField::dropdown(
                            vec![
                                ("五笔86极点".into(), "wubi86_jidian".into()),
                                ("五笔86".into(), "wubi86".into()),
                                ("五笔98".into(), "wubi98".into()),
                            ],
                            |_| "wubi86_jidian".into(),
                            |val, _| {
                                println!("方案切换: {}", val);
                            },
                        )
                    )
                    .description("选择输入方案"),
                    SettingItem::new(
                        "字体大小",
                        SettingField::number_input(
                            NumberFieldOptions {
                                min: 12.0,
                                max: 30.0,
                                step: 2.0,
                                ..Default::default()
                            },
                            |_| 18.0,
                            |val, _| {
                                println!("字体大小: {}", val);
                            },
                        )
                    )
                    .description("候选栏字体大小"),
                    SettingItem::new(
                        "候选数量",
                        SettingField::number_input(
                            NumberFieldOptions {
                                min: 1.0,
                                max: 10.0,
                                step: 1.0,
                                ..Default::default()
                            },
                            |_| 5.0,
                            |val, _| {
                                println!("候选数量: {}", val);
                            },
                        )
                    )
                    .description("候选栏显示的候选词数量"),
                    SettingItem::new(
                        "显示编码提示",
                        SettingField::switch(
                            |_| true,
                            |val, _| {
                                println!("显示编码: {}", val);
                            },
                        )
                    )
                    .description("在候选词旁显示编码"),
                ])
        )
        .child(
            SettingGroup::new()
                .title("外观")
                .items(vec![
                    SettingItem::new(
                        "配色主题",
                        SettingField::dropdown(
                            vec![
                                ("默认".into(), "default".into()),
                                ("深色".into(), "dark".into()),
                                ("浅色".into(), "light".into()),
                            ],
                            |_| "default".into(),
                            |val, _| {
                                println!("主题: {}", val);
                            },
                        )
                    )
                    .description("候选栏配色方案"),
                    SettingItem::new(
                        "圆角大小",
                        SettingField::number_input(
                            NumberFieldOptions {
                                min: 0.0,
                                max: 12.0,
                                step: 1.0,
                                ..Default::default()
                            },
                            |_| 8.0,
                            |val, _| {
                                println!("圆角: {}", val);
                            },
                        )
                    )
                    .description("候选栏窗口圆角"),
                ])
        )
}