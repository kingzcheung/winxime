use gpui::*;
use gpui_component::{
    h_flex, v_flex,
    label::Label,
    button::Button,
    setting::{SettingGroup, SettingItem, SettingField},
};

pub fn render(cx: &mut Context<super::SettingsApp>) -> impl IntoElement {
    let user_data_dir = winxime_config::XimeConfig::default().user_data_dir();
    let path_str = user_data_dir.to_string_lossy().to_string();

    v_flex()
        .w_full()
        .p_4()
        .gap_4()
        .child(
            Label::new("词库管理")
                .text_size(px(18.0))
                .font_weight(FontWeight::SEMIBOLD)
        )
        .child(
            SettingGroup::new()
                .title("词库操作")
                .items(vec![
                    SettingItem::new(
                        "导入词库",
                        SettingField::render(|_, _, _| {
                            Button::new("import-btn")
                                .label("导入")
                                .on_click(|_, _, _| {
                                    println!("导入词库");
                                })
                                .into_any_element()
                        })
                    )
                    .description("从文件导入用户词库"),
                    SettingItem::new(
                        "导出词库",
                        SettingField::render(|_, _, _| {
                            Button::new("export-btn")
                                .label("导出")
                                .on_click(|_, _, _| {
                                    println!("导出词库");
                                })
                                .into_any_element()
                        })
                    )
                    .description("导出用户词库到文件"),
                    SettingItem::new(
                        "同步词库",
                        SettingField::render(|_, _, _| {
                            Button::new("sync-btn")
                                .label("同步")
                                .on_click(|_, _, _| {
                                    println!("同步词库");
                                })
                                .into_any_element()
                        })
                    )
                    .description("重新编译词库"),
                ])
        )
        .child(
            SettingGroup::new()
                .title("词库信息")
                .items(vec![
                    SettingItem::new(
                        "词库路径",
                        SettingField::render(|_, _, _| {
                            Label::new(path_str.clone())
                                .text_size(px(12.0))
                                .into_any_element()
                        })
                    )
                    .description("用户词库存储位置"),
                    SettingItem::new(
                        "清空词库",
                        SettingField::render(|_, _, _| {
                            Button::new("clear-btn")
                                .label("清空")
                                .on_click(|_, _, _| {
                                    println!("清空词库");
                                })
                                .into_any_element()
                        })
                    )
                    .description("清空用户词库，恢复默认状态"),
                ])
        )
        .child(
            SettingGroup::new()
                .title("词库管理")
                .items(vec![
                    SettingItem::new(
                        "添加词条",
                        SettingField::render(|_, _, _| {
                            Button::new("add-btn")
                                .label("添加")
                                .on_click(|_, _, _| {
                                    println!("添加词条");
                                })
                                .into_any_element()
                        })
                    )
                    .description("手动添加新的词条"),
                ])
        )
}