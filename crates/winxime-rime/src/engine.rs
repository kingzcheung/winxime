use librime_sys::*;
use std::ffi::CStr;

pub struct RimeEngine {
    api: *const RimeApi,
    session: RimeSessionId,
}

impl RimeEngine {
    pub fn new() -> Result<Self, RimeError> {
        let api = rime_get_api().ok_or(RimeError::ApiNotFound)?;
        
        unsafe {
            rime_struct!(traits: RimeTraits);
            
            if let Some(init) = (*api).initialize {
                init(&mut traits);
            }
            
            let session = if let Some(create) = (*api).create_session {
                create()
            } else {
                0
            };
            
            if session == 0 {
                return Err(RimeError::SessionCreateFailed);
            }
            
            Ok(Self { api, session })
        }
    }
    
    pub fn process_key(&mut self, keycode: i32, modifiers: i32) -> bool {
        unsafe {
            if let Some(process) = (*self.api).process_key {
                process(self.session, keycode, modifiers) != 0
            } else {
                false
            }
        }
    }
    
    pub fn get_composition(&self) -> Option<Composition> {
        unsafe {
            rime_struct!(ctx: RimeContext);
            
            if let Some(get_ctx) = (*self.api).get_context {
                if get_ctx(self.session, &mut ctx) == 0 {
                    if let Some(free) = (*self.api).free_context {
                        free(&mut ctx);
                    }
                    return None;
                }
            }
            
            let composition = Composition {
                length: ctx.composition.length as usize,
                cursor_pos: ctx.composition.cursor_pos as usize,
                sel_start: ctx.composition.sel_start as usize,
                sel_end: ctx.composition.sel_end as usize,
                preedit: to_c_str_nullable(ctx.composition.preedit),
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
                if get_ctx(self.session, &mut ctx) == 0 {
                    if let Some(free) = (*self.api).free_context {
                        free(&mut ctx);
                    }
                    return Vec::new();
                }
            }
            
            let num = ctx.menu.num_candidates as usize;
            let mut candidates = Vec::with_capacity(num);
            
            for i in 0..num {
                let candidate_ptr = ctx.menu.candidates.add(i);
                if !candidate_ptr.is_null() {
                    let text = to_c_str((*candidate_ptr).text);
                    let comment = to_c_str_nullable((*candidate_ptr).comment);
                    candidates.push(Candidate { text, comment });
                }
            }
            
            if let Some(free) = (*self.api).free_context {
                free(&mut ctx);
            }
            
            candidates
        }
    }
    
    pub fn get_commit(&self) -> Option<String> {
        unsafe {
            rime_struct!(commit: RimeCommit);
            
            if let Some(get_commit) = (*self.api).get_commit {
                if get_commit(self.session, &mut commit) == 0 {
                    if let Some(free) = (*self.api).free_commit {
                        free(&mut commit);
                    }
                    return None;
                }
            }
            
            let text = to_c_str_nullable(commit.text);
            if let Some(free) = (*self.api).free_commit {
                free(&mut commit);
            }
            text
        }
    }
    
    pub fn is_composing(&self) -> bool {
        unsafe {
            rime_struct!(status: RimeStatus);
            
            if let Some(get_status) = (*self.api).get_status {
                if get_status(self.session, &mut status) == 0 {
                    if let Some(free) = (*self.api).free_status {
                        free(&mut status);
                    }
                    return false;
                }
            }
            
            let composing = status.is_composing != 0;
            if let Some(free) = (*self.api).free_status {
                free(&mut status);
            }
            composing
        }
    }
}

impl Drop for RimeEngine {
    fn drop(&mut self) {
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

unsafe fn to_c_str(ptr: *mut std::os::raw::c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    CStr::from_ptr(ptr).to_string_lossy().into_owned()
}

unsafe fn to_c_str_nullable(ptr: *mut std::os::raw::c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
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
    SessionCreateFailed,
}

impl std::fmt::Display for RimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RimeError::ApiNotFound => write!(f, "Rime API not found (rime.dll missing)"),
            RimeError::SessionCreateFailed => write!(f, "Failed to create Rime session"),
        }
    }
}

impl std::error::Error for RimeError {}