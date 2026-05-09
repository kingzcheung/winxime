use gpui::{prelude::FluentBuilder, ParentElement, IntoElement, *};
use crate::components::{Radio};

pub fn render() -> AnyElement {
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
                .children(vec![
                    render_radio_item("五笔86极点", 0, true),
                    render_radio_item("五笔86", 1, false),
                    render_radio_item("五笔98", 2, false),
                ])
        )
        .into_any_element()
}

fn render_radio_item(label: impl Into<String>, index: usize, checked: bool) -> impl IntoElement {
    let label = label.into();
    let primary = rgb(0x8F73E2);
    div()
        .id(("schema", index))
        .flex()
        .items_center()
        .gap(px(12.0))
        .py(px(8.0))
        .px(px(12.0))
        .rounded(px(8.0))
        .cursor_pointer()
        .hover(|style: StyleRefinement| style.bg(rgb(0x262626)))
        .when(checked, |this: Stateful<Div>| {
            this.border_1()
                .border_color(primary)
                .bg(rgb(0x3d2d5d))
        })
        .child(Radio::new(checked))
        .child(
            div()
                .text_size(px(14.0))
                .text_color(if checked { primary } else { rgb(0xe6e6e6) })
                .child(label)
        )
}