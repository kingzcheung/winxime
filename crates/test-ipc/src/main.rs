use winxime_ipc::{IpcClient, IpcRequest, IpcCommand, IpcRequestData};

fn main() {
    println!("Connecting to IPC server...");
    
    let mut client = match IpcClient::connect() {
        Ok(c) => {
            println!("Connected!");
            c
        },
        Err(e) => {
            eprintln!("Connection failed: {:?}", e);
            return;
        }
    };
    
    println!("Sending StartSession...");
    let request = IpcRequest {
        command: IpcCommand::StartSession,
        session_id: 0,
        data: IpcRequestData::None,
    };
    
    match client.send_request(&request) {
        Ok(response) => {
            println!("Response: {:?}", response);
        },
        Err(e) => {
            eprintln!("Request failed: {:?}", e);
        }
    }
    
    println!("Sending ProcessKeyEvent (keycode=97, modifiers=0)...");
    let request = IpcRequest {
        command: IpcCommand::ProcessKeyEvent,
        session_id: 1,
        data: IpcRequestData::KeyEvent(winxime_ipc::KeyEventData {
            keycode: 97,
            modifiers: 0,
        }),
    };
    
    match client.send_request(&request) {
        Ok(response) => {
            println!("Response: {:?}", response);
        },
        Err(e) => {
            eprintln!("Request failed: {:?}", e);
        }
    }
    
    println!("Test complete");
}