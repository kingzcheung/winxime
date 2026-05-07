use std::path::{Path, PathBuf};
use std::io::{self, Read, Write};
use std::fs::{self, File};

const DEFAULT_USER_CONFIG_URL: &str = "https://github.com/kingzcheung/rime-wubi/archive/refs/tags/1.0.0.tar.gz";
const REQUIRED_USER_CONFIGS: &[&str] = &[
    "default.custom.yaml",
    "wubi86_jidian.schema.yaml",
    "wubi86_jidian.dict.yaml",
];

const REQUIRED_SHARED_CONFIGS: &[&str] = &[
    "default.yaml",
    "essay.txt",
];

pub struct XimeConfig {
    shared_data_dir: PathBuf,
    user_data_dir: PathBuf,
}

impl XimeConfig {
    pub fn new(shared_data_dir: PathBuf, user_data_dir: PathBuf) -> Self {
        Self { shared_data_dir, user_data_dir }
    }

    pub fn default() -> Self {
        let user_data_dir = Self::default_user_data_dir();
        let shared_data_dir = Self::default_shared_data_dir();
        Self::new(shared_data_dir, user_data_dir)
    }

    pub fn default_user_data_dir() -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            std::env::var("APPDATA")
                .map(|p| PathBuf::from(p).join("Xime"))
                .unwrap_or_else(|_| PathBuf::from("."))
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            PathBuf::from(".")
        }
    }

    pub fn default_shared_data_dir() -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            let exe_path = std::env::current_exe().unwrap_or_default();
            exe_path.parent()
                .map(|p| p.join("data"))
                .unwrap_or_default()
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            PathBuf::from("/usr/share/rime")
        }
    }

    pub fn from_workspace(workspace_dir: &Path) -> Self {
        let shared_data_dir = workspace_dir.join("librime").join("data").join("minimal");
        let user_data_dir = workspace_dir.join("rime-data");
        Self::new(shared_data_dir, user_data_dir)
    }

    pub fn shared_data_dir(&self) -> &Path {
        &self.shared_data_dir
    }

    pub fn user_data_dir(&self) -> &Path {
        &self.user_data_dir
    }

    pub fn ensure_dirs(&self) -> io::Result<()> {
        fs::create_dir_all(&self.user_data_dir)?;
        fs::create_dir_all(&self.shared_data_dir)?;
        Ok(())
    }

    pub fn has_required_shared_configs(&self) -> bool {
        REQUIRED_SHARED_CONFIGS.iter().all(|cfg| {
            self.shared_data_dir.join(cfg).exists()
        })
    }

    pub fn has_required_user_configs(&self) -> bool {
        REQUIRED_USER_CONFIGS.iter().all(|cfg| {
            self.user_data_dir.join(cfg).exists()
        })
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        self.ensure_dirs()?;
        
        if !self.has_required_user_configs() {
            println!("User configs missing, downloading default configs...");
            self.download_default_user_configs()?;
        }

        if !self.has_required_shared_configs() {
            let missing = REQUIRED_SHARED_CONFIGS.iter()
                .find(|cfg| !self.shared_data_dir.join(cfg).exists())
                .copied()
                .unwrap_or("unknown");
            return Err(ConfigError::MissingSharedConfig(missing));
        }

        Ok(())
    }

    fn download_default_user_configs(&self) -> Result<(), ConfigError> {
        println!("Downloading from {}", DEFAULT_USER_CONFIG_URL);
        
        let temp_dir = self.user_data_dir.join(".temp_download");
        fs::create_dir_all(&temp_dir)?;
        
        let tar_path = temp_dir.join("rime-wubi.tar.gz");
        
        let response = ureq::get(DEFAULT_USER_CONFIG_URL)
            .call()
            .map_err(|e| ConfigError::DownloadFailed(e.to_string()))?;
        
        let mut file = File::create(&tar_path)?;
        let mut buffer = Vec::new();
        response.into_reader().read_to_end(&mut buffer)?;
        file.write_all(&buffer)?;
        
        println!("Extracting...");
        self.extract_tar_gz(&tar_path)?;
        
        fs::remove_dir_all(&temp_dir)?;
        
        println!("Default user configs installed");
        Ok(())
    }

    fn extract_tar_gz(&self, tar_path: &Path) -> Result<(), ConfigError> {
        use flate2::read::GzDecoder;
        use tar::Archive;
        
        let file = File::open(tar_path)?;
        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);
        
        for entry_result in archive.entries()? {
            let mut entry = entry_result?;
            let path = entry.path()?;
            let path_str = path.to_string_lossy();
            
            if path_str.contains("rime-wubi-1.0.0/") {
                let relative_path = path_str
                    .strip_prefix("rime-wubi-1.0.0/")
                    .unwrap_or(&path_str);
                
                if !relative_path.is_empty() && !relative_path.ends_with('/') {
                    let dest = self.user_data_dir.join(relative_path);
                    
                    if let Some(parent) = dest.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    
                    entry.unpack(dest)?;
                }
            }
        }
        
        Ok(())
    }
}

#[derive(Debug)]
pub enum ConfigError {
    IoError(io::Error),
    DownloadFailed(String),
    ExtractFailed(String),
    MissingSharedConfig(&'static str),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ConfigError::DownloadFailed(s) => write!(f, "Download failed: {}", s),
            ConfigError::ExtractFailed(s) => write!(f, "Extract failed: {}", s),
            ConfigError::MissingSharedConfig(s) => write!(f, "Missing shared config: {}", s),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> Self {
        ConfigError::IoError(e)
    }
}