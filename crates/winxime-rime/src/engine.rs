use librime_sys::*;
use std::ffi::{CStr, CString};
use std::path::Path;
use std::sync::Mutex;

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

            // Set notification handler for deploy/option events
            if let Some(set_handler) = (*api).set_notification_handler {
                set_handler(Some(rime_notification_callback), std::ptr::null_mut());
            }

            let session = if let Some(create) = (*api).create_session {
                create()
            } else {
                return Err(RimeError::ApiFunctionMissing("create_session"));
            };

            if session == 0 {
                return Err(RimeError::SessionCreateFailed);
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

    pub fn get_candidates(&self) -> Vec<Candidate> {
        unsafe {
            rime_struct!(ctx: RimeContext);

            if let Some(get_ctx) = (*self.api).get_context {
                if get_ctx(self.session, &mut ctx) == FALSE {
                    if let Some(free) = (*self.api).free_context {
                        free(&mut ctx);
                    }
                    return Vec::new();
                }
            } else {
                return Vec::new();
            }

            let num = ctx.menu.num_candidates as usize;
            let mut candidates = Vec::with_capacity(num);

            for i in 0..num {
                let candidate_ptr = ctx.menu.candidates.add(i);
                let text = c_str_to_string((*candidate_ptr).text).unwrap_or_default();
                let comment = c_str_to_string((*candidate_ptr).comment);
                candidates.push(Candidate { text, comment });
            }

            if let Some(free) = (*self.api).free_context {
                free(&mut ctx);
            }

            candidates
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
                set_opt(self.session, c_opt.as_ptr(), if value { TRUE } else { FALSE });
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
            std::ffi::CStr::from_ptr(message_type).to_str().unwrap_or("")
        };
        let val = if message_value.is_null() {
            ""
        } else {
            std::ffi::CStr::from_ptr(message_value).to_str().unwrap_or("")
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
