use crate::components::{Button, SettingsControl, SettingsGroup, SettingsItem, SettingsPage};
use crate::pages::SettingsApp;
use crate::state::{PairState, SettingsState};
use crate::theme::ThemeColors;
use gpui::prelude::FluentBuilder;
use gpui::*;

const XIMED_PORT: u16 = 8370;

pub fn render(settings: Entity<SettingsState>, cx: &mut Context<SettingsApp>) -> AnyElement {
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

    SettingsGroup::new("服务器状态", colors.clone()).items(vec![
        SettingsItem::new("剪贴板共享服务", SettingsControl::label(status_text)),
        SettingsItem::new(
            "",
            SettingsControl::button_with("刷新状态", move |_window, cx| {
                settings_btn.update(cx, |s: &mut SettingsState, cx| {
                    s.pair.refresh();
                    cx.notify();
                });
            }),
        ),
    ])
}

fn render_add_device(
    state: &PairState,
    colors: &ThemeColors,
    settings: Entity<SettingsState>,
) -> SettingsGroup {
    let mut group = SettingsGroup::new("关联设备", colors.clone());

    let code = state.code.clone();
    let code_len = code.len();
    let settings_toggle = settings.clone();

    if !state.editing {
        group = group.items(vec![SettingsItem::new(
            "",
            SettingsControl::button_with("关联设备", move |_window, cx| {
                settings_toggle.update(cx, |s: &mut SettingsState, cx| {
                    s.pair.editing = true;
                    cx.notify();
                });
            }),
        )]);
    } else {
        let mut digits: Vec<String> = code.chars().map(|c| c.to_string()).collect();
        while digits.len() < 6 {
            digits.push(String::new());
        }

        let s = settings.clone();
        let s2 = settings.clone();
        let s3 = settings.clone();
        let s4 = settings.clone();
        let s5 = settings.clone();

        let digit_area = div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .justify_center()
            .pb(px(16.0))
            .children(digits.iter().enumerate().map(|(i, d)| {
                let filled = !d.is_empty();
                div()
                    .id(("pair-digit", i))
                    .w(px(36.0))
                    .h(px(44.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(8.0))
                    .bg(if filled {
                        colors.primary
                    } else {
                        colors.surface_variant
                    })
                    .text_size(px(20.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(if filled {
                        colors.on_primary
                    } else {
                        colors.foreground_muted
                    })
                    .child(if filled { d.clone() } else { "".to_string() })
            }));

        let numpad =
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .flex()
                        .gap(px(8.0))
                        .child(digit_button("1", colors, &s))
                        .child(digit_button("2", colors, &s2))
                        .child(digit_button("3", colors, &s3)),
                )
                .child(
                    div()
                        .flex()
                        .gap(px(8.0))
                        .child(digit_button("4", colors, &s))
                        .child(digit_button("5", colors, &s2))
                        .child(digit_button("6", colors, &s3)),
                )
                .child(
                    div()
                        .flex()
                        .gap(px(8.0))
                        .child(digit_button("7", colors, &s))
                        .child(digit_button("8", colors, &s2))
                        .child(digit_button("9", colors, &s3)),
                )
                .child(
                    div()
                        .flex()
                        .gap(px(8.0))
                        .child(Button::new("清除").theme(colors.clone()).on_click(
                            move |_window, cx| {
                                s4.update(cx, |s: &mut SettingsState, cx| {
                                    s.pair.code.clear();
                                    cx.notify();
                                });
                            },
                        ))
                        .child(digit_button("0", colors, &s))
                        .child(Button::new("⌫").theme(colors.clone()).on_click(
                            move |_window, cx| {
                                s5.update(cx, |s: &mut SettingsState, cx| {
                                    s.pair.code.pop();
                                    cx.notify();
                                });
                            },
                        )),
                );

        let actions = div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .pt(px(12.0))
            .child(Button::new("取消").theme(colors.clone()).on_click({
                let s = settings.clone();
                move |_window, cx| {
                    s.update(cx, |s: &mut SettingsState, cx| {
                        s.pair.code.clear();
                        s.pair.editing = false;
                        cx.notify();
                    });
                }
            }))
            .when(code_len == 6, |this: Div| {
                this.child(Button::new("确认配对").theme(colors.clone()).on_click({
                    let s = settings.clone();
                    move |_window, cx| {
                        s.update(cx, |s: &mut SettingsState, cx| {
                            let code = s.pair.code.clone();
                            match PairState::confirm_code(&code) {
                                Ok(msg) => {
                                    s.pair.status_message = msg;
                                    s.pair.code.clear();
                                    s.pair.editing = false;
                                    s.pair.refresh();
                                }
                                Err(e) => {
                                    s.pair.status_message = e;
                                }
                            }
                            cx.notify();
                        });
                    }
                }))
            });

        let msg = state.status_message.clone();
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
                    .child("在手机端生成配对码后，输入 6 位数字完成配对"),
            )
            .child(digit_area)
            .child(numpad)
            .when(!msg.is_empty(), |this: Div| {
                this.child(
                    div()
                        .text_size(px(12.0))
                        .text_color(colors.foreground_muted)
                        .child(msg),
                )
            })
            .child(actions);

        group = group.custom_item(content);
    }

    group
}

fn digit_button(
    label: &str,
    colors: &ThemeColors,
    settings: &Entity<SettingsState>,
) -> impl IntoElement {
    let digit = label.to_string();
    let display = label.to_string();
    let s = settings.clone();
    div()
        .id(format!("numpad-{}", label))
        .w(px(48.0))
        .h(px(48.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(10.0))
        .bg(colors.surface_variant)
        .text_size(px(20.0))
        .text_color(colors.foreground)
        .cursor_pointer()
        .hover(|style| style.bg(colors.border_variant))
        .on_click(move |_, _window, cx| {
            s.update(cx, |s: &mut SettingsState, cx| {
                if s.pair.code.len() < 6 {
                    s.pair.code.push_str(&digit);
                }
                cx.notify();
            });
        })
        .child(display)
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
