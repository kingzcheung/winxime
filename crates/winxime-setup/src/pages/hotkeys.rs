use gpui::*;
use gpui_component::{
    h_flex, v_flex,
    label::Label,
    kbd::Kbd,
    setting::{SettingGroup, SettingItem, SettingField},
};

pub fn render(cx: &mut Context<super::SettingsApp>) -> impl IntoElement {
    v_flex()
        .w_full()
        .p_4()
        .gap_4()
        .child(
            Label::new("快捷键")
                .text_size(px(18.0))
                .font_weight(FontWeight::SEMIBOLD)
        )
        .child(
            SettingGroup::new()
                .title("基础快捷键")
                .items(vec![
                    SettingItem::new(
                        "切换中/英文",
                        SettingField::render(|_, _, _| {
                            h_flex()
                                .gap_1()
                                .child(Kbd::new("Shift"))
                                .into_any_element()
                        })
                    )
                    .description("按 Shift 切换中/英文输入模式"),
                    SettingItem::new(
                        "翻页键",
                        SettingField::render(|_, _, _| {
                            h_flex()
                                .gap_1()
                                .child(Kbd::new("PageDown"))
                                .child("/")
                                .child(Kbd::new("PageUp"))
                                .into_any_element()
                        })
                    )
                    .description("翻页查看更多候选词"),
                    SettingItem::new(
                        "第二三候选",
                        SettingField::render(|_, _, _| {
                            h_flex()
                                .gap_1()
                                .child(Kbd::new("["))
                                .child(Kbd::new("]"))
                                .into_any_element()
                        })
                    )
                    .description("快速选择第二/第三个候选词"),
                ])
        )
        .child(
            SettingGroup::new()
                .title("编辑快捷键")
                .items(vec![
                    SettingItem::new(
                        "清空编码",
                        SettingField::render(|_, _, _| {
                            h_flex()
                                .gap_1()
                                .child(Kbd::new("Esc"))
                                .into_any_element()
                        })
                    )
                    .description("按 Esc 清空当前输入编码"),
                    SettingItem::new(
                        "删除末字",
                        SettingField::render(|_, _, _| {
                            h_flex()
                                .gap_1()
                                .child(Kbd::new("Backspace"))
                                .into_any_element()
                        })
                    )
                    .description("删除已输入的最后一个字"),
                    SettingItem::new(
                        "回退编码",
                        SettingField::render(|_, _, _| {
                            h_flex()
                                .gap_1()
                                .child(Kbd::new("-"))
                                .into_any_element()
                        })
                    )
                    .description("删除最后一个编码字母"),
                ])
        )
        .child(
            SettingGroup::new()
                .title("功能快捷键")
                .items(vec![
                    SettingItem::new(
                        "打开设置",
                        SettingField::render(|_, _, _| {
                            h_flex()
                                .gap_1()
                                .child(Kbd::new("Ctrl"))
                                .child("+")
                                .child(Kbd::new("Shift"))
                                .child("+")
                                .child(Kbd::new("S"))
                                .into_any_element()
                        })
                    )
                    .description("打开设置界面"),
                ])
        )
}