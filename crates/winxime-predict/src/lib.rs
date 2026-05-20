use ort::{inputs, session::builder::GraphOptimizationLevel, session::Session, value::TensorRef};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;
use tracing::info;

#[derive(Error, Debug)]
pub enum PredictError {
    #[error("Failed to load vocab: {0}")]
    VocabLoad(String),
    #[error("Failed to load model: {0}")]
    ModelLoad(String),
    #[error("ORT error: {0}")]
    Ort(String),
    #[error("Model not found: {0}")]
    ModelNotFound(String),
}

impl From<ort::Error> for PredictError {
    fn from(e: ort::Error) -> Self {
        PredictError::Ort(e.to_string())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Vocab {
    tokens: HashMap<String, i64>,
    ids: HashMap<i64, String>,
    bos_id: i64,
    eos_id: i64,
    unk_id: i64,
    pad_id: i64,
}

impl Vocab {
    pub fn load(path: &PathBuf) -> Result<Self, PredictError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| PredictError::VocabLoad(e.to_string()))?;

        let raw: HashMap<String, i64> =
            serde_json::from_str(&content).map_err(|e| PredictError::VocabLoad(e.to_string()))?;

        let ids: HashMap<i64, String> = raw.iter().map(|(k, v)| (*v, k.clone())).collect();

        let bos_id = raw.get("[BOS]").copied().unwrap_or(1);
        let eos_id = raw.get("[EOS]").copied().unwrap_or(2);
        let unk_id = raw.get("[UNK]").copied().unwrap_or(3);
        let pad_id = raw.get("[PAD]").copied().unwrap_or(0);

        Ok(Self {
            tokens: raw,
            ids,
            bos_id,
            eos_id,
            unk_id,
            pad_id,
        })
    }

    pub fn encode(&self, text: &str) -> Vec<i64> {
        let mut ids = vec![self.bos_id];
        for ch in text.chars() {
            ids.push(
                self.tokens
                    .get(&ch.to_string())
                    .copied()
                    .unwrap_or(self.unk_id),
            );
        }
        ids
    }

    pub fn decode(&self, id: i64) -> Option<&str> {
        self.ids.get(&id).map(|s| s.as_str())
    }

    pub fn is_special(&self, id: i64) -> bool {
        id == self.pad_id || id == self.bos_id || id == self.eos_id || id == self.unk_id
    }

    pub fn vocab_size(&self) -> usize {
        self.tokens.len()
    }
}

pub struct Predictor {
    vocab: Vocab,
    session: Session,
    model_dir: PathBuf,
    model_name: String,
}

impl Predictor {
    pub fn new(model_name: Option<&str>) -> Result<Self, PredictError> {
        let name = model_name.unwrap_or("predictive-text-small");
        let model_dir = get_model_dir(Some(name));

        let vocab_path = model_dir.join("vocab.json");
        let model_path = model_dir.join("model.onnx");

        if !model_path.exists() {
            return Err(PredictError::ModelNotFound(
                model_path.display().to_string(),
            ));
        }

        let vocab = Vocab::load(&vocab_path)?;

        info!("Loading ONNX model from {}", model_path.display());

        let session = Session::builder()
            .map_err(|e| PredictError::ModelLoad(e.to_string()))?
            .with_optimization_level(GraphOptimizationLevel::Level1)
            .map_err(|e| PredictError::ModelLoad(e.to_string()))?
            .with_intra_threads(1)
            .map_err(|e| PredictError::ModelLoad(e.to_string()))?
            .commit_from_file(model_path)?;

        info!("Model loaded successfully");

        Ok(Self {
            vocab,
            session,
            model_dir,
            model_name: name.to_string(),
        })
    }

    pub fn predict(
        &mut self,
        prefix: &str,
        top_k: usize,
    ) -> Result<Vec<(String, f32)>, PredictError> {
        let tokens = self.vocab.encode(prefix);

        let input =
            TensorRef::from_array_view((vec![1i64, tokens.len() as i64], tokens.as_slice()))?;

        let outputs = self.session.run(inputs![input])?;

        let (dim, probabilities) = outputs[0].try_extract_tensor::<f32>()?;

        let seq_len = dim[1] as usize;
        let vocab_size = dim[2] as usize;

        let last_token_probs = &probabilities[(seq_len - 1) * vocab_size..];

        let mut candidates: Vec<(usize, f32)> =
            last_token_probs.iter().copied().enumerate().collect();

        candidates
            .sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Less));

        Ok(candidates
            .iter()
            .take(top_k)
            .filter_map(|(id, score)| {
                if self.vocab.is_special(*id as i64) {
                    return None;
                }
                self.vocab
                    .decode(*id as i64)
                    .map(|token| (token.to_string(), *score))
            })
            .collect())
    }

    pub fn model_dir(&self) -> &PathBuf {
        &self.model_dir
    }

    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    pub fn vocab_size(&self) -> usize {
        self.vocab.vocab_size()
    }
}

pub fn get_model_dir(model_name: Option<&str>) -> PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let base = PathBuf::from(appdata).join("Xime").join("models");
    match model_name {
        Some(name) => base.join(name),
        None => base.join("predictive-text-small"),
    }
}

pub fn check_model_exists(model_name: Option<&str>) -> bool {
    let model_dir = get_model_dir(model_name);
    model_dir.join("vocab.json").exists()
        && model_dir.join("model.onnx").exists()
        && model_dir.join("model.onnx.data").exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn download_test_model() -> bool {
        let model_dir = get_model_dir(Some("predictive-text-small"));
        if check_model_exists(Some("predictive-text-small")) {
            return true;
        }

        if !model_dir.exists() {
            std::fs::create_dir_all(&model_dir).ok();
        }

        let base_url = "https://modelscope.cn/models/bikeand/predictive-text-small/resolve/master";
        let files = [
            ("vocab.json", format!("{}/vocab.json", base_url)),
            ("model.onnx", format!("{}/model.onnx", base_url)),
            ("model.onnx.data", format!("{}/model.onnx.data", base_url)),
        ];

        for (filename, url) in files {
            println!("Downloading {}...", filename);
            let response = ureq::get(&url).call().ok();
            if response.is_none() {
                println!("Failed to download {}", filename);
                return false;
            }

            let mut content = Vec::new();
            if response
                .and_then(|r| r.into_reader().read_to_end(&mut content).ok())
                .is_none()
            {
                println!("Failed to read {}", filename);
                return false;
            }

            let path = model_dir.join(filename);
            if std::fs::write(&path, &content).is_err() {
                println!("Failed to write {}", filename);
                return false;
            }
            println!("{} downloaded", filename);
        }
        true
    }

    #[test]
    fn test_vocab_load() {
        let vocab_path = PathBuf::from("test_vocab.json");
        let test_vocab = r#"{"[PAD]":0,"[BOS]":1,"[EOS]":2,"[UNK]":3,"你":4,"好":5,"世":6,"界":7}"#;
        std::fs::write(&vocab_path, test_vocab).ok();

        if let Ok(vocab) = Vocab::load(&vocab_path) {
            assert_eq!(vocab.encode("你好"), vec![1, 4, 5]);
            assert_eq!(vocab.decode(4), Some("你"));
            assert!(vocab.is_special(0));
            assert!(!vocab.is_special(4));
            assert_eq!(vocab.vocab_size(), 8);
        }

        // std::fs::remove_file(&vocab_path).ok();
    }

    #[test]
    fn test_vocab_encode_decode() {
        let vocab_path = PathBuf::from("test_vocab2.json");
        let test_vocab = r#"{"[PAD]":0,"[BOS]":1,"[EOS]":2,"[UNK]":3,"a":4,"b":5,"c":6,"d":7}"#;
        std::fs::write(&vocab_path, test_vocab).ok();

        let vocab = Vocab::load(&vocab_path).ok().unwrap();

        assert_eq!(vocab.encode("abc"), vec![1, 4, 5, 6]);
        assert_eq!(vocab.encode(""), vec![1]);
        assert_eq!(vocab.encode("xyz"), vec![1, 3, 3, 3]);

        assert_eq!(vocab.decode(0), Some("[PAD]"));
        assert_eq!(vocab.decode(1), Some("[BOS]"));
        assert_eq!(vocab.decode(99), None);

        std::fs::remove_file(&vocab_path).ok();
    }

    #[test]
    fn test_vocab_special_tokens() {
        let vocab_path = PathBuf::from("test_vocab3.json");
        let test_vocab = r#"{"[PAD]":0,"[BOS]":1,"[EOS]":2,"[UNK]":3,"test":4}"#;
        std::fs::write(&vocab_path, test_vocab).ok();

        let vocab = Vocab::load(&vocab_path).ok().unwrap();

        assert!(vocab.is_special(0));
        assert!(vocab.is_special(1));
        assert!(vocab.is_special(2));
        assert!(vocab.is_special(3));
        assert!(!vocab.is_special(4));

        std::fs::remove_file(&vocab_path).ok();
    }

    #[test]
    fn test_get_model_dir() {
        let dir = get_model_dir(Some("test-model"));
        assert!(dir.to_string_lossy().contains("Xime"));
        assert!(dir.to_string_lossy().contains("models"));
        assert!(dir.to_string_lossy().contains("test-model"));

        let default_dir = get_model_dir(None);
        assert!(default_dir
            .to_string_lossy()
            .contains("predictive-text-small"));
    }

    #[test]
    fn test_check_model_exists() {
        let _fake_dir = get_model_dir(Some("nonexistent-model"));
        assert!(!check_model_exists(Some("nonexistent-model")));
    }

    #[test]
    #[ignore]
    fn test_predictor_with_real_model() {
        if !download_test_model() {
            println!("Skipping test: model download failed");
            return;
        }

        let result = Predictor::new(Some("predictive-text-small"));
        if result.is_err() {
            println!("Failed to load predictor: {:?}", result.err());
            return;
        }

        let mut predictor = result.ok().unwrap();
        println!("Model loaded: {}", predictor.model_name());
        println!("Vocab size: {}", predictor.vocab_size());
        println!("Model dir: {}", predictor.model_dir().display());

        let pred_result = predictor.predict("你", 5);
        match pred_result {
            Ok(results) => {
                println!("Prediction for '你': {:?}", results);
                assert!(
                    !results.is_empty(),
                    "Prediction results should not be empty"
                );
                for (token, score) in &results {
                    println!("  {} (score: {:.4})", token, score);
                }
            }
            Err(e) => {
                println!("Prediction failed: {:?}", e);
            }
        }

        let pred_result2 = predictor.predict("今天", 5);
        match pred_result2 {
            Ok(results) => {
                println!("Prediction for '今天': {:?}", results);
                assert!(!results.is_empty());
            }
            Err(e) => {
                println!("Prediction failed: {:?}", e);
            }
        }
    }

    #[test]
    #[ignore]
    fn test_predictor_vocab_loading() {
        if !download_test_model() {
            println!("Skipping test: model download failed");
            return;
        }

        if let Ok(predictor) = Predictor::new(Some("predictive-text-small")) {
            assert!(predictor.vocab_size() > 0);
            assert_eq!(predictor.model_name(), "predictive-text-small");
        }
    }
}
