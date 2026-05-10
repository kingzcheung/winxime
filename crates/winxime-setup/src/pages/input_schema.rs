use gpui::{prelude::FluentBuilder, ParentElement, IntoElement, *};
use crate::components::{Radio};
use crate::state::SettingsState;
use crate::pages::SettingsApp;

pub fn render(settings: Entity<SettingsState>, cx: &mut Context<SettingsApp>) -> AnyElement {
    let (selected, colors, is_dark) = cx.read_entity(&settings, |state, _| {
        (state.input_schema.selected_schema, state.colors(), state.system_theme.is_dark())
    });
    
    let schemas = ["五笔86极点", "五笔86", "五笔98"];
    
    let items: Vec<AnyElement> = schemas
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let is_selected = i == selected;
            let settings_clone = settings.clone();
            let primary = colors.primary.clone();
            let is_dark_clone = is_dark;
            
            div()
                .id(("schema", i))
                .flex()
                .items_center()
                .gap(px(12.0))
                .py(px(8.0))
                .px(px(12.0))
                .rounded(px(8.0))
                .cursor_pointer()
                .hover(|style: StyleRefinement| {
                    if is_dark_clone {
                        style.bg(rgb(0x262626))
                    } else {
                        style.bg(rgb(0xf5f5f5))
                    }
                })
                .when(is_selected, |this: Stateful<Div>| {
                    this.border_1()
                        .border_color(primary.clone())
                        .bg(if is_dark_clone { rgb(0x3d2d5d) } else { rgb(0xe8e0f8) })
                })
                .on_click(move |_event, _window, cx| {
                    settings_clone.update(cx, |s: &mut SettingsState, cx| {
                        s.input_schema.selected_schema = i;
                        cx.notify();
                    });
                })
                .child(Radio::new(is_selected))
                .child(
                    div()
                        .text_size(px(14.0))
                        .text_color(if is_selected { primary } else { colors.foreground })
                        .child(name.to_string())
                )
                .into_any_element()
        })
        .collect();
    
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
                .child("输入方案")
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(12.0))
                .p(px(16.0))
                .rounded(px(12.0))
                .bg(colors.surface)
                .border_1()
                .border_color(colors.border)
                .child(
                    div()
                        .text_size(px(14.0))
                        .text_color(colors.foreground_muted)
                        .child("选择输入方案")
                )
                .children(items)
        )
        .into_any_element()
}