use librime_sys::*;
use std::ffi::{CStr, CString};
use std::path::Path;
use std::sync::Mutex;
use std::io::Write;

fn log_to_file(msg: &str) {
    if let Ok(temp) = std::env::var("TEMP") {
        let log_path = std::path::PathBuf::from(temp).join("winxime-rime.log");
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&log_path) {
            let _ = writeln!(file, "[rime] {}", msg);
            let _ = file.flush();
        }
    }
}

static INIT_LOCK: Mutex<()> = Mutex::new(());

pub struct RimeEngine {
    api: *const RimeApi,
    session: RimeSessionId,
    initialized: bool,
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

        let api = rime_get_api().ok_or(RimeError::ApiNotFound)?;

        unsafe {
            let shared = CString::new(shared_data_dir.to_str().unwrap()).unwrap();
            let user = CString::new(user_data_dir.to_str().unwrap()).unwrap();
            let dist_name = CString::new(distribution_name).unwrap();
            let app_name = CString::new("rime.xime").unwrap();

            rime_struct!(traits: RimeTraits);
            traits.shared_data_dir = shared.as_ptr();
            traits.user_data_dir = user.as_ptr();
            traits.distribution_name = dist_name.as_ptr();
            traits.distribution_code_name = dist_name.as_ptr();
            traits.distribution_version = b"1.0\0".as_ptr() as *const i8;
            traits.app_name = app_name.as_ptr();
            traits.min_log_level = 1; // WARNING

            if let Some(setup) = (*api).setup {
                setup(&mut traits);
            } else {
                return Err(RimeError::ApiFunctionMissing("setup"));
            }

            if let Some(init) = (*api).initialize {
                init(&mut traits);
            } else {
                return Err(RimeError::ApiFunctionMissing("initialize"));
            }

            // Initialize deployer
            log_to_file("Initializing deployer...");
            if let Some(deployer_init) = (*api).deployer_initialize {
                deployer_init(std::ptr::null_mut());
                log_to_file("Deployer initialized");
            }

            // Deploy schemas
            log_to_file("Deploying Rime schemas...");
            if let Some(deploy) = (*api).deploy {
                let deploy_result = deploy();
                log_to_file(&format!("Deploy result: {}", deploy_result));
                if deploy_result == FALSE {
                    log_to_file("Warning: Rime deploy returned false");
                } else {
                    log_to_file("Rime schemas deployed successfully");
                }
            } else {
                log_to_file("Warning: deploy function not available");
            }

            // Deploy UI config file (xime.yaml)
            log_to_file("Deploying xime.yaml...");
            if let Some(deploy_config) = (*api).deploy_config_file {
                let config_file = CString::new("xime.yaml").unwrap_or_default();
                let version_key = CString::new("config_version").unwrap_or_default();
                let result = deploy_config(config_file.as_ptr(), version_key.as_ptr());
                log_to_file(&format!("Deploy xime.yaml result: {}", result));
            } else {
                log_to_file("Warning: deploy_config_file not available");
            }

            // Set notification handler for deploy/option events
            if let Some(set_handler) = (*api).set_notification_handler {
                set_handler(Some(rime_notification_callback), std::ptr::null_mut());
            }

            let session = if let Some(create) = (*api).create_session {
                let sid = create();
                log_to_file(&format!("Session created: {}", sid));
                sid
            } else {
                return Err(RimeError::ApiFunctionMissing("create_session"));
            };

            if session == 0 {
                log_to_file("Session creation failed!");
                return Err(RimeError::SessionCreateFailed);
            }

            // Check current schema
            let mut schema_buf = [0i8; 256];
            if let Some(get_schema) = (*api).get_current_schema {
                if get_schema(session, schema_buf.as_mut_ptr(), schema_buf.len()) != FALSE {
                    let schema_id = CStr::from_ptr(schema_buf.as_ptr()).to_string_lossy();
                    log_to_file(&format!("Current schema: {}", schema_id));
                }
            }

            Ok(Self {
                api,
                session,
                initialized: true,
            })
        }
    }

    pub fn process_key(&mut self, keycode: i32, modifiers: i32) -> bool {
        unsafe {
            if let Some(process) = (*self.api).process_key {
                process(self.session, keycode, modifiers) != FALSE
            } else {
                false
            }
        }
    }

    pub fn commit_composition(&mut self) -> bool {
        unsafe {
            if let Some(commit) = (*self.api).commit_composition {
                commit(self.session) != FALSE
            } else {
                false
            }
        }
    }

    pub fn clear_composition(&mut self) {
        unsafe {
            if let Some(clear) = (*self.api).clear_composition {
                clear(self.session);
            }
        }
    }

    pub fn get_commit(&self) -> Option<String> {
        unsafe {
            rime_struct!(commit: RimeCommit);

            if let Some(get_commit) = (*self.api).get_commit {
                if get_commit(self.session, &mut commit) == FALSE {
                    if let Some(free) = (*self.api).free_commit {
                        free(&mut commit);
                    }
                    return None;
                }
            }

            let text = c_str_to_string(commit.text);
            if let Some(free) = (*self.api).free_commit {
                free(&mut commit);
            }
            text
        }
    }

    pub fn get_composition(&self) -> Option<Composition> {
        unsafe {
            rime_struct!(ctx: RimeContext);

            if let Some(get_ctx) = (*self.api).get_context {
                if get_ctx(self.session, &mut ctx) == FALSE {
                    if let Some(free) = (*self.api).free_context {
                        free(&mut ctx);
                    }
                    return None;
                }
            } else {
                return None;
            }

            let composition = Composition {
                length: ctx.composition.length as usize,
                cursor_pos: ctx.composition.cursor_pos as usize,
                sel_start: ctx.composition.sel_start as usize,
                sel_end: ctx.composition.sel_end as usize,
                preedit: c_str_to_string(ctx.composition.preedit),
            };

            if let Some(free) = (*self.api).free_context {
                free(&mut ctx);
            }

            Some(composition)
        }
    }

    pub fn get_candidates(&self) -> CandidateList {
        unsafe {
            rime_struct!(ctx: RimeContext);

            if let Some(get_ctx) = (*self.api).get_context {
                if get_ctx(self.session, &mut ctx) == FALSE {
                    if let Some(free) = (*self.api).free_context {
                        free(&mut ctx);
                    }
                    return CandidateList {
                        candidates: Vec::new(),
                        highlighted: 0,
                        page_no: 0,
                        is_last_page: true,
                    };
                }
            } else {
                return CandidateList {
                    candidates: Vec::new(),
                    highlighted: 0,
                    page_no: 0,
                    is_last_page: true,
                };
            }

            let num = ctx.menu.num_candidates as usize;
            let mut candidates = Vec::with_capacity(num);

            for i in 0..num {
                let candidate_ptr = ctx.menu.candidates.add(i);
                let text = c_str_to_string((*candidate_ptr).text).unwrap_or_default();
                let comment = c_str_to_string((*candidate_ptr).comment);
                candidates.push(Candidate { text, comment });
            }

            let highlighted = ctx.menu.highlighted_candidate_index as usize;
            let page_no = ctx.menu.page_no as usize;
            let is_last_page = ctx.menu.is_last_page != FALSE;

            if let Some(free) = (*self.api).free_context {
                free(&mut ctx);
            }

            CandidateList {
                candidates,
                highlighted,
                page_no,
                is_last_page,
            }
        }
    }

    pub fn is_composing(&self) -> bool {
        unsafe {
            rime_struct!(status: RimeStatus);

            if let Some(get_status) = (*self.api).get_status {
                if get_status(self.session, &mut status) == FALSE {
                    if let Some(free) = (*self.api).free_status {
                        free(&mut status);
                    }
                    return false;
                }
            } else {
                return false;
            }

            let composing = status.is_composing != FALSE;
            if let Some(free) = (*self.api).free_status {
                free(&mut status);
            }
            composing
        }
    }

    pub fn is_ascii_mode(&self) -> bool {
        unsafe {
            rime_struct!(status: RimeStatus);

            if let Some(get_status) = (*self.api).get_status {
                if get_status(self.session, &mut status) == FALSE {
                    if let Some(free) = (*self.api).free_status {
                        free(&mut status);
                    }
                    return false;
                }
            } else {
                return false;
            }

            let ascii_mode = status.is_ascii_mode != FALSE;
            if let Some(free) = (*self.api).free_status {
                free(&mut status);
            }
            ascii_mode
        }
    }

    pub fn get_status(&self) -> Option<RimeEngineStatus> {
        unsafe {
            rime_struct!(status: RimeStatus);

            if let Some(get_status) = (*self.api).get_status {
                if get_status(self.session, &mut status) == FALSE {
                    if let Some(free) = (*self.api).free_status {
                        free(&mut status);
                    }
                    return None;
                }
            } else {
                return None;
            }

            let result = RimeEngineStatus {
                is_composing: status.is_composing != FALSE,
                is_ascii_mode: status.is_ascii_mode != FALSE,
                schema_id: c_str_to_string(status.schema_id).unwrap_or_default(),
                schema_name: c_str_to_string(status.schema_name).unwrap_or_default(),
            };

            if let Some(free) = (*self.api).free_status {
                free(&mut status);
            }
            Some(result)
        }
    }

    pub fn select_candidate(&mut self, index: usize) -> bool {
        unsafe {
            if let Some(select) = (*self.api).select_candidate {
                select(self.session, index) != FALSE
            } else if let Some(select_on_page) = (*self.api).select_candidate_on_current_page {
                select_on_page(self.session, index) != FALSE
            } else {
                false
            }
        }
    }

    pub fn change_page(&mut self, backward: bool) -> bool {
        unsafe {
            if let Some(change) = (*self.api).change_page {
                change(self.session, if backward { TRUE } else { FALSE }) != FALSE
            } else {
                false
            }
        }
    }

    pub fn get_input(&self) -> Option<String> {
        unsafe {
            if let Some(get_input) = (*self.api).get_input {
                let ptr = get_input(self.session);
                if ptr.is_null() {
                    None
                } else {
                    CStr::from_ptr(ptr).to_str().ok().map(|s| s.to_owned())
                }
            } else {
                None
            }
        }
    }

    pub fn set_input(&mut self, input: &str) -> bool {
        unsafe {
            if let Some(set_input) = (*self.api).set_input {
                let c_input = CString::new(input).unwrap();
                set_input(self.session, c_input.as_ptr()) != FALSE
            } else {
                false
            }
        }
    }

    pub fn set_option(&mut self, option: &str, value: bool) {
        unsafe {
            if let Some(set_opt) = (*self.api).set_option {
                let c_opt = CString::new(option).unwrap();
                set_opt(
                    self.session,
                    c_opt.as_ptr(),
                    if value { TRUE } else { FALSE },
                );
            }
        }
    }

    pub fn get_option(&self, option: &str) -> Option<bool> {
        unsafe {
            if let Some(get_opt) = (*self.api).get_option {
                let c_opt = CString::new(option).unwrap();
                let val = get_opt(self.session, c_opt.as_ptr());
                Some(val != FALSE)
            } else {
                None
            }
        }
    }

    pub fn get_schema_list(&self) -> Vec<(String, String)> {
        unsafe {
            let list_init = std::mem::MaybeUninit::<RimeSchemaList>::zeroed();
            let mut list = list_init.assume_init();
            list.size = 0;
            list.list = std::ptr::null_mut();

            let mut result = Vec::new();
            if let Some(get_list) = (*self.api).get_schema_list {
                if get_list(&mut list) != FALSE {
                    for i in 0..list.size {
                        let item = list.list.add(i);
                        let id = c_str_to_string((*item).schema_id).unwrap_or_default();
                        let name = c_str_to_string((*item).name).unwrap_or_default();
                        result.push((id, name));
                    }
                    if let Some(free) = (*self.api).free_schema_list {
                        free(&mut list);
                    }
                }
            }
            result
        }
    }

    pub fn select_schema(&mut self, schema_id: &str) -> bool {
        unsafe {
            if let Some(select) = (*self.api).select_schema {
                let c_id = CString::new(schema_id).unwrap();
                select(self.session, c_id.as_ptr()) != FALSE
            } else {
                false
            }
        }
    }

    pub fn get_current_schema(&self) -> Option<String> {
        unsafe {
            let mut buf = [0i8; 256];
            if let Some(get) = (*self.api).get_current_schema {
                if get(self.session, buf.as_mut_ptr(), buf.len()) != FALSE {
                    return c_str_to_string(buf.as_mut_ptr());
                }
            }
            None
        }
    }

    pub fn deploy(&self) -> bool {
        unsafe {
            if let Some(deploy) = (*self.api).deploy {
                deploy() != FALSE
            } else {
                false
            }
        }
    }

    pub fn get_version(&self) -> Option<String> {
        unsafe {
            if let Some(get_ver) = (*self.api).get_version {
                let ptr = get_ver();
                if ptr.is_null() {
                    None
                } else {
                    CStr::from_ptr(ptr).to_str().ok().map(|s| s.to_owned())
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
        unsafe {
            if let Some(destroy) = (*self.api).destroy_session {
                destroy(self.session);
            }
            if let Some(finalize) = (*self.api).finalize {
                finalize();
            }
        }
    }
}

extern "C" fn rime_notification_callback(
    _context_object: *mut std::ffi::c_void,
    _session_id: RimeSessionId,
    message_type: *const std::os::raw::c_char,
    message_value: *const std::os::raw::c_char,
) {
    unsafe {
        let typ = if message_type.is_null() {
            ""
        } else {
            std::ffi::CStr::from_ptr(message_type)
                .to_str()
                .unwrap_or("")
        };
        let val = if message_value.is_null() {
            ""
        } else {
            std::ffi::CStr::from_ptr(message_value)
                .to_str()
                .unwrap_or("")
        };
        println!("Xime notification: {} = {}", typ, val);
    }
}

unsafe fn c_str_to_string(ptr: *mut i8) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    CStr::from_ptr(ptr).to_str().ok().map(|s| s.to_owned())
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
    ApiFunctionMissing(&'static str),
    SessionCreateFailed,
    LockFailed,
}

impl std::fmt::Display for RimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RimeError::ApiNotFound => write!(f, "Rime API not found (rime.dll missing)"),
            RimeError::ApiFunctionMissing(name) => {
                write!(f, "Rime API function '{}' not available", name)
            }
            RimeError::SessionCreateFailed => write!(f, "Failed to create Rime session"),
            RimeError::LockFailed => write!(f, "Failed to acquire initialization lock"),
        }
    }
}

impl std::error::Error for RimeError {}
