mod components;
mod pages;

use gpui::*;
use gpui_platform::application;
use pages::SettingsApp;

fn main() {
    application().run(|cx: &mut App| {
        let _ = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::centered(
                    Size { width: px(800.0), height: px(640.0) },
                    cx,
                )),
                titlebar: Some(TitlebarOptions {
                    title: Some("Xime 设置".into()),
                    appears_transparent: true,
                    traffic_light_position: None,
                }),
                is_resizable: false,
                ..Default::default()
            },
            |_window: &mut Window, cx: &mut App| cx.new(|_| SettingsApp::new()),
        );
    });
}