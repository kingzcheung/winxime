slint::include_modules!();

use slint::ComponentHandle;

fn main() {
    let settings = SettingsWindow::new().unwrap();
    
    settings.set_show_comment(true);
    settings.set_font_size("18".into());
    settings.set_candidate_count("5".into());
    settings.set_theme("default".into());
    
    settings.on_save_clicked({
        let settings_weak = settings.as_weak();
        move || {
            let settings = settings_weak.unwrap();
            let font_size = settings.get_font_size();
            let candidate_count = settings.get_candidate_count();
            let show_comment = settings.get_show_comment();
            let theme = settings.get_theme();
            
            println!("保存设置:");
            println!("  字体大小: {}", font_size);
            println!("  候选数量: {}", candidate_count);
            println!("  显示注释: {}", show_comment);
            println!("  配色主题: {}", theme);
            
            slint::quit_event_loop().unwrap();
        }
    });
    
    settings.on_cancel_clicked({
        move || {
            println!("取消设置");
            slint::quit_event_loop().unwrap();
        }
    });
    
    settings.run().unwrap();
}