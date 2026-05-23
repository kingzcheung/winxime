use crate::components::{SettingsControl, SettingsGroup, SettingsItem, SettingsPage};
use crate::pages::SettingsApp;
use crate::state::SettingsState;
use gpui::*;
use std::io::Read;
use winxime_config::XimeConfig;
use winxime_predict::check_model_exists;

pub fn render(settings: Entity<SettingsState>, cx: &mut Context<SettingsApp>) -> AnyElement {
    let (
        enabled,
        suggestion_count,
        record_user_frequency,
        auto_adjust_frequency,
        learning_threshold,
        model_downloaded,
        downloading,
        model_name,
        colors,
    ) = cx.read_entity(&settings, |state, _| {
        (
            state.smart_suggestion.enabled,
            state.smart_suggestion.suggestion_count,
            state.smart_suggestion.record_user_frequency,
            state.smart_suggestion.auto_adjust_frequency,
            state.smart_suggestion.learning_threshold,
            check_model_exists(Some(&state.smart_suggestion.model_name)),
            state.smart_suggestion.downloading,
            state.smart_suggestion.model_name.clone(),
            state.colors(),
        )
    });

    let s1 = settings.clone();
    let s2 = settings.clone();
    let s4 = settings.clone();
    let s5 = settings.clone();
    let s6 = settings.clone();
    let s7 = settings.downgrade();

    let model_status = if downloading {
        "下载中..."
    } else if model_downloaded {
        "已下载"
    } else {
        "下载模型"
    };

    SettingsPage::new("智能联想", colors.clone())
        .group(SettingsGroup::new("模型管理", colors.clone()).items(vec![
                SettingsItem::new(
                    "下载模型",
                    SettingsControl::button_with(model_status, move |_window, cx| {
                        if downloading {
                            return;
                        }
                        let entity = s7.clone();

                        let _ = entity.update(cx, |s: &mut SettingsState, cx| {
                            s.smart_suggestion.downloading = true;
                            cx.notify();
                        });

                        cx.spawn(async move |cx| {
                            let result = download_model_async();

                            let _ = cx.update(|cx| {
                                let _ = entity.update(cx, |s: &mut SettingsState, cx| {
                                    s.smart_suggestion.downloading = false;
                                    if result.is_ok() {
                                        s.smart_suggestion.model_downloaded = true;
                                    }
                                    cx.notify();
                                });
                            });
                        }).detach();
                    }),
                )
                .description("智能联想模型（vocab.json + model.onnx）"),
            ]))
        .group(SettingsGroup::new("联想功能", colors.clone()).items(vec![
                    SettingsItem::new("启用智能联想", 
                        SettingsControl::switch_with(enabled,
                            move |val, _window, cx| {
                                s1.update(cx, |s: &mut SettingsState, cx| {
                                    s.smart_suggestion.enabled = val;
                                    if let Err(e) = s.save_smart_suggestion() {
                                        eprintln!("Auto-save smart_suggestion failed: {}", e);
                                    }
                                    cx.notify();
                                });
                            }
                        )
                    ).description("根据输入自动联想可能的词语"),
                    SettingsItem::new("联想词数量", 
                        SettingsControl::number_input_with(suggestion_count as f64,
                            move |val, _window, cx| {
                                s2.update(cx, |s: &mut SettingsState, cx| {
                                    s.smart_suggestion.suggestion_count = val as i32;
                                    if let Err(e) = s.save_smart_suggestion() {
                                        eprintln!("Auto-save smart_suggestion failed: {}", e);
                                    }
                                    cx.notify();
                                });
                            }
                        )
                    ).description("显示的联想词数量"),
                ]))
        .group(SettingsGroup::new("学习功能", colors.clone()).items(vec![
                    SettingsItem::new("记录用户词频", 
                        SettingsControl::switch_with(record_user_frequency,
                            move |val, _window, cx| {
                                s4.update(cx, |s: &mut SettingsState, cx| {
                                    s.smart_suggestion.record_user_frequency = val;
                                    if let Err(e) = s.save_smart_suggestion() {
                                        eprintln!("Auto-save smart_suggestion failed: {}", e);
                                    }
                                    cx.notify();
                                });
                            }
                        )
                    ).description("记录用户输入习惯，优化词序"),
                    SettingsItem::new("自动调频", 
                        SettingsControl::switch_with(auto_adjust_frequency,
                            move |val, _window, cx| {
                                s5.update(cx, |s: &mut SettingsState, cx| {
                                    s.smart_suggestion.auto_adjust_frequency = val;
                                    if let Err(e) = s.save_smart_suggestion() {
                                        eprintln!("Auto-save smart_suggestion failed: {}", e);
                                    }
                                    cx.notify();
                                });
                            }
                        )
                    ).description("根据输入频率自动调整候选词顺序"),
                    SettingsItem::new("学习阈值", 
                        SettingsControl::number_input_with(learning_threshold as f64,
                            move |val, _window, cx| {
                                s6.update(cx, |s: &mut SettingsState, cx| {
                                    s.smart_suggestion.learning_threshold = val as i32;
                                    if let Err(e) = s.save_smart_suggestion() {
                                        eprintln!("Auto-save smart_suggestion failed: {}", e);
                                    }
                                    cx.notify();
                                });
                            }
                        )
                    ).description("输入次数达到阈值后开始调整词序"),
                ]))
        .into_any_element()
}

fn download_model_async() -> Result<(), String> {
    let config = XimeConfig::load();
    let model_name = config.smart_suggestion.model.name.clone();
    let model_dir = winxime_predict::get_model_dir(Some(&model_name));

    if !model_dir.exists() {
        std::fs::create_dir_all(&model_dir).map_err(|e| format!("创建模型目录失败: {}", e))?;
    }

    let files: Vec<_> = config
        .smart_suggestion
        .model
        .files
        .iter()
        .filter(|f| !f.url.is_empty() && !f.filename.is_empty())
        .collect();

    for file in files {
        println!("正在下载 {} from {}...", file.filename, file.url);

        let mut response = ureq::get(&file.url)
            .call()
            .map_err(|e| format!("下载 {} 失败: {}", file.filename, e))?;

        let content = response
            .body_mut()
            .read_to_string()
            .map_err(|e| format!("读取 {} 失败: {}", file.filename, e))?;

        let path = model_dir.join(&file.filename);
        std::fs::write(&path, &content)
            .map_err(|e| format!("保存 {} 失败: {}", file.filename, e))?;

        println!("{} 下载完成", file.filename);
    }

    Ok(())
}
