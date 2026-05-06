use std::path::PathBuf;

fn get_data_dir() -> PathBuf {
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    base.join("Xime")
}

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_dir = manifest_dir.parent().unwrap().parent().unwrap();

    let shared_data_dir = workspace_dir.join("librime").join("data").join("minimal");
    let user_data_dir = if cfg!(debug_assertions) {
        workspace_dir.join("rime-data")
    } else {
        get_data_dir()
    };

    let _ = std::fs::create_dir_all(&user_data_dir);

    match winxime_rime::RimeEngine::new(&shared_data_dir, &user_data_dir, "Xime") {
        Ok(_engine) => {
            println!("Xime - Rime engine initialized");
            println!("Shared data: {}", shared_data_dir.display());
            println!("User data: {}", user_data_dir.display());
            println!("Xime 五笔输入法 ready");
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
        Err(e) => {
            eprintln!("Failed to initialize Rime: {}", e);
            std::process::exit(1);
        }
    }
}
