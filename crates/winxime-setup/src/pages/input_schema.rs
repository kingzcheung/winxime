use gpui::{prelude::FluentBuilder, ParentElement, IntoElement, *};
use crate::components::{Radio};
use crate::state::SettingsState;
use crate::pages::SettingsApp;

pub fn render(settings: Entity<SettingsState>, cx: &mut Context<SettingsApp>) -> AnyElement {
    let schemas_loaded = cx.read_entity(&settings, |state, _| state.schemas_loaded);
    
    if !schemas_loaded {
        cx.update_entity(&settings, |state: &mut SettingsState, cx| {
            if let Ok(manager) = crate::rime_config::SchemaManager::new() {
                let schemas = manager.get_schema_list();
                let selected_schema = manager.get_selected_schema()
                    .and_then(|selected_id| {
                        schemas.iter().position(|s| s.schema_id == selected_id)
                    })
                    .unwrap_or(0);
                state.input_schema.available_schemas = schemas;
                state.input_schema.selected_schema = selected_schema;
                state.schemas_loaded = true;
                cx.notify();
            }
        });
    }
    
    let (selected, schemas, colors) = cx.read_entity(&settings, |state, _| {
        (state.input_schema.selected_schema, state.input_schema.available_schemas.clone(), state.colors())
    });
    
    let settings_clone = settings.clone();
    
    let schema_items: Vec<AnyElement> = schemas
        .iter()
        .enumerate()
        .map(|(i, schema)| {
            let is_selected = i == selected;
            let settings_item = settings_clone.clone();
            let primary = colors.primary.clone();
            let surface_variant = colors.surface_variant.clone();
            let radio_colors = colors.clone();
            
            div()
                .id(("schema", i))
                .flex()
                .items_center()
                .gap(px(12.0))
                .py(px(8.0))
                .px(px(12.0))
                .rounded(px(8.0))
                .cursor_pointer()
                .hover(|style| style.bg(surface_variant))
                .when(is_selected, |this| {
                    this.border_1()
                        .border_color(primary.clone())
                        .bg(colors.selection)
                })
                .on_click(move |_event, _window, cx| {
                    settings_item.update(cx, |s: &mut SettingsState, cx| {
                        s.input_schema.selected_schema = i;
                        if let Err(e) = s.save_schema() {
                            eprintln!("Auto-save schema failed: {}", e);
                        }
                        cx.notify();
                    });
                })
                .child(Radio::new(is_selected).theme(radio_colors))
                .child(
                    div()
                        .text_size(px(14.0))
                        .text_color(if is_selected { primary } else { colors.foreground })
                        .child(schema.name.clone())
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
                        .child("选择默认输入方案")
                )
                .when(schemas.is_empty(), |this| {
                    this.child(
                        div()
                            .text_size(px(14.0))
                            .text_color(colors.foreground_muted)
                            .child("未找到输入方案，请先部署")
                    )
                })
                .when(!schemas.is_empty(), |this| this.children(schema_items))
        )
        .into_any_element()
}