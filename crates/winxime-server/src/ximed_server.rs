use std::thread;
use tracing::info;
use winxime_config::XimeConfig;

const DEFAULT_PORT: u16 = 8370;

pub fn start() {
    thread::spawn(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build tokio runtime for clipboard server");
        rt.block_on(async {
            info!("Starting clipboard sharing server on port {}", DEFAULT_PORT);

            // 从配置中读取持久化的 pair_secret（base64 编码）
            let config = XimeConfig::load();
            let secret_b64 = if config.pair_secret.is_empty() {
                None
            } else {
                Some(config.pair_secret.clone())
            };

            if let Err(e) = ximed::serve(DEFAULT_PORT, secret_b64).await {
                tracing::error!("Clipboard sharing server failed: {}", e);
            }
        });
    });
}
