mod win32_ui;

use std::sync::Arc;
use winxime_core::SharedInputContext;

pub use win32_ui::CandidateWindowInner;

impl CandidateWindowInner {
    pub fn new(context: &'static SharedInputContext) -> Arc<Self> {
        Self::start(context)
    }
}
