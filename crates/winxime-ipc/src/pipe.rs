use interprocess::os::windows::named_pipe::{pipe_mode::Bytes, DuplexPipeStream};
use std::io::{Read, Write};

#[derive(Debug)]
pub enum IpcError {
    ConnectionFailed(String),
    SerializeFailed(String),
    DeserializeFailed(String),
    WriteFailed(String),
    ReadFailed(String),
    EmptyResponse,
}

impl std::fmt::Display for IpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpcError::ConnectionFailed(s) => write!(f, "ConnectionFailed: {}", s),
            IpcError::SerializeFailed(s) => write!(f, "SerializeFailed: {}", s),
            IpcError::DeserializeFailed(s) => write!(f, "DeserializeFailed: {}", s),
            IpcError::WriteFailed(s) => write!(f, "WriteFailed: {}", s),
            IpcError::ReadFailed(s) => write!(f, "ReadFailed: {}", s),
            IpcError::EmptyResponse => write!(f, "EmptyResponse"),
        }
    }
}

pub fn check_server_running() -> bool {
    let pipe_path = crate::messages::get_pipe_path();
    match DuplexPipeStream::<Bytes>::connect_by_path(pipe_path) {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn stop_server() -> bool {
    if check_server_running() {
        IpcClient::shutdown_server()
    } else {
        false
    }
}

pub struct IpcClient {
    pipe: DuplexPipeStream<Bytes>,
}

impl IpcClient {
    pub fn connect() -> Result<Self, IpcError> {
        let pipe_path = crate::messages::get_pipe_path();

        match DuplexPipeStream::connect_by_path(pipe_path) {
            Ok(pipe) => Ok(Self { pipe }),
            Err(e) => Err(IpcError::ConnectionFailed(format!("{:?}", e))),
        }
    }

    pub fn send_request(
        &mut self,
        request: &crate::IpcRequest,
    ) -> Result<crate::IpcResponse, IpcError> {
        let json = serde_json::to_vec(request).map_err(|e| IpcError::SerializeFailed(format!("{:?}", e)))?;

        self.pipe
            .write_all(&json)
            .map_err(|e| IpcError::WriteFailed(format!("write_all json: {:?}", e)))?;
        self.pipe
            .write_all(&[0])
            .map_err(|e| IpcError::WriteFailed(format!("write_all terminator: {:?}", e)))?;
        self.pipe.flush().map_err(|e| IpcError::WriteFailed(format!("flush: {:?}", e)))?;

        let mut response_buf = Vec::new();
        let mut byte = [0u8; 1];

        loop {
            match self.pipe.read(&mut byte) {
                Ok(0) => break,
                Ok(_) => {
                    if byte[0] == 0 {
                        break;
                    }
                    response_buf.push(byte[0]);
                }
                Err(e) => return Err(IpcError::ReadFailed(format!("{:?}", e))),
            }
        }

        if response_buf.is_empty() {
            return Err(IpcError::EmptyResponse);
        }

        serde_json::from_slice(&response_buf).map_err(|e| IpcError::DeserializeFailed(format!("{:?}", e)))
    }

    pub fn send_oneway(&mut self, request: &crate::IpcRequest) -> Result<(), IpcError> {
        let json = serde_json::to_vec(request).map_err(|e| IpcError::SerializeFailed(format!("{:?}", e)))?;
        self.pipe
            .write_all(&json)
            .map_err(|e| IpcError::WriteFailed(format!("{:?}", e)))?;
        self.pipe
            .write_all(&[0])
            .map_err(|e| IpcError::WriteFailed(format!("{:?}", e)))?;
        self.pipe.flush().map_err(|e| IpcError::WriteFailed(format!("{:?}", e)))?;

        let mut response_buf = Vec::new();
        let mut byte = [0u8; 1];
        loop {
            match self.pipe.read(&mut byte) {
                Ok(0) => break,
                Ok(_) => {
                    if byte[0] == 0 {
                        break;
                    }
                    response_buf.push(byte[0]);
                }
                Err(_) => break,
            }
        }
        Ok(())
    }

    pub fn shutdown_server() -> bool {
        if let Ok(mut client) = Self::connect() {
            let request = crate::IpcRequest {
                command: crate::IpcCommand::ShutdownServer,
                session_id: 0,
                data: crate::IpcRequestData::None,
            };
            client.send_oneway(&request).is_ok()
        } else {
            false
        }
    }

    pub fn reload_config() -> bool {
        if let Ok(mut client) = Self::connect() {
            let request = crate::IpcRequest {
                command: crate::IpcCommand::ReloadConfig,
                session_id: 0,
                data: crate::IpcRequestData::None,
            };
            match client.send_request(&request) {
                Ok(response) => response.success,
                Err(_) => false,
            }
        } else {
            false
        }
    }
}
