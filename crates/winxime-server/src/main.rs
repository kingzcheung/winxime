#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ipc_server;
mod tray;
mod ui;

use std::sync::{Arc, atomic::AtomicBool};
use windows::Win32::{
    System::Recovery::{REGISTER_APPLICATION_RESTART_FLAGS, RegisterApplicationRestart},
    UI::HiDpi::{DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext},
};
use winxime_core::SharedInputContext;
use winxime_ipc::{check_server_running, IpcClient};
use winxime_rime::RimeEngine;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|arg| arg == "/q" || arg == "/quit") {
        if check_server_running() {
            IpcClient::shutdown_server();
            #[cfg(debug_assertions)]
            println!("Server stopped");
        }
        return;
    }

    if check_server_running() {
        #[cfg(debug_assertions)]
        println!("Stopping existing server...");
        
        IpcClient::shutdown_server();
        for _ in 0..10 {
            std::thread::sleep(std::time::Duration::from_millis(50));
            if !check_server_running() {
                break;
            }
        }
        if check_server_running() {
            #[cfg(debug_assertions)]
            println!("Failed to stop, exiting");
            return;
        }
    }

    #[cfg(debug_assertions)]
    println!("Starting winxime-server...");

    unsafe {
        let _ = RegisterApplicationRestart(None, REGISTER_APPLICATION_RESTART_FLAGS(0));
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }

    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_dir = manifest_dir.parent().unwrap().parent().unwrap();

    let shared_data_dir = workspace_dir.join("librime").join("data");
    let user_data_dir = workspace_dir.join("rime-data");

    if !shared_data_dir.exists() {
        #[cfg(debug_assertions)]
        eprintln!("Shared data not found!");
        std::process::exit(1);
    }

    let _ = std::fs::create_dir_all(&user_data_dir);

    let engine = match RimeEngine::new(&shared_data_dir, &user_data_dir, "Xime") {
        Ok(mut e) => {
            e.set_option("_horizontal", true);
            #[cfg(debug_assertions)]
            println!("Rime initialized");
            Arc::new(std::sync::Mutex::new(e))
        }
        Err(e) => {
            #[cfg(debug_assertions)]
            eprintln!("Rime init failed: {}", e);
            std::process::exit(1);
        }
    };

    let context = Arc::new(SharedInputContext::new());
    
    let ascii_mode = Arc::new(AtomicBool::new(false));
    
    #[cfg(debug_assertions)]
    println!("Creating UI window...");
    
    let window = ui::CandidateWindow::new();

    #[cfg(debug_assertions)]
    println!("Starting IPC...");
    
    let engine_clone = engine.clone();
    let context_clone = context.clone();
    let window_clone = window.clone();
    let ascii_mode_clone = ascii_mode.clone();
    std::thread::spawn(move || {
        ipc_server::run_ipc_server(engine_clone, context_clone, window_clone, ascii_mode_clone);
    });

    #[cfg(debug_assertions)]
    println!("Creating tray icon...");
    
    let on_action = {
        let engine = engine.clone();
        Arc::new(move |action: tray::TrayAction| {
            match action {
                tray::TrayAction::ToggleAsciiMode => {
                    let mut eng = engine.lock().unwrap();
                    let current = eng.is_ascii_mode();
                    eng.set_option("ascii_mode", !current);
                    tray::update_tray_icon(!current);
                    #[cfg(debug_assertions)]
                    println!("Tray: toggled ascii_mode to {}", !current);
                }
                tray::TrayAction::OpenSettings => {
                    let exe_path = std::env::current_exe().unwrap();
                    let exe_dir = exe_path.parent().unwrap();
                    let setup_path = exe_dir.join("winxime-setup.exe");
                    if setup_path.exists() {
                        std::process::Command::new(&setup_path)
                            .spawn()
                            .ok();
                    }
                    #[cfg(debug_assertions)]
                    println!("Tray: open settings");
                }
                tray::TrayAction::Quit => {
                    IpcClient::shutdown_server();
                    #[cfg(debug_assertions)]
                    println!("Tray: quit");
                }
            }
        })
    };
    
    tray::TrayIcon::new(on_action);
    
    #[cfg(debug_assertions)]
    println!("Server ready");
    
    unsafe {
        let mut msg = windows::Win32::UI::WindowsAndMessaging::MSG::default();
        while windows::Win32::UI::WindowsAndMessaging::GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = windows::Win32::UI::WindowsAndMessaging::TranslateMessage(&msg);
            windows::Win32::UI::WindowsAndMessaging::DispatchMessageW(&msg);
        }
    }
}