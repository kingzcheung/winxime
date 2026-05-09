mod pages;

use gpui::*;
use gpui_component::Root;

fn main() {
    let app = gpui_platform::application().with_assets(gpui_component_assets::Assets);

    app.run(|cx| {
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), |window, cx| {
                let view = cx.new(|_| pages::SettingsApp::new());
                cx.new(|cx| Root::new(view.into(), window, cx))
            })
        })
        .detach();
    });
}