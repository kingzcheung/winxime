use gpui::{ParentElement, IntoElement, *};
use crate::components::{Switch, NumberInput};

fn primary() -> Hsla {
    hsla(0.63, 0.65, 0.67, 1.0)
}

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
                .child("外观")
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(12.0))
                .p(px(20.0))
                .rounded(px(16.0))
                .bg(hsla(0.0, 0.0, 0.1, 0.6))
                .border_1()
                .border_color(rgb(0x303030))
                .child(render_item("字体大小", "候选栏字体大小", NumberInput::new(18.0)))
                .child(render_item("候选数量", "候选栏显示的候选词数量", NumberInput::new(5.0)))
                .child(render_switch_item("显示编码提示", "在候选词旁显示编码", true))
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(12.0))
                .p(px(20.0))
                .rounded(px(16.0))
                .bg(hsla(0.0, 0.0, 0.1, 0.6))
                .border_1()
                .border_color(rgb(0x303030))
                .child(render_item("圆角大小", "候选栏窗口圆角", NumberInput::new(8.0)))
        )
        .into_any_element()
}

fn render_item(label: impl Into<String>, desc: impl Into<String>, control: impl IntoElement) -> Div {
    let label = label.into();
    let desc = desc.into();
    div()
        .flex()
        .items_center()
        .justify_between()
        .py(px(12.0))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(
                    div()
                        .text_size(px(14.0))
                        .text_color(rgb(0xe0e0e0))
                        .child(label)
                )
                .child(
                    div()
                        .text_size(px(12.0))
                        .text_color(rgb(0x808080))
                        .child(desc)
                )
        )
        .child(control)
}

fn render_switch_item(label: impl Into<String>, desc: impl Into<String>, checked: bool) -> Div {
    let label = label.into();
    let desc = desc.into();
    div()
        .flex()
        .items_center()
        .justify_between()
        .py(px(12.0))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(
                    div()
                        .text_size(px(14.0))
                        .text_color(rgb(0xe0e0e0))
                        .child(label)
                )
                .child(
                    div()
                        .text_size(px(12.0))
                        .text_color(rgb(0x808080))
                        .child(desc)
                )
        )
        .child(Switch::new(checked))
}