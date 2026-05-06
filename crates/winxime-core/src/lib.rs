use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

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
    /// Cursor position in screen coordinates for UI positioning
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
    where
        F: FnOnce(&mut InputContext) -> R,
    {
        let result = {
            let mut guard = self.inner.lock().unwrap();
            f(&mut guard)
        };
        self.dirty.store(true, Ordering::Release);
        result
    }

    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut InputContext),
    {
        let mut guard = self.inner.lock().unwrap();
        f(&mut guard);
        self.dirty.store(true, Ordering::Release);
    }

    pub fn read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&InputContext) -> R,
    {
        let guard = self.inner.lock().unwrap();
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
