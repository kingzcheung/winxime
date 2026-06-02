use base64::Engine;
use std::thread;
use tracing::{info, warn};
use winxime_config::XimeConfig;

const DEFAULT_PORT: u16 = 8370;

/// 检查字符串是否为有效的 URL_SAFE base64 编码
fn is_valid_base64(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    // URL_SAFE base64: [A-Za-z0-9-_=]，允许下划线和横线
    s.bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'=')
}

/// 生成并持久化新的 pair_secret，返回 base64 编码的密钥字符串
fn ensure_pair_secret(config: &XimeConfig) -> Option<String> {
    // 已有有效密钥，直接使用
    if !config.pair_secret.is_empty() && is_valid_base64(&config.pair_secret) {
        return Some(config.pair_secret.clone());
    }

    // 密钥无效或不存在，生成新的
    if !config.pair_secret.is_empty() {
        warn!(
            "Invalid base64 pair_secret in config, generating new one: {}",
            config.pair_secret
        );
    } else {
        info!("No pair_secret found, generating new one");
    }

    let secret_bytes = uuid::Uuid::new_v4().as_bytes().to_vec();
    let secret_b64 =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&secret_bytes);

    // 保存到配置
    let updated = XimeConfig {
        pair_secret: secret_b64.clone(),
        ..config.clone()
    };
    if let Err(e) = updated.save() {
        warn!("Failed to persist pair_secret: {e}");
    }

    Some(secret_b64)
}

pub fn start() {
    thread::spawn(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build tokio runtime for clipboard server");
        rt.block_on(async {
            info!("Starting clipboard sharing server on port {}", DEFAULT_PORT);

            let config = XimeConfig::load();
            let secret_b64 = ensure_pair_secret(&config);

            info!("Clipboard sharing server using persistent secret");
            if let Err(e) = ximed::serve(DEFAULT_PORT, secret_b64).await {
                tracing::error!("Clipboard sharing server failed: {}", e);
            }
        });
    });
}
