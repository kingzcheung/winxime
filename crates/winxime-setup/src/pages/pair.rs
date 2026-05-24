use crate::components::{Button, SettingsControl, SettingsGroup, SettingsItem, SettingsPage};
use crate::pages::SettingsApp;
use crate::state::{PairState, SettingsState};
use crate::theme::ThemeColors;
use gpui::prelude::FluentBuilder;
use gpui::*;

const XIMED_PORT: u16 = 8370;

pub fn render(settings: Entity<SettingsState>, cx: &mut Context<SettingsApp>) -> AnyElement {
    let (state, colors) = cx.read_entity(&settings, |s, _| (s.pair.clone(), s.colors()));

    if !state.loaded {
        settings.update(cx, |s: &mut SettingsState, cx| {
            s.pair.loaded = true;
            s.pair.refresh();
            cx.notify();
        });
    }

    SettingsPage::new("设备关联", colors.clone())
        .group(render_server_status(&state, &colors, settings.clone()))
        .group(render_add_device(&state, &colors, settings.clone()))
        .group(render_device_list(&state, &colors, settings.clone()))
        .into_any_element()
}

fn render_server_status(
    state: &PairState,
    colors: &ThemeColors,
    settings: Entity<SettingsState>,
) -> SettingsGroup {
    let status_text = if state.server_running {
        format!("运行中 (端口 {})", XIMED_PORT)
    } else {
        "未运行".to_string()
    };

    let settings_btn = settings.clone();

    SettingsGroup::new("服务器状态", colors.clone()).items(vec![SettingsItem::new(
        status_text,
        SettingsControl::button_with("刷新状态", move |_window, cx| {
            settings_btn.update(cx, |s: &mut SettingsState, cx| {
                s.pair.refresh();
                cx.notify();
            });
        }),
    )])
}

fn render_add_device(
    state: &PairState,
    colors: &ThemeColors,
    settings: Entity<SettingsState>,
) -> SettingsGroup {
    let mut group = SettingsGroup::new("关联设备", colors.clone());

    if state.showing_code {
        let digits: Vec<char> = state.local_code.chars().collect();

        let code_display = div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .justify_center()
            .pb(px(16.0))
            .children(digits.iter().enumerate().map(|(i, d)| {
                div()
                    .id(("pair-local-code", i))
                    .w(px(36.0))
                    .h(px(44.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(8.0))
                    .bg(colors.primary.alpha(0.1))
                    .border_1()
                    .border_color(colors.primary.alpha(0.5))
                    .text_size(px(20.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(colors.on_primary)
                    .child(d.to_string())
            }));

        let s = settings.clone();
        let actions = div()
            .flex()
            .items_center()
            .justify_center()
            .gap(px(8.0))
            .pt(px(12.0))
            .child(
                Button::new("关闭")
                    .theme(colors.clone())
                    .on_click(move |_window, cx| {
                        s.update(cx, |s: &mut SettingsState, cx| {
                            s.pair.showing_code = false;
                            cx.notify();
                        });
                    }),
            );

        let content = div()
            .px(px(16.0))
            .pb(px(12.0))
            .flex()
            .flex_col()
            .gap(px(12.0))
            .child(
                div()
                    .text_size(px(14.0))
                    .text_color(colors.foreground_muted)
                    .child("在手机端输入以下配对码完成关联"),
            )
            .child(code_display)
            .child(actions);

        group = group.custom_item(content);
    } else {
        let s = settings.clone();
        group = group.items(vec![SettingsItem::new(
            "本机设备",
            SettingsControl::button_with("查看匹配码", move |_window, cx| {
                s.update(cx, |s: &mut SettingsState, cx| {
                    match s.pair.request_pair_code() {
                        Ok(()) => {
                            s.pair.showing_code = true;
                        }
                        Err(e) => {
                            s.pair.status_message = e;
                        }
                    }
                    cx.notify();
                });
            }),
        )]);
    }

    group
}

fn render_device_list(
    state: &PairState,
    colors: &ThemeColors,
    settings: Entity<SettingsState>,
) -> SettingsGroup {
    let mut group = SettingsGroup::new("已配对设备", colors.clone());

    if state.devices.is_empty() {
        group = group.custom_item(
            div().px(px(16.0)).pb(px(12.0)).child(
                div()
                    .text_size(px(14.0))
                    .text_color(colors.foreground_muted)
                    .child("暂无已配对设备"),
            ),
        );
    } else {
        for device in &state.devices {
            let device_id = device.device_id.clone();
            let device_name = device.device_name.clone();
            let paired_at = device.paired_at.clone();
            let settings_remove = settings.clone();

            group = group.custom_item(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px(px(16.0))
                    .py(px(4.0))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .text_size(px(14.0))
                                    .text_color(colors.foreground)
                                    .child(device_name.clone()),
                            )
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .text_color(colors.foreground_muted)
                                    .child(format!(
                                        "配对时间: {}",
                                        &paired_at[..19].replace("T", " ")
                                    )),
                            ),
                    )
                    .child(Button::new("移除").theme(colors.clone()).on_click(
                        move |_window, cx| {
                            if let Err(e) = PairState::remove_device(&device_id) {
                                settings_remove.update(cx, |s: &mut SettingsState, cx| {
                                    s.pair.status_message = e;
                                    cx.notify();
                                });
                            } else {
                                settings_remove.update(cx, |s: &mut SettingsState, cx| {
                                    s.pair.status_message = format!("已移除 {}", device_name);
                                    s.pair.refresh();
                                    cx.notify();
                                });
                            }
                        },
                    )),
            );
        }
    }

    group
}
