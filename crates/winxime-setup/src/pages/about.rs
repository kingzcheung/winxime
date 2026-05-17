use crate::pages::SettingsApp;
use crate::state::SettingsState;
use crate::theme::ThemeColors;
use gpui::*;

pub fn render(settings: Entity<SettingsState>, cx: &mut Context<SettingsApp>) -> AnyElement {
    let colors = cx.read_entity(&settings, |state, _| state.colors());

    div()
        .flex()
        .flex_col()
        .gap(px(16.0))
        .p(px(16.0))
        .w_full()
        .child(
            div()
                .text_size(px(20.0))
                .font_weight(FontWeight::BOLD)
                .text_color(colors.foreground)
                .child("关于"),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(24.0))
                .p(px(20.0))
                .rounded(px(16.0))
                .bg(colors.surface)
                .border_1()
                .border_color(colors.border)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(16.0))
                        .child(img("image/icon.png").w(px(64.0)).h(px(64.0)))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .child(
                                    div()
                                        .text_size(px(24.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(colors.foreground)
                                        .child("Xime 曦码输入法"),
                                )
                                .child(
                                    div()
                                        .text_size(px(14.0))
                                        .text_color(colors.foreground_muted)
                                        .child("基于 Rime 引擎的 Windows 五笔输入法"),
                                ),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(12.0))
                .p(px(20.0))
                .rounded(px(16.0))
                .bg(colors.surface)
                .border_1()
                .border_color(colors.border)
                .child(
                    div()
                        .text_size(px(16.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(colors.foreground)
                        .child("基本信息"),
                )
                .child(render_info_row(
                    "版本".to_string(),
                    env!("CARGO_PKG_VERSION").to_string(),
                    &colors,
                ))
                .child(render_info_row(
                    "作者".to_string(),
                    env!("CARGO_PKG_AUTHORS").to_string(),
                    &colors,
                ))
                .child(render_link_row(
                    "仓库".to_string(),
                    "https://github.com/ximeiorg/winxime".to_string(),
                    &colors,
                ))
                .child(render_info_row(
                    "许可".to_string(),
                    "GPL-3.0".to_string(),
                    &colors,
                )),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(12.0))
                .p(px(20.0))
                .rounded(px(16.0))
                .bg(colors.surface)
                .border_1()
                .border_color(colors.border)
                .child(
                    div()
                        .text_size(px(16.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(colors.foreground)
                        .child("技术栈"),
                )
                .child(render_info_row(
                    "输入引擎".to_string(),
                    "Rime (librime)".to_string(),
                    &colors,
                ))
                .child(render_info_row(
                    "界面框架".to_string(),
                    "Rust + TSF".to_string(),
                    &colors,
                ))
                .child(render_info_row(
                    "UI 框架".to_string(),
                    "GPUI".to_string(),
                    &colors,
                )),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(12.0))
                .p(px(20.0))
                .rounded(px(16.0))
                .bg(colors.surface)
                .border_1()
                .border_color(colors.border)
                .child(
                    div()
                        .text_size(px(16.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(colors.error)
                        .child("卸载"),
                )
                .child(
                    div()
                        .text_size(px(14.0))
                        .text_color(colors.foreground_muted)
                        .child(
                            "卸载将删除 System32 中的 TSF DLL、注册表相关键和开始菜单快捷方式。",
                        ),
                )
                .child(
                    div()
                        .text_size(px(14.0))
                        .text_color(colors.foreground_muted)
                        .child(
                            "用户数据目录 (%APPDATA%\\Xime) 将保留，重新安装后可恢复配置和词库。",
                        ),
                )
                .child(
                    div()
                        .id("uninstall-btn")
                        .px(px(16.0))
                        .py(px(8.0))
                        .rounded(px(8.0))
                        .bg(colors.error)
                        .text_color(colors.on_error)
                        .text_size(px(14.0))
                        .font_weight(FontWeight::BOLD)
                        .cursor_pointer()
                        .child("卸载输入法")
                        .on_click(|_, _, cx| {
                            perform_uninstall(cx);
                        }),
                ),
        )
        .into_any_element()
}

fn perform_uninstall(cx: &mut App) {
    let exe_path = std::env::current_exe().ok();
    let exe_dir = exe_path.and_then(|p| p.parent().map(|p| p.to_path_buf()));

    if let Some(exe_dir) = exe_dir {
        let register_exe = exe_dir.join("winxime-tsf-register.exe");

        if register_exe.exists() {
            let _ = std::process::Command::new(&register_exe)
                .arg("-unregister-and-remove")
                .status();

            let install_dir = exe_dir.to_string_lossy().to_string();
            if install_dir.contains("Program Files") {
                let _ = std::process::Command::new("cmd")
                    .args([
                        "/c",
                        "ping",
                        "127.0.0.1",
                        "-n",
                        "3",
                        ">nul",
                        "&",
                        "rmdir",
                        "/s",
                        "/q",
                        &install_dir,
                    ])
                    .spawn();
            }

            let title = "卸载完成";
            let message = "Xime 输入法已卸载。\n\n请重启 Windows 以完成清理。\n\n程序将在关闭后删除安装目录。";

            #[cfg(windows)]
            {
                use windows::core::PCWSTR;
                use windows::Win32::UI::WindowsAndMessaging::*;

                let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
                let message_wide: Vec<u16> =
                    message.encode_utf16().chain(std::iter::once(0)).collect();

                unsafe {
                    MessageBoxW(
                        None,
                        PCWSTR(message_wide.as_ptr()),
                        PCWSTR(title_wide.as_ptr()),
                        MB_OK | MB_ICONINFORMATION,
                    );
                }
            }

            cx.quit();
        }
    }
}

fn render_info_row(label: String, value: String, colors: &ThemeColors) -> Div {
    div()
        .flex()
        .items_center()
        .justify_between()
        .py(px(8.0))
        .child(
            div()
                .text_size(px(14.0))
                .text_color(colors.foreground_muted)
                .child(label),
        )
        .child(
            div()
                .text_size(px(14.0))
                .text_color(colors.foreground)
                .child(value),
        )
}

fn render_link_row(label: String, url: String, colors: &ThemeColors) -> Div {
    div()
        .flex()
        .items_center()
        .justify_between()
        .py(px(8.0))
        .child(
            div()
                .text_size(px(14.0))
                .text_color(colors.foreground_muted)
                .child(label),
        )
        .child(
            div()
                .text_size(px(14.0))
                .text_color(colors.primary)
                .child(url),
        )
}
