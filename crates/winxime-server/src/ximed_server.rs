use std::thread;
use tracing::info;

const DEFAULT_PORT: u16 = 8370;

pub fn start() {
    thread::spawn(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build tokio runtime for clipboard server");
        rt.block_on(async {
            info!("Starting clipboard sharing server on port {}", DEFAULT_PORT);
            if let Err(e) = ximed::serve(DEFAULT_PORT).await {
                tracing::error!("Clipboard sharing server failed: {}", e);
            }
        });
    });
}
