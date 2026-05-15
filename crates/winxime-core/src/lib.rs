use std::path::PathBuf;
use tracing_subscriber::{fmt, EnvFilter, prelude::*};

static LOG_GUARD: std::sync::OnceLock<tracing_appender::non_blocking::WorkerGuard> = std::sync::OnceLock::new();

pub fn init_logging(component: &str) {
    let log_dir = get_log_dir();
    std::fs::create_dir_all(&log_dir).ok();
    
    let file_appender = tracing_appender::rolling::RollingFileAppender::new(
        tracing_appender::rolling::Rotation::NEVER,
        log_dir,
        format!("{}.log", component),
    );
    
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    LOG_GUARD.set(guard).ok();
    
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("debug"));
    
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false)
            .with_target(false)
            .with_line_number(true))
        .try_init()
        .ok();
}

pub fn init_logging_with_console(component: &str) {
    let log_dir = get_log_dir();
    std::fs::create_dir_all(&log_dir).ok();
    
    let file_appender = tracing_appender::rolling::RollingFileAppender::new(
        tracing_appender::rolling::Rotation::NEVER,
        log_dir,
        format!("{}.log", component),
    );
    
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    LOG_GUARD.set(guard).ok();
    
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("debug"));
    
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false)
            .with_target(false)
            .with_line_number(true))
        .with(fmt::layer()
            .with_writer(std::io::stdout)
            .with_ansi(true))
        .try_init()
        .ok();
}

fn get_log_dir() -> PathBuf {
    std::env::var("TEMP")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("winxime")
}

pub fn log_dir() -> PathBuf {
    get_log_dir()
}

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

#[derive(Debug, Clone, Default)]
pub struct CompositionInfo {
    pub preedit: String,
    pub cursor_pos: usize,
    pub sel_start: usize,
    pub sel_end: usize,
}

#[derive(Debug, Clone, Default)]
pub struct CandidateInfo {
    pub text: String,
    pub comment: String,
}

#[derive(Debug, Clone, Default)]
pub struct InputContext {
    pub composition: CompositionInfo,
    pub candidates: Vec<CandidateInfo>,
    pub selected_index: usize,
    pub page_size: usize,
    pub is_composing: bool,
    pub commit_text: String,
    pub caret_x: i32,
    pub caret_y: i32,
}

pub struct SharedInputContext {
    inner: Mutex<InputContext>,
    dirty: AtomicBool,
}

impl SharedInputContext {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(InputContext::default()),
            dirty: AtomicBool::new(false),
        }
    }

    pub fn with<F, R>(&self, f: F) -> R
    where F: FnOnce(&mut InputContext) -> R,
    {
        let result = {
            let mut guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            f(&mut guard)
        };
        self.dirty.store(true, Ordering::Release);
        result
    }

    pub fn update<F>(&self, f: F)
    where F: FnOnce(&mut InputContext),
    {
        let mut guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        f(&mut guard);
        self.dirty.store(true, Ordering::Release);
    }

    pub fn read<F, R>(&self, f: F) -> R
    where F: FnOnce(&InputContext) -> R,
    {
        let guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        f(&guard)
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty.swap(false, Ordering::Acquire)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Chinese,
    Ascii,
}

impl Default for InputMode {
    fn default() -> Self {
        Self::Chinese
    }
}