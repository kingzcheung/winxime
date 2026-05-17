use crate::components::{SettingsGroup, SettingsItem, SettingsPage};
use crate::pages::SettingsApp;
use crate::state::SettingsState;
use gpui::*;

pub fn render(settings: Entity<SettingsState>, cx: &mut Context<SettingsApp>) -> AnyElement {
    let colors = cx.read_entity(&settings, |state, _| state.colors());
    let path_str = winxime_config::XimeConfig::user_data_dir()
        .to_string_lossy()
        .to_string();

    SettingsPage::new("词库管理", colors.clone())
        .group(SettingsGroup::new("词库操作", colors.clone()).items(vec![
                    SettingsItem::new("导入词库", SettingsItem::button("导入"))
                        .description("从文件导入用户词库"),
                    SettingsItem::new("导出词库", SettingsItem::button("导出"))
                        .description("导出用户词库到文件"),
                    SettingsItem::new("同步词库", SettingsItem::button("同步"))
                        .description("重新编译词库"),
                ]))
        .group(SettingsGroup::new("词库信息", colors.clone()).items(vec![
                    SettingsItem::new("词库路径", SettingsItem::label(path_str))
                        .description("用户词库存储位置"),
                    SettingsItem::new("清空词库", SettingsItem::button("清空"))
                        .description("清空用户词库，恢复默认状态"),
                ]))
        .group(SettingsGroup::new("添加词条", colors.clone()).items(vec![
                    SettingsItem::new("添加词条", SettingsItem::button("添加"))
                        .description("手动添加新的词条"),
                ]))
        .into_any_element()
}
