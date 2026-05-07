use interprocess::os::windows::named_pipe::{DuplexPipeStream, pipe_mode::Bytes};
use std::time::Duration;
use std::thread;

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
        
        // Quick connect - no retry for TSF (must not block UI)
        match DuplexPipeStream::connect_by_path(pipe_path.clone()) {
            Ok(pipe) => Ok(Self { pipe }),
            Err(_) => Err(IpcError::ConnectionFailed),
        }
    }
    
    pub fn send_request(&mut self, request: &crate::IpcRequest) -> Result<crate::IpcResponse, IpcError> {
        use std::io::{Write, BufReader, BufRead};
        
        let json = serde_json::to_vec(request).map_err(|_| IpcError::SerializeFailed)?;
        self.pipe.write_all(&json).map_err(|_| IpcError::WriteFailed)?;
        self.pipe.write_all(&[0]).map_err(|_| IpcError::WriteFailed)?;
        self.pipe.flush().map_err(|_| IpcError::WriteFailed)?;
        
        let mut reader = BufReader::new(&self.pipe);
        let mut response_buf = Vec::new();
        reader.read_until(0, &mut response_buf).map_err(|_| IpcError::ReadFailed)?;
        
        // Remove trailing null byte
        if response_buf.last() == Some(&0) {
            response_buf.pop();
        }
        
        serde_json::from_slice(&response_buf).map_err(|_| IpcError::DeserializeFailed)
    }
    
    pub fn send_oneway(&mut self, request: &crate::IpcRequest) -> Result<(), IpcError> {
        use std::io::Write;
        
        let json = serde_json::to_vec(request).map_err(|_| IpcError::SerializeFailed)?;
        self.pipe.write_all(&json).map_err(|_| IpcError::WriteFailed)?;
        self.pipe.write_all(&[0]).map_err(|_| IpcError::WriteFailed)?;
        self.pipe.flush().ok();
        Ok(())
    }
}
