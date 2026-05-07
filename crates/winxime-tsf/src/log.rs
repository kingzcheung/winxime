use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;

static LOG_FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);

fn get_log_path() -> Option<std::path::PathBuf> {
    // Try to use user's temp directory
    if let Ok(temp) = std::env::var("TEMP") {
        let path = std::path::PathBuf::from(temp).join("winxime_tsf.log");
        return Some(path);
    }
    // Fallback to current directory
    Some(std::path::PathBuf::from("winxime_tsf.log"))
}

pub fn init_log() {
    let mut guard = LOG_FILE.lock().unwrap();
    if guard.is_none() {
        if let Some(path) = get_log_path() {
            if let Ok(file) = OpenOptions::new().create(true).append(true).open(&path) {
                *guard = Some(file);
            }
        }
    }
}

pub fn log(msg: &str) {
    let mut guard = LOG_FILE.lock().unwrap();
    if let Some(ref mut file) = guard.as_mut() {
        let _ = writeln!(
            file,
            "[{}] {}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            msg
        );
        let _ = file.flush();
    }
}
