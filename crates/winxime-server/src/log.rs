use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;
use time::OffsetDateTime;

static LOG_FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);

fn get_log_path() -> Option<std::path::PathBuf> {
    if let Ok(temp) = std::env::var("TEMP") {
        return Some(std::path::PathBuf::from(temp).join("winxime-server.log"));
    }
    Some(std::path::PathBuf::from("winxime-server.log"))
}

pub fn init_log() {
    if let Ok(mut guard) = LOG_FILE.lock() {
        if guard.is_none() {
            if let Some(path) = get_log_path() {
                if let Ok(file) = OpenOptions::new().create(true).append(true).open(&path) {
                    *guard = Some(file);
                }
            }
        }
    }
}

pub fn log(msg: &str) {
    if let Ok(mut guard) = LOG_FILE.lock() {
        if let Some(ref mut file) = guard.as_mut() {
            let now = OffsetDateTime::now_utc();
            let time_str = format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                now.year(),
                now.month() as u8,
                now.day(),
                now.hour(),
                now.minute(),
                now.second()
            );
            let _ = writeln!(file, "[{}] {}", time_str, msg);
            let _ = file.flush();
        }
    }
}