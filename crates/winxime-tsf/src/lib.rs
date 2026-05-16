mod class_factory;
mod dll;
mod language_bar;
mod text_input_processor;

pub use class_factory::ClassFactory;
pub use language_bar::{LangBarItemButton, set_instance};
pub use text_input_processor::XimeTextService;

#[macro_export]
macro_rules! tsf_debug {
    ($($arg:tt)*) => {
        {
            if let Ok(temp_path) = std::env::var("TEMP") {
                let dir_path = std::path::PathBuf::from(&temp_path).join("winxime");
                let log_path = dir_path.join("tsf.log");
                let _ = std::fs::create_dir_all(&dir_path);
                if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&log_path) {
                    use std::io::Write;
                    let msg = format!("{} TSF {}\n", chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f"), format_args!($($arg)*));
                    let _ = file.write_all(msg.as_bytes());
                    let _ = file.flush();
                }
            }
        }
    };
}
