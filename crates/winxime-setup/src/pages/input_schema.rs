use gpui::{prelude::FluentBuilder, ParentElement, IntoElement, *};
use crate::components::{Radio};
use crate::state::SettingsState;
use crate::pages::SettingsApp;

pub fn render(settings: Entity<SettingsState>, cx: &mut Context<SettingsApp>) -> AnyElement {
    let selected = cx.read_entity(&settings, |state, _| state.input_schema.selected_schema);
    
    let schemas = ["五笔86极点", "五笔86", "五笔98"];
    
    let items: Vec<AnyElement> = schemas
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let is_selected = i == selected;
            let settings_clone = settings.clone();
            let primary = rgb(0x8F73E2);
            
            div()
                .id(("schema", i))
                .flex()
                .items_center()
                .gap(px(12.0))
                .py(px(8.0))
                .px(px(12.0))
                .rounded(px(8.0))
                .cursor_pointer()
                .hover(|style: StyleRefinement| style.bg(rgb(0x262626)))
                .when(is_selected, |this: Stateful<Div>| {
                    this.border_1()
                        .border_color(primary)
                        .bg(rgb(0x3d2d5d))
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
                        .text_color(if is_selected { primary } else { rgb(0xe6e6e6) })
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
                .text_color(rgb(0xe0e0e0))
                .child("输入方案")
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(12.0))
                .p(px(16.0))
                .rounded(px(12.0))
                .bg(rgb(0x1a1a1a))
                .border_1()
                .border_color(rgb(0x303030))
                .child(
                    div()
                        .text_size(px(14.0))
                        .text_color(rgb(0x808080))
                        .child("选择输入方案")
                )
                .children(items)
        )
        .into_any_element()
}