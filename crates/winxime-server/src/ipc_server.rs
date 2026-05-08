use crate::ui::CandidateWindow;
use interprocess::os::windows::named_pipe::{pipe_mode::Bytes, PipeListenerOptions};
use interprocess::os::windows::security_descriptor::SecurityDescriptor;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use widestring::u16cstr;
use winxime_core::SharedInputContext;
use winxime_ipc::{get_pipe_path, IpcCommand, IpcRequest, IpcRequestData, IpcResponse};
use winxime_rime::RimeEngine;

pub fn run_ipc_server(
    engine: Arc<std::sync::Mutex<RimeEngine>>,
    context: Arc<SharedInputContext>,
    window: Arc<CandidateWindow>,
    ascii_mode: Arc<AtomicBool>,
) {
    let pipe_path = get_pipe_path();
    println!("Winxime Server: creating named pipe at {}", pipe_path);

    let sd = SecurityDescriptor::deserialize(u16cstr!("D:(A;;GA;;;WD)"))
        .expect("Failed to create security descriptor");

    let listener = match PipeListenerOptions::new()
        .path(pipe_path)
        .mode(interprocess::os::windows::named_pipe::PipeMode::Bytes)
        .security_descriptor(Some(sd))
        .create_duplex::<Bytes>()
    {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to create pipe listener: {}", e);
            return;
        }
    };

    println!("Waiting for client connections...");

    for pipe in listener.incoming() {
        match pipe {
            Ok(p) => {
                println!("Client connected!");
                let engine_clone = engine.clone();
                let context_clone = context.clone();
                let window_clone = window.clone();
                let ascii_mode_clone = ascii_mode.clone();
                std::thread::spawn(move || {
                    handle_connection(p, engine_clone, context_clone, window_clone, ascii_mode_clone);
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}

fn handle_connection(
    pipe: interprocess::os::windows::named_pipe::PipeStream<Bytes, Bytes>,
    engine: Arc<std::sync::Mutex<RimeEngine>>,
    context: Arc<SharedInputContext>,
    window: Arc<CandidateWindow>,
    ascii_mode: Arc<AtomicBool>,
) {
    let (recv, send) = pipe.split();
    let mut reader = BufReader::new(recv);
    let mut writer = BufWriter::new(send);

    loop {
        let mut buffer = Vec::new();
        if let Err(_) = reader.read_until(0, &mut buffer) {
            break;
        }

        if buffer.last() != Some(&0) {
            break;
        }
        buffer.pop();

        if buffer.is_empty() {
            continue;
        }

        let request: IpcRequest = match serde_json::from_slice(&buffer) {
            Ok(r) => r,
            Err(_) => continue,
        };
        println!("Received request: {:?}", request.command);

        let response = process_request(&request, &engine, &context, &window, &ascii_mode);

        let json = match serde_json::to_vec(&response) {
            Ok(j) => j,
            Err(_) => continue,
        };
        if writer.write_all(&json).is_err() {
            break;
        }
        if writer.write_all(&[0]).is_err() {
            break;
        }
        if writer.flush().is_err() {
            break;
        }
    }
    println!("Client disconnected");
}

fn process_request(
    request: &IpcRequest,
    engine: &Arc<std::sync::Mutex<RimeEngine>>,
    context: &Arc<SharedInputContext>,
    window: &Arc<CandidateWindow>,
    ascii_mode: &Arc<AtomicBool>,
) -> IpcResponse {
    let mut eng = engine.lock().unwrap();

    match request.command {
        IpcCommand::Echo => IpcResponse {
            success: true,
            session_id: request.session_id,
            context: None,
            status: None,
        },

        IpcCommand::StartSession => {
            println!("StartSession");
            IpcResponse {
                success: true,
                session_id: request.session_id,
                context: None,
                status: Some(get_ipc_status(&eng)),
            }
        }

        IpcCommand::EndSession => {
            println!("EndSession");
            IpcResponse {
                success: true,
                session_id: request.session_id,
                context: None,
                status: None,
            }
        }

        IpcCommand::FocusIn => {
            println!("FocusIn");
            IpcResponse {
                success: true,
                session_id: request.session_id,
                context: None,
                status: Some(get_ipc_status(&eng)),
            }
        }

        IpcCommand::FocusOut => {
            println!("FocusOut -> hide");
            window.hide();
            IpcResponse {
                success: true,
                session_id: request.session_id,
                context: None,
                status: None,
            }
        }

        IpcCommand::ProcessKeyEvent => {
            let handled = match &request.data {
                IpcRequestData::KeyEvent(key) => {
                    println!("Key: {} mod: {}", key.keycode, key.modifiers);
                    let result = eng.process_key(key.keycode, key.modifiers);
                    println!("  handled: {}", result);
                    result
                }
                _ => false,
            };

            let commit = eng.get_commit();
            println!("  commit: {:?}", commit);
            println!("  input: {:?}", eng.get_input());
            println!("  composing: {}", eng.is_composing());

            let ipc_ctx = get_ipc_context(&eng, &commit);
            update_context(&mut eng, context);
            
            if commit.is_some() {
                println!("  -> hide (commit)");
                window.hide();
            } else if !eng.is_composing() {
                println!("  -> hide (not composing)");
                window.hide();
            } else if let Some(ctx) = &ipc_ctx {
                println!("  candies: {:?}", ctx.candidates.candies);
                if ctx.candidates.candies.is_empty() && ctx.preedit.str.is_empty() {
                    println!("  -> hide (empty)");
                    window.hide();
                } else {
                    let pos = context.read(|c| (c.caret_x, c.caret_y));
                    println!("  -> show at ({}, {})", pos.0, pos.1);
                    window.show(pos.0, pos.1);
                    println!("  -> update {} candies", ctx.candidates.candies.len());
                    window.update(ctx);
                }
            }

            IpcResponse {
                success: handled,
                session_id: request.session_id,
                context: ipc_ctx,
                status: Some(get_ipc_status(&eng)),
            }
        }

        IpcCommand::UpdateInputPosition => {
            match &request.data {
                IpcRequestData::Position(pos) => {
                    println!("Position: {},{}", pos.x, pos.y);
                    context.update(|ctx| {
                        ctx.caret_x = pos.x;
                        ctx.caret_y = pos.y;
                    });
                }
                _ => {}
            }

            IpcResponse {
                success: true,
                session_id: request.session_id,
                context: None,
                status: None,
            }
        }

        IpcCommand::ShutdownServer => {
            println!("Shutdown requested");
            std::process::exit(0);
        }

        IpcCommand::ToggleAsciiMode => {
            println!("ToggleAsciiMode requested");
            let current = eng.is_ascii_mode();
            let new_mode = !current;
            println!("  -> current={}, setting to {}", current, new_mode);
            eng.set_option("ascii_mode", new_mode);
            ascii_mode.store(new_mode, Ordering::Release);
            crate::tray::update_tray_icon(new_mode);
            
            IpcResponse {
                success: true,
                session_id: request.session_id,
                context: None,
                status: Some(get_ipc_status(&eng)),
            }
        }

        IpcCommand::ShowTrayIcon => {
            crate::tray::show_icon();
            IpcResponse {
                success: true,
                session_id: request.session_id,
                context: None,
                status: None,
            }
        }

        IpcCommand::HideTrayIcon => {
            crate::tray::hide_icon();
            IpcResponse {
                success: true,
                session_id: request.session_id,
                context: None,
                status: None,
            }
        }

        _ => IpcResponse {
            success: false,
            session_id: request.session_id,
            context: None,
            status: None,
        },
    }
}

fn update_context(eng: &mut RimeEngine, context: &Arc<SharedInputContext>) {
    context.update(|ctx| {
        ctx.is_composing = eng.is_composing();
        ctx.composition.preedit = eng.get_input().unwrap_or_default();
        ctx.commit_text = eng.get_commit().unwrap_or_default();

        let cand_list = eng.get_candidates();
        ctx.candidates = cand_list
            .candidates
            .iter()
            .map(|c| winxime_core::CandidateInfo {
                text: c.text.clone(),
                comment: c.comment.clone().unwrap_or_default(),
            })
            .collect();
    });
}

fn get_ipc_status(eng: &RimeEngine) -> winxime_ipc::Status {
    let status = eng.get_status();
    winxime_ipc::Status {
        composing: eng.is_composing(),
        ascii_mode: status.as_ref().map(|s| s.is_ascii_mode).unwrap_or(false),
        schema_id: status.as_ref().map(|s| s.schema_id.clone()).unwrap_or_default(),
        schema_name: status.as_ref().map(|s| s.schema_name.clone()).unwrap_or_default(),
    }
}

fn get_ipc_context(eng: &RimeEngine, commit: &Option<String>) -> Option<winxime_ipc::Context> {
    let composing = eng.is_composing();

    if !composing && commit.is_none() {
        return None;
    }

    let cand_list = eng.get_candidates();

    Some(winxime_ipc::Context {
        preedit: winxime_ipc::Text {
            str: eng.get_input().unwrap_or_default(),
        },
        commit: commit.clone(),
        candidates: winxime_ipc::CandidateInfo {
            current_page: cand_list.page_no as u32,
            total_pages: (if cand_list.is_last_page { cand_list.page_no + 1 } else { cand_list.page_no + 2 }) as u32,
            highlighted: cand_list.highlighted,
            is_last_page: cand_list.is_last_page,
            candies: cand_list
                .candidates
                .iter()
                .map(|c| winxime_ipc::Text {
                    str: c.text.clone(),
                })
                .collect(),
            comments: cand_list
                .candidates
                .iter()
                .map(|c| winxime_ipc::Text {
                    str: c.comment.clone().unwrap_or_default(),
                })
                .collect(),
            labels: Vec::new(),
        },
    })
}
