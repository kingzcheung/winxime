#![windows_subsystem = "console"]

mod ipc_server;
mod ui;

use std::sync::Arc;
use winxime_core::SharedInputContext;
use winxime_rime::RimeEngine;

fn main() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_dir = manifest_dir.parent().unwrap().parent().unwrap();
    
    let shared_data_dir = workspace_dir.join("librime").join("data");
    let user_data_dir = workspace_dir.join("rime-data");
    
    println!("Shared data: {}", shared_data_dir.display());
    println!("User data: {}", user_data_dir.display());
    
    if !shared_data_dir.exists() {
        eprintln!("Shared data directory does not exist!");
        std::process::exit(1);
    }
    
    let _ = std::fs::create_dir_all(&user_data_dir);
    
    let engine = match RimeEngine::new(&shared_data_dir, &user_data_dir, "Xime") {
        Ok(e) => {
            println!("Rime engine initialized successfully");
            let schemas = e.get_schema_list();
            println!("Available schemas: {:?}", schemas);
            if let Some(schema) = e.get_current_schema() {
                println!("Current schema: {}", schema);
            } else {
                println!("Warning: No current schema selected!");
            }
            if e.is_composing() {
                println!("Warning: Engine is composing at init!");
            }
            Arc::new(std::sync::Mutex::new(e))
        },
        Err(e) => {
            eprintln!("Failed to initialize Rime: {}", e);
            std::process::exit(1);
        }
    };
    
    let context = Arc::new(SharedInputContext::new());
    let window = ui::CandidateWindow::new();
    
    println!("Starting IPC server...");
    let engine_clone = engine.clone();
    let context_clone = context.clone();
    let window_clone = window.clone();
    std::thread::spawn(move || {
        ipc_server::run_ipc_server(engine_clone, context_clone, window_clone);
    });
    
    // Message loop for UI
    println!("Running UI message loop...");
    unsafe {
        let mut msg = windows::Win32::UI::WindowsAndMessaging::MSG::default();
        while windows::Win32::UI::WindowsAndMessaging::GetMessageW(&mut msg, None, 0, 0).as_bool() {
            windows::Win32::UI::WindowsAndMessaging::TranslateMessage(&msg);
            windows::Win32::UI::WindowsAndMessaging::DispatchMessageW(&msg);
        }
    }
}