use crate::components::{Button, SettingsControl, SettingsGroup, SettingsItem, SettingsPage};
use crate::pages::SettingsApp;
use crate::state::{PairState, SettingsState};
use crate::theme::ThemeColors;
use gpui::prelude::FluentBuilder;
use gpui::*;

const XIMED_PORT: u16 = 8370;

pub fn render(settings: Entity<SettingsState>, cx: &mut Context<SettingsApp>) -> AnyElement {
    // 先检查是否需要首次加载刷新（只读 boolean，不克隆整个 state）
    let needs_refresh = cx.read_entity(&settings, |s, _| !s.pair.loaded);

    if needs_refresh {
        // 先刷新数据（直接修改实体中的 PairState）
        settings.update(cx, |s: &mut SettingsState, cx| {
            s.pair.loaded = true;
            s.pair.refresh();
            cx.notify();
        });
    }

    // 刷新完成后再克隆 state，保证拿到最新数据
    let (state, colors) = cx.read_entity(&settings, |s, _| (s.pair.clone(), s.colors()));

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
            .pb(px(8.0))
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
                            s.pair.polling = false;
                            s.pair.refresh();
                            cx.notify();
                        });
                    }),
            );

        let waiting_text = if state.polling {
            Some(
                div()
                    .text_size(px(13.0))
                    .text_color(colors.primary)
                    .pt(px(4.0))
                    .child("等待手机端确认..."),
            )
        } else {
            None
        };

        let content = div()
            .px(px(16.0))
            .pb(px(12.0))
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(
                div()
                    .text_size(px(14.0))
                    .text_color(colors.foreground_muted)
                    .child("在手机端输入以下配对码完成关联"),
            )
            .child(code_display)
            .when_some(waiting_text, |this, text| this.child(text))
            .child(actions);

        group = group.custom_item(content);
    } else {
        let s = settings.clone();
        group = group.items(vec![SettingsItem::new(
            "本机设备",
            SettingsControl::button_with("查看匹配码", move |window, cx| {
                let poll_settings = s.clone();
                s.update(cx, |s: &mut SettingsState, cx| {
                    match s.pair.request_pair_code() {
                        Ok(()) => {
                            s.pair.showing_code = true;
                            s.pair.polling = true;
                            // 在 Context<SettingsState> 中 spawn 异步轮询任务
                            cx.spawn(async move |_weak, cx| {
                                loop {
                                    smol::Timer::after(
                                        std::time::Duration::from_secs(2),
                                    )
                                    .await;

                                    let should_stop = cx.update(|cx| {
                                        poll_settings.update(
                                            cx,
                                            |s: &mut SettingsState, cx| {
                                                if !s.pair.showing_code
                                                    || s.pair.local_code.is_empty()
                                                {
                                                    s.pair.polling = false;
                                                    cx.notify();
                                                    return true;
                                                }

                                                match s.pair.poll_pair_status() {
                                                    Ok(true) => {
                                                        s.pair.showing_code = false;
                                                        s.pair.polling = false;
                                                        s.pair.status_message =
                                                            "设备配对成功！".to_string();
                                                        s.pair.refresh();
                                                        cx.notify();
                                                        true
                                                    }
                                                    Ok(false) => false,
                                                    Err(e) => {
                                                        s.pair.polling = false;
                                                        s.pair.status_message = e;
                                                        cx.notify();
                                                        true
                                                    }
                                                }
                                            },
                                        )
                                    });

                                    if should_stop {
                                        break;
                                    }
                                }
                            })
                            .detach();
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
    let settings_btn = settings.clone();
    let mut group = SettingsGroup::new("已配对设备", colors.clone()).items(vec![
        SettingsItem::new(
            format!("共 {} 台设备", state.devices.len()),
            SettingsControl::button_with("刷新", move |_window, cx| {
                settings_btn.update(cx, |s: &mut SettingsState, cx| {
                    s.pair.refresh();
                    cx.notify();
                });
            }),
        ),
    ]);

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
                            settings_remove.update(cx, |s: &mut SettingsState, cx| {
                                match s.pair.remove_device(&device_id) {
                                    Err(e) => {
                                        s.pair.status_message = e;
                                    }
                                    Ok(()) => {
                                        s.pair.status_message = format!("已移除 {}", device_name);
                                        s.pair.refresh();
                                    }
                                }
                                cx.notify();
                            });
                        },
                    )),
            );
        }
    }

    group
}
