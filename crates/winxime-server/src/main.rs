#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod context;
mod ipc_server;
mod tray;
mod ui;

use crate::config::UiConfig;
use crate::context::SharedInputContext;
use std::sync::{atomic::AtomicBool, Arc};
use tracing::info;
use windows::Win32::{
    System::Recovery::{RegisterApplicationRestart, REGISTER_APPLICATION_RESTART_FLAGS},
    UI::HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2},
};
use winxime_config::init_logging_with_console;
use winxime_ipc::{check_server_running, IpcClient};
use winxime_rime::RimeEngine;

fn main() {
    init_logging_with_console("server");
    info!("Server starting");
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|arg| arg == "/q" || arg == "/quit") {
        if check_server_running() {
            IpcClient::shutdown_server();
        }
        info!("Server stopped via /q");
        return;
    }

    if check_server_running() {
        info!("Stopping existing server...");
        IpcClient::shutdown_server();
        for _ in 0..10 {
            std::thread::sleep(std::time::Duration::from_millis(50));
            if !check_server_running() {
                break;
            }
        }
        if check_server_running() {
            info!("Failed to stop existing server, exiting");
            return;
        }
        info!("Existing server stopped");
    }

    unsafe {
        let _ = RegisterApplicationRestart(None, REGISTER_APPLICATION_RESTART_FLAGS(0));
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }

    let (shared_data_dir, user_data_dir) = get_data_dirs();
    info!(
        "Data dirs: shared={}, user={}",
        shared_data_dir.display(),
        user_data_dir.display()
    );

    if !shared_data_dir.exists() {
        info!("Shared data not found at {:?}", shared_data_dir);
        std::process::exit(1);
    }
    info!("Shared data dir exists");

    let _ = std::fs::create_dir_all(&user_data_dir);
    ensure_user_config_files(&user_data_dir);

    info!("Initializing Rime engine...");
    let ui_config = UiConfig::load();
    let engine = match RimeEngine::new(&shared_data_dir, &user_data_dir, "Xime") {
        Ok(mut e) => {
            e.set_option("_horizontal", ui_config.horizontal);
            info!(
                "Rime initialized successfully with horizontal={}",
                ui_config.horizontal
            );
            Arc::new(std::sync::Mutex::new(e))
        }
        Err(e) => {
            info!("Rime init failed: {}", e);
            std::process::exit(1);
        }
    };

    run_server(engine);
}

fn get_data_dirs() -> (std::path::PathBuf, std::path::PathBuf) {
    #[cfg(debug_assertions)]
    {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_dir = manifest_dir.parent().unwrap().parent().unwrap();
        (
            workspace_dir.join("librime").join("data").join("minimal"),
            workspace_dir.join("target").join("debug").join("user-data"),
        )
    }

    #[cfg(not(debug_assertions))]
    {
        let exe_path = std::env::current_exe().ok().unwrap_or_else(|| {
            std::path::PathBuf::from("C:\\Program Files\\Xime\\winxime-server.exe")
        });
        let exe_dir = exe_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("C:\\Program Files\\Xime"));

        let user_data_dir = std::env::var("APPDATA")
            .ok()
            .map(|p| std::path::PathBuf::from(p).join("Xime").join("rime"))
            .unwrap_or_else(|| exe_dir.join("user-data"));

        (exe_dir.join("data"), user_data_dir)
    }
}

fn get_config_source_dir() -> std::path::PathBuf {
    #[cfg(debug_assertions)]
    {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_dir = manifest_dir.parent().unwrap().parent().unwrap();
        workspace_dir.join("resources")
    }

    #[cfg(not(debug_assertions))]
    {
        let exe_path = std::env::current_exe().ok().unwrap_or_else(|| {
            std::path::PathBuf::from("C:\\Program Files\\Xime\\winxime-server.exe")
        });
        exe_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("C:\\Program Files\\Xime"))
            .join("resources")
    }
}

fn ensure_user_config_files(user_data_dir: &std::path::Path) {
    if !user_data_dir.exists() {
        std::fs::create_dir_all(user_data_dir).ok();
    }

    let config_source_dir = get_config_source_dir();

    let xime_yaml = user_data_dir.join("xime.yaml");
    if !xime_yaml.exists() {
        let source = config_source_dir.join("xime.yaml");
        if source.exists() {
            std::fs::copy(&source, &xime_yaml).ok();
        }
    }
}

fn run_server(engine: Arc<std::sync::Mutex<RimeEngine>>) {
    info!("run_server: starting");
    let context = Arc::new(SharedInputContext::new());
    let ascii_mode = Arc::new(AtomicBool::new(false));

    info!("Creating UI window...");
    let window = ui::CandidateWindow::new();
    info!("UI window created");

    info!("Starting IPC thread...");
    let engine_clone = engine.clone();
    let context_clone = context.clone();
    let window_clone = window.clone();
    let ascii_mode_clone = ascii_mode.clone();
    std::thread::spawn(move || {
        ipc_server::run_ipc_server(engine_clone, context_clone, window_clone, ascii_mode_clone);
    });
    info!("IPC thread started");

    info!("Creating tray icon...");
    let on_action = {
        let engine = engine.clone();
        Arc::new(move |action: tray::TrayAction| match action {
            tray::TrayAction::ToggleAsciiMode => {
                let mut eng = engine.lock().unwrap();
                let current = eng.is_ascii_mode();
                eng.set_option("ascii_mode", !current);
                tray::update_tray_icon(!current);
            }
            tray::TrayAction::OpenSettings => {
                let exe_path = std::env::current_exe().ok().unwrap_or_else(|| {
                    std::path::PathBuf::from(
                        "C:\\Program Files\\winxime-server\\winxime-server.exe",
                    )
                });
                let exe_dir = exe_path
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new("C:\\Program Files\\winxime-server"));
                let setup_path = exe_dir.join("winxime-setup.exe");
                if setup_path.exists() {
                    std::process::Command::new(&setup_path).spawn().ok();
                }
            }
            tray::TrayAction::About => {
                let exe_path = std::env::current_exe().ok().unwrap_or_else(|| {
                    std::path::PathBuf::from(
                        "C:\\Program Files\\winxime-server\\winxime-server.exe",
                    )
                });
                let exe_dir = exe_path
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new("C:\\Program Files\\winxime-server"));
                let setup_path = exe_dir.join("winxime-setup.exe");
                let _ = std::process::Command::new(&setup_path)
                    .arg("--about")
                    .spawn();
            }
            tray::TrayAction::Feedback => {
                let _ = std::process::Command::new("cmd")
                    .args([
                        "/C",
                        "start",
                        "https://github.com/kingzcheung/winxime/issues",
                    ])
                    .spawn();
            }
            tray::TrayAction::Quit => {
                IpcClient::shutdown_server();
            }
        })
    };

    tray::TrayIcon::new(on_action);
    info!("Tray icon created");

    info!("Server ready, entering message loop");

    unsafe {
        let mut msg = windows::Win32::UI::WindowsAndMessaging::MSG::default();
        while windows::Win32::UI::WindowsAndMessaging::GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = windows::Win32::UI::WindowsAndMessaging::TranslateMessage(&msg);
            windows::Win32::UI::WindowsAndMessaging::DispatchMessageW(&msg);
        }
    }
    info!("Message loop exited");
}
