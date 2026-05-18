use librime::{
    create_session, finalize, full_deploy_and_wait, get_api, initialize, is_maintenance_mode,
    join_maintenance_thread, setup, start_maintenance, DeployResult, Session, Traits,
};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::Mutex;

fn log_to_file(msg: &str) {
    if let Ok(temp) = std::env::var("TEMP") {
        let log_path = std::path::PathBuf::from(temp).join("winxime-rime.log");
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            let _ = writeln!(file, "[rime] {}", msg);
            let _ = file.flush();
        }
    }
}

static INIT_LOCK: Mutex<()> = Mutex::new(());

pub struct RimeEngine {
    session: Session,
    initialized: bool,
    shared_data_dir: PathBuf,
    user_data_dir: PathBuf,
    distribution_name: String,
}

unsafe impl Send for RimeEngine {}
unsafe impl Sync for RimeEngine {}

impl RimeEngine {
    pub fn new(
        shared_data_dir: &Path,
        user_data_dir: &Path,
        distribution_name: &str,
    ) -> Result<Self, RimeError> {
        let _lock = INIT_LOCK.lock().map_err(|_| RimeError::LockFailed)?;

        log_to_file("Creating RimeTraits...");
        let mut traits = Traits::new();
        traits
            .set_shared_data_dir(shared_data_dir.to_str().unwrap_or(""))
            .set_user_data_dir(user_data_dir.to_str().unwrap_or(""))
            .set_distribution_name(distribution_name)
            .set_distribution_code_name(distribution_name)
            .set_distribution_version("1.0")
            .set_app_name("rime.xime")
            .set_min_log_level(1);

        log_to_file("Calling setup...");
        setup(&mut traits);

        log_to_file("Calling initialize...");
        initialize(&mut traits).map_err(|e| RimeError::InitFailed(e.to_string()))?;

        log_to_file("Starting maintenance...");
        let need_maintenance = start_maintenance(false);
        log_to_file(&format!("start_maintenance result: {:?}", need_maintenance));
        if need_maintenance.is_ok() {
            log_to_file("Joining maintenance thread...");
            join_maintenance_thread();
            log_to_file("Maintenance complete");
        }

        log_to_file("Creating session...");
        let session =
            create_session().map_err(|e| RimeError::SessionCreateFailed(e.to_string()))?;

        log_to_file("Session created");

        let status = session
            .status()
            .map_err(|e| RimeError::StatusError(e.to_string()))?;
        log_to_file(&format!("Current schema: {}", status.schema_id()));

        Ok(Self {
            session,
            initialized: true,
            shared_data_dir: shared_data_dir.to_path_buf(),
            user_data_dir: user_data_dir.to_path_buf(),
            distribution_name: distribution_name.to_string(),
        })
    }

    pub fn process_key(&mut self, keycode: i32, modifiers: i32) -> bool {
        self.session.process_key(keycode, modifiers)
    }

    pub fn commit_composition(&mut self) -> bool {
        unsafe {
            let api = get_api();
            if api.is_null() {
                return false;
            }
            if let Some(commit) = (*api).commit_composition {
                commit(self.session.session_id()) != 0
            } else {
                false
            }
        }
    }

    pub fn clear_composition(&mut self) {
        unsafe {
            let api = get_api();
            if !api.is_null() {
                if let Some(clear) = (*api).clear_composition {
                    clear(self.session.session_id());
                }
            }
        }
    }

    pub fn get_commit(&self) -> Option<String> {
        self.session.commit().map(|c| c.text().to_string())
    }

    pub fn get_composition(&self) -> Option<Composition> {
        self.session.context().map(|ctx| {
            let comp = ctx.composition();
            Composition {
                length: comp.length,
                cursor_pos: comp.cursor_pos,
                sel_start: comp.sel_start,
                sel_end: comp.sel_end,
                preedit: comp.preedit.map(|s: &str| s.to_string()),
            }
        })
    }

    pub fn get_candidates(&self) -> CandidateList {
        self.session
            .context()
            .map(|ctx| {
                let menu = ctx.menu();
                let candidates = menu
                    .candidates
                    .iter()
                    .map(|c| Candidate {
                        text: c.text.to_string(),
                        comment: c.comment.map(|s: &str| s.to_string()),
                    })
                    .collect();

                CandidateList {
                    candidates,
                    highlighted: menu.highlighted_candidate_index,
                    page_no: menu.page_no,
                    is_last_page: menu.is_last_page,
                }
            })
            .unwrap_or_else(|| CandidateList {
                candidates: Vec::new(),
                highlighted: 0,
                page_no: 0,
                is_last_page: true,
            })
    }

    pub fn is_composing(&self) -> bool {
        self.session
            .status()
            .map(|s| s.is_composing)
            .unwrap_or(false)
    }

    pub fn is_ascii_mode(&self) -> bool {
        self.session
            .status()
            .map(|s| s.is_ascii_mode)
            .unwrap_or(false)
    }

    pub fn get_status(&self) -> Option<RimeEngineStatus> {
        self.session.status().ok().map(|s| RimeEngineStatus {
            is_composing: s.is_composing,
            is_ascii_mode: s.is_ascii_mode,
            schema_id: s.schema_id().to_string(),
            schema_name: s.schema_name().to_string(),
        })
    }

    pub fn select_candidate(&mut self, index: usize) -> bool {
        unsafe {
            let api = get_api();
            if api.is_null() {
                return false;
            }
            if let Some(select) = (*api).select_candidate {
                select(self.session.session_id(), index) != 0
            } else if let Some(select_on_page) = (*api).select_candidate_on_current_page {
                select_on_page(self.session.session_id(), index) != 0
            } else {
                false
            }
        }
    }

    pub fn change_page(&mut self, backward: bool) -> bool {
        unsafe {
            let api = get_api();
            if api.is_null() {
                return false;
            }
            if let Some(change) = (*api).change_page {
                change(self.session.session_id(), if backward { 1 } else { 0 }) != 0
            } else {
                false
            }
        }
    }

    pub fn get_input(&self) -> Option<String> {
        self.session.get_input().map(|s: &str| s.to_string())
    }

    pub fn set_input(&mut self, input: &str) -> bool {
        unsafe {
            let api = get_api();
            if api.is_null() {
                return false;
            }
            if let Some(set_input) = (*api).set_input {
                let c_input = std::ffi::CString::new(input).unwrap();
                set_input(self.session.session_id(), c_input.as_ptr()) != 0
            } else {
                false
            }
        }
    }

    pub fn set_option(&mut self, option: &str, value: bool) {
        let _ = self.session.set_option(option, value);
    }

    pub fn get_option(&self, option: &str) -> Option<bool> {
        self.session.get_option(option).ok()
    }

    pub fn get_schema_list(&self) -> Vec<(String, String)> {
        unsafe {
            let api = get_api();
            if api.is_null() {
                return Vec::new();
            }

            let mut list = librime_sys2::RimeSchemaList {
                size: 0,
                list: ptr::null_mut(),
            };

            let mut result = Vec::new();
            if let Some(get_list) = (*api).get_schema_list {
                if get_list(&mut list) != 0 {
                    for i in 0..list.size {
                        let item = list.list.add(i);
                        let id = std::ffi::CStr::from_ptr((*item).schema_id)
                            .to_string_lossy()
                            .to_string();
                        let name = std::ffi::CStr::from_ptr((*item).name)
                            .to_string_lossy()
                            .to_string();
                        result.push((id, name));
                    }
                    if let Some(free) = (*api).free_schema_list {
                        free(&mut list);
                    }
                }
            }
            result
        }
    }

    pub fn select_schema(&mut self, schema_id: &str) -> bool {
        self.session.select_schema(schema_id).is_ok()
    }

    pub fn get_current_schema(&self) -> Option<String> {
        self.session
            .status()
            .ok()
            .map(|s| s.schema_id().to_string())
    }

    pub fn redeploy(&mut self) -> bool {
        log_to_file("Redeploying Rime...");

        log_to_file("Finalizing old session...");
        self.initialized = false;
        finalize();

        log_to_file("Creating new traits...");
        let mut traits = Traits::new();
        traits
            .set_shared_data_dir(self.shared_data_dir.to_str().unwrap_or(""))
            .set_user_data_dir(self.user_data_dir.to_str().unwrap_or(""))
            .set_distribution_name(&self.distribution_name)
            .set_distribution_code_name(&self.distribution_name)
            .set_distribution_version("1.0")
            .set_app_name("rime.xime")
            .set_min_log_level(1);

        log_to_file("Calling setup...");
        setup(&mut traits);

        log_to_file("Calling initialize...");
        if initialize(&mut traits).is_err() {
            log_to_file("Initialize failed!");
            return false;
        }

        log_to_file("Running full_deploy_and_wait...");
        let result = full_deploy_and_wait();
        log_to_file(&format!("Deploy result: {:?}", result));

        if is_maintenance_mode() {
            log_to_file("Joining maintenance thread...");
            join_maintenance_thread();
        }

        log_to_file("Creating new session...");
        match create_session() {
            Ok(session) => {
                self.session = session;
                self.initialized = true;
                log_to_file("New session created successfully");

                if let Ok(status) = self.session.status() {
                    log_to_file(&format!("Current schema: {}", status.schema_id()));
                }

                true
            }
            Err(e) => {
                log_to_file(&format!("Failed to create session: {}", e));
                false
            }
        }
    }

    pub fn deploy(&self) -> bool {
        full_deploy_and_wait() == DeployResult::Success
    }

    pub fn get_version(&self) -> Option<String> {
        unsafe {
            let api = get_api();
            if api.is_null() {
                return None;
            }
            if let Some(get_ver) = (*api).get_version {
                let ptr = get_ver();
                if ptr.is_null() {
                    None
                } else {
                    std::ffi::CStr::from_ptr(ptr)
                        .to_str()
                        .ok()
                        .map(|s: &str| s.to_string())
                }
            } else {
                None
            }
        }
    }
}

impl Drop for RimeEngine {
    fn drop(&mut self) {
        if !self.initialized {
            return;
        }
        finalize();
    }
}

#[derive(Debug, Clone)]
pub struct RimeEngineStatus {
    pub is_composing: bool,
    pub is_ascii_mode: bool,
    pub schema_id: String,
    pub schema_name: String,
}

#[derive(Debug, Clone)]
pub struct Composition {
    pub length: usize,
    pub cursor_pos: usize,
    pub sel_start: usize,
    pub sel_end: usize,
    pub preedit: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Candidate {
    pub text: String,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CandidateList {
    pub candidates: Vec<Candidate>,
    pub highlighted: usize,
    pub page_no: usize,
    pub is_last_page: bool,
}

#[derive(Debug)]
pub enum RimeError {
    ApiNotFound,
    InitFailed(String),
    SessionCreateFailed(String),
    StatusError(String),
    LockFailed,
}

impl std::fmt::Display for RimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RimeError::ApiNotFound => write!(f, "Rime API not found (rime.dll missing)"),
            RimeError::InitFailed(msg) => write!(f, "Rime initialization failed: {}", msg),
            RimeError::SessionCreateFailed(msg) => {
                write!(f, "Failed to create Rime session: {}", msg)
            }
            RimeError::StatusError(msg) => write!(f, "Failed to get status: {}", msg),
            RimeError::LockFailed => write!(f, "Failed to acquire initialization lock"),
        }
    }
}

impl std::error::Error for RimeError {}
