pub mod input_schema;
pub mod appearance;
pub mod clipboard;
pub mod hotkeys;
pub mod smart_suggestion;
pub mod dictionary;

use gpui::{prelude::FluentBuilder, ParentElement, IntoElement, *};
use crate::components::{TitleBar};

pub struct SettingsApp {
    current_page: usize,
}

impl SettingsApp {
    pub fn new() -> Self {
        Self { 
            current_page: 0,
        }
    }
}

impl Render for SettingsApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_background_appearance(WindowBackgroundAppearance::Blurred);
        
        let pages = ["输入方案", "外观", "剪切板", "快捷键", "智能联想", "词库管理"];
        let current = self.current_page;
        
        let sidebar = div()
            .w(px(213.0))
            .h_full()
            .bg(rgb(0x0d0d0d))
            .flex()
            .flex_col()
            .gap(px(2.0))
            .p(px(8.0))
            .children(
                pages
                    .iter()
                    .enumerate()
                    .map(|(i, name)| {
                        let is_current = i == current;
                        let view = cx.entity();
                        div()
                            .id(("menu", i))
                            .py(px(10.0))
                            .px(px(12.0))
                            .rounded(px(8.0))
                            .when(is_current, |this: Stateful<Div>| this.bg(rgb(0x2d2d2d)))
                            .when(!is_current, |this: Stateful<Div>| {
                                this.cursor_pointer()
                                    .hover(|style: StyleRefinement| style.bg(rgb(0x1a1a1a)))
                            })
                            .text_size(px(13.0))
                            .text_color(if is_current { rgb(0xe0e0e0) } else { rgb(0x808080) })
                            .on_click(move |_, window: &mut Window, cx: &mut App| {
                                cx.update_entity(&view, |app: &mut SettingsApp, cx: &mut Context<SettingsApp>| {
                                    app.current_page = i;
                                    cx.notify();
                                });
                            })
                            .child(name.to_string())
                    })
            );

        let content = match self.current_page {
            0 => input_schema::render(),
            1 => appearance::render(),
            2 => clipboard::render(),
            3 => hotkeys::render(),
            4 => smart_suggestion::render(),
            5 => dictionary::render(),
            _ => input_schema::render(),
        };

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(hsla(0.0, 0.0, 0.05, 0.85))
            .child(TitleBar::render(window))
            .child(
                div()
                    .id("content-area")
                    .flex()
                    .flex_1()
                    .h_full()
                    .overflow_hidden()
                    .child(sidebar)
                    .child(
                        div()
                            .id("content-scroll")
                            .flex_1()
                            .overflow_y_scroll()
                            .child(content)
                    )
            )
    }
}