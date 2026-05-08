use interprocess::os::windows::named_pipe::{pipe_mode::Bytes, DuplexPipeStream};
use std::io::{Read, Write};

#[derive(Debug)]
pub enum IpcError {
    ConnectionFailed,
    SerializeFailed,
    DeserializeFailed,
    WriteFailed,
    ReadFailed,
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
        IpcClient::shutdown_server();
        true
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
            Err(_) => Err(IpcError::ConnectionFailed),
        }
    }

    pub fn send_request(
        &mut self,
        request: &crate::IpcRequest,
    ) -> Result<crate::IpcResponse, IpcError> {
        let json = serde_json::to_vec(request).map_err(|_| IpcError::SerializeFailed)?;

        self.pipe
            .write_all(&json)
            .map_err(|_| IpcError::WriteFailed)?;
        self.pipe
            .write_all(&[0])
            .map_err(|_| IpcError::WriteFailed)?;
        self.pipe.flush().map_err(|_| IpcError::WriteFailed)?;

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
                Err(_) => return Err(IpcError::ReadFailed),
            }
        }

        if response_buf.is_empty() {
            return Err(IpcError::ReadFailed);
        }

        serde_json::from_slice(&response_buf).map_err(|_| IpcError::DeserializeFailed)
    }

    pub fn send_oneway(&mut self, request: &crate::IpcRequest) -> Result<(), IpcError> {
        let json = serde_json::to_vec(request).map_err(|_| IpcError::SerializeFailed)?;
        self.pipe
            .write_all(&json)
            .map_err(|_| IpcError::WriteFailed)?;
        self.pipe
            .write_all(&[0])
            .map_err(|_| IpcError::WriteFailed)?;
        self.pipe.flush().map_err(|_| IpcError::WriteFailed)?;

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
}
