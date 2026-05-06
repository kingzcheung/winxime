#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::os::raw::{c_char, c_int, c_void};
use std::mem::MaybeUninit;

#[macro_export]
macro_rules! rime_struct {
    ($var:ident : $t:ty) => {
        let $var = MaybeUninit::<$t>::zeroed();
        let mut $var = unsafe { $var.assume_init() };
        $var.data_size = (std::mem::size_of::<$t>() - std::mem::size_of_val(&$var.data_size)) as c_int;
    };
}

#[repr(C)]
pub struct RimeString {
    pub str: *mut c_char,
}

#[repr(C)]
pub struct RimeComposition {
    pub length: c_int,
    pub cursor_pos: c_int,
    pub sel_start: c_int,
    pub sel_end: c_int,
    pub preedit: *mut c_char,
}

#[repr(C)]
pub struct RimeCandidate {
    pub text: *mut c_char,
    pub comment: *mut c_char,
}

#[repr(C)]
pub struct RimeMenu {
    pub page_size: c_int,
    pub page_no: c_int,
    pub is_last_page: c_int,
    pub highlighted_candidate_index: c_int,
    pub num_candidates: c_int,
    pub candidates: *mut RimeCandidate,
    pub select_keys: *mut c_char,
}

#[repr(C)]
pub struct RimeContext {
    pub data_size: c_int,
    pub composition: RimeComposition,
    pub menu: RimeMenu,
    pub commit: *mut RimeCommit,
    pub select_labels: *mut *mut c_char,
}

#[repr(C)]
pub struct RimeCommit {
    pub data_size: c_int,
    pub text: *mut c_char,
}

#[repr(C)]
pub struct RimeStatus {
    pub data_size: c_int,
    pub schema_id: *mut c_char,
    pub schema_name: *mut c_char,
    pub is_disabled: c_int,
    pub is_composing: c_int,
    pub is_ascii_mode: c_int,
    pub is_full_shape: c_int,
    pub is_simplified: c_int,
    pub is_traditional: c_int,
    pub is_ascii_punct: c_int,
}

#[repr(C)]
pub struct RimeTraits {
    pub data_size: c_int,
    pub shared_data_dir: *mut c_char,
    pub user_data_dir: *mut c_char,
    pub distribution_name: *mut c_char,
    pub distribution_code_name: *mut c_char,
    pub distribution_version: *mut c_char,
    pub app_name: *mut c_char,
    pub min_log_level: c_int,
    pub log_dir: *mut c_char,
    pub prebuilt_data_dir: *mut c_char,
    pub staging_dir: *mut c_char,
}

pub type RimeSessionId = c_int;

#[repr(C)]
pub struct RimeApi {
    pub data_size: c_int,
    pub get_api: Option<unsafe extern "C" fn() -> *const RimeApi>,
    
    pub setup: Option<unsafe extern "C" fn(*mut RimeTraits)>,
    pub initialize: Option<unsafe extern "C" fn(*mut RimeTraits)>,
    pub finalize: Option<unsafe extern "C" fn()>,
    
    pub start_maintenance: Option<unsafe extern "C" fn(c_int) -> c_int>,
    pub is_maintenance_mode: Option<unsafe extern "C" fn() -> c_int>,
    pub join_maintenance_thread: Option<unsafe extern "C" fn()>,
    
    pub create_session: Option<unsafe extern "C" fn() -> RimeSessionId>,
    pub destroy_session: Option<unsafe extern "C" fn(RimeSessionId) -> c_int>,
    pub find_session: Option<unsafe extern "C" fn(RimeSessionId) -> c_int>,
    
    pub process_key: Option<unsafe extern "C" fn(RimeSessionId, c_int, c_int) -> c_int>,
    pub commit_text: Option<unsafe extern "C" fn(RimeSessionId) -> c_int>,
    
    pub get_context: Option<unsafe extern "C" fn(RimeSessionId, *mut RimeContext) -> c_int>,
    pub get_commit: Option<unsafe extern "C" fn(RimeSessionId, *mut RimeCommit) -> c_int>,
    pub get_status: Option<unsafe extern "C" fn(RimeSessionId, *mut RimeStatus) -> c_int>,
    
    pub free_commit: Option<unsafe extern "C" fn(*mut RimeCommit)>,
    pub free_context: Option<unsafe extern "C" fn(*mut RimeContext)>,
    pub free_status: Option<unsafe extern "C" fn(*mut RimeStatus)>,
    
    pub set_notification_handler: Option<unsafe extern "C" fn(
        Option<unsafe extern "C" fn(*mut c_void, RimeSessionId, *const c_char, *const c_char)>,
        *mut c_void,
    )>,
    
    pub simulate_key_sequence: Option<unsafe extern "C" fn(RimeSessionId, *const c_char) -> c_int>,
    pub select_schema: Option<unsafe extern "C" fn(RimeSessionId, *const c_char) -> c_int>,
}

#[cfg(target_os = "windows")]
pub fn rime_get_api() -> Option<*const RimeApi> {
    use std::ptr;
    use windows::Win32::System::LibraryLoader::{LoadLibraryA, GetProcAddress};
    use windows::core::PCSTR;
    use std::ffi::CString;
    
    static mut API_CACHE: *const RimeApi = ptr::null();
    
    unsafe {
        if !API_CACHE.is_null() {
            return Some(API_CACHE);
        }
        
        let lib_name = CString::new("rime.dll").unwrap();
        let hmodule = LoadLibraryA(PCSTR(lib_name.as_ptr() as *const u8));
        
        if let Ok(hmodule) = hmodule {
            let fn_name = CString::new("rime_get_api").unwrap();
            let proc = GetProcAddress(hmodule, PCSTR(fn_name.as_ptr() as *const u8));
            
            if let Some(proc) = proc {
                let get_api: extern "C" fn() -> *const RimeApi = std::mem::transmute(proc);
                API_CACHE = get_api();
                return Some(API_CACHE);
            }
        }
        None
    }
}

#[cfg(not(target_os = "windows"))]
pub fn rime_get_api() -> Option<*const RimeApi> {
    None
}

pub const RIME_MODIFIER_SHIFT: c_int = 1 << 0;
pub const RIME_MODIFIER_LOCK: c_int = 1 << 1;
pub const RIME_MODIFIER_CTRL: c_int = 1 << 2;
pub const RIME_MODIFIER_ALT: c_int = 1 << 3;
pub const RIME_MODIFIER_RELEASE: c_int = 1 << 30;

pub const XK_BackSpace: c_int = 65288;
pub const XK_Tab: c_int = 65289;
pub const XK_Return: c_int = 65293;
pub const XK_Escape: c_int = 65307;
pub const XK_Delete: c_int = 65535;
pub const XK_space: c_int = 32;