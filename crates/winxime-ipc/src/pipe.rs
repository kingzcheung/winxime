use interprocess::os::windows::named_pipe::{DuplexPipeStream, pipe_mode::Bytes};
use std::io::{Read, Write};

#[derive(Debug)]
pub enum IpcError {
    ConnectionFailed,
    SerializeFailed,
    DeserializeFailed,
    WriteFailed,
    ReadFailed,
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
    
    pub fn send_request(&mut self, request: &crate::IpcRequest) -> Result<crate::IpcResponse, IpcError> {
        let json = serde_json::to_vec(request).map_err(|_| IpcError::SerializeFailed)?;
        
        // Write request
        self.pipe.write_all(&json).map_err(|_| IpcError::WriteFailed)?;
        self.pipe.write_all(&[0]).map_err(|_| IpcError::WriteFailed)?;
        self.pipe.flush().map_err(|_| IpcError::WriteFailed)?;
        
        // Read response directly from pipe (no BufReader to avoid buffering issues)
        let mut response_buf = Vec::new();
        let mut byte = [0u8; 1];
        
        loop {
            match self.pipe.read(&mut byte) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    if byte[0] == 0 {
                        break; // Null terminator
                    }
                    response_buf.push(byte[0]);
                },
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
        self.pipe.write_all(&json).map_err(|_| IpcError::WriteFailed)?;
        self.pipe.write_all(&[0]).map_err(|_| IpcError::WriteFailed)?;
        self.pipe.flush().map_err(|_| IpcError::WriteFailed)?;
        Ok(())
    }
}