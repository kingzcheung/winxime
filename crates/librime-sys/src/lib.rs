#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::os::raw::{c_char, c_int, c_void};

pub type Bool = c_int;

pub type RimeSessionId = usize;

pub const FALSE: Bool = 0;
pub const TRUE: Bool = 1;

#[macro_export]
macro_rules! rime_struct {
    ($var:ident : $t:ty) => {
        let $var = std::mem::MaybeUninit::<$t>::zeroed();
        let mut $var = unsafe { $var.assume_init() };
        $var.data_size = (std::mem::size_of::<$t>() - std::mem::size_of_val(&$var.data_size))
            as std::os::raw::c_int;
    };
}

#[repr(C)]
pub struct RimeTraits {
    pub data_size: c_int,
    pub shared_data_dir: *const c_char,
    pub user_data_dir: *const c_char,
    pub distribution_name: *const c_char,
    pub distribution_code_name: *const c_char,
    pub distribution_version: *const c_char,
    pub app_name: *const c_char,
    pub modules: *const *const c_char,
    pub min_log_level: c_int,
    pub log_dir: *const c_char,
    pub prebuilt_data_dir: *const c_char,
    pub staging_dir: *const c_char,
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
    pub reserved: *mut c_void,
}

#[repr(C)]
pub struct RimeMenu {
    pub page_size: c_int,
    pub page_no: c_int,
    pub is_last_page: Bool,
    pub highlighted_candidate_index: c_int,
    pub num_candidates: c_int,
    pub candidates: *mut RimeCandidate,
    pub select_keys: *mut c_char,
}

#[repr(C)]
pub struct RimeCommit {
    pub data_size: c_int,
    pub text: *mut c_char,
}

#[repr(C)]
pub struct RimeContext {
    pub data_size: c_int,
    pub composition: RimeComposition,
    pub menu: RimeMenu,
    pub commit_text_preview: *mut c_char,
    pub select_labels: *mut *mut c_char,
}

#[repr(C)]
pub struct RimeStatus {
    pub data_size: c_int,
    pub schema_id: *mut c_char,
    pub schema_name: *mut c_char,
    pub is_disabled: Bool,
    pub is_composing: Bool,
    pub is_ascii_mode: Bool,
    pub is_full_shape: Bool,
    pub is_simplified: Bool,
    pub is_traditional: Bool,
    pub is_ascii_punct: Bool,
}

#[repr(C)]
pub struct RimeCandidateListIterator {
    pub ptr: *mut c_void,
    pub index: c_int,
    pub candidate: RimeCandidate,
}

#[repr(C)]
pub struct RimeConfig {
    pub ptr: *mut c_void,
}

#[repr(C)]
pub struct RimeConfigIterator {
    pub list: *mut c_void,
    pub map: *mut c_void,
    pub index: c_int,
    pub key: *const c_char,
    pub path: *const c_char,
}

#[repr(C)]
pub struct RimeSchemaListItem {
    pub schema_id: *mut c_char,
    pub name: *mut c_char,
    pub reserved: *mut c_void,
}

#[repr(C)]
pub struct RimeSchemaList {
    pub size: usize,
    pub list: *mut RimeSchemaListItem,
}

#[repr(C)]
pub struct RimeStringSlice {
    pub str: *const c_char,
    pub length: usize,
}

#[repr(C)]
pub struct RimeCustomApi {
    pub data_size: c_int,
}

#[repr(C)]
pub struct RimeModule {
    pub data_size: c_int,
    pub module_name: *const c_char,
    pub initialize: Option<unsafe extern "C" fn()>,
    pub finalize: Option<unsafe extern "C" fn()>,
    pub get_api: Option<unsafe extern "C" fn() -> *mut RimeCustomApi>,
}

pub type RimeNotificationHandler = Option<
    unsafe extern "C" fn(
        context_object: *mut c_void,
        session_id: RimeSessionId,
        message_type: *const c_char,
        message_value: *const c_char,
    ),
>;

#[repr(C)]
pub struct RimeApi {
    pub data_size: c_int,

    // 1
    pub setup: Option<unsafe extern "C" fn(traits: *mut RimeTraits)>,
    // 2
    pub set_notification_handler:
        Option<unsafe extern "C" fn(handler: RimeNotificationHandler, context_object: *mut c_void)>,
    // 3
    pub initialize: Option<unsafe extern "C" fn(traits: *mut RimeTraits)>,
    // 4
    pub finalize: Option<unsafe extern "C" fn()>,
    // 5
    pub start_maintenance: Option<unsafe extern "C" fn(full_check: Bool) -> Bool>,
    // 6
    pub is_maintenance_mode: Option<unsafe extern "C" fn() -> Bool>,
    // 7
    pub join_maintenance_thread: Option<unsafe extern "C" fn()>,

    // 8
    pub deployer_initialize: Option<unsafe extern "C" fn(traits: *mut RimeTraits)>,
    // 9
    pub prebuild: Option<unsafe extern "C" fn() -> Bool>,
    // 10
    pub deploy: Option<unsafe extern "C" fn() -> Bool>,
    // 11
    pub deploy_schema: Option<unsafe extern "C" fn(schema_file: *const c_char) -> Bool>,
    // 12
    pub deploy_config_file:
        Option<unsafe extern "C" fn(file_name: *const c_char, version_key: *const c_char) -> Bool>,
    // 13
    pub sync_user_data: Option<unsafe extern "C" fn() -> Bool>,

    // 14
    pub create_session: Option<unsafe extern "C" fn() -> RimeSessionId>,
    // 15
    pub find_session: Option<unsafe extern "C" fn(session_id: RimeSessionId) -> Bool>,
    // 16
    pub destroy_session: Option<unsafe extern "C" fn(session_id: RimeSessionId) -> Bool>,
    // 17
    pub cleanup_stale_sessions: Option<unsafe extern "C" fn()>,
    // 18
    pub cleanup_all_sessions: Option<unsafe extern "C" fn()>,

    // 19
    pub process_key: Option<
        unsafe extern "C" fn(session_id: RimeSessionId, keycode: c_int, mask: c_int) -> Bool,
    >,
    // 20
    pub commit_composition: Option<unsafe extern "C" fn(session_id: RimeSessionId) -> Bool>,
    // 21
    pub clear_composition: Option<unsafe extern "C" fn(session_id: RimeSessionId)>,

    // 22
    pub get_commit:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, commit: *mut RimeCommit) -> Bool>,
    // 23
    pub free_commit: Option<unsafe extern "C" fn(commit: *mut RimeCommit) -> Bool>,
    // 24
    pub get_context:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, context: *mut RimeContext) -> Bool>,
    // 25
    pub free_context: Option<unsafe extern "C" fn(ctx: *mut RimeContext) -> Bool>,
    // 26
    pub get_status:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, status: *mut RimeStatus) -> Bool>,
    // 27
    pub free_status: Option<unsafe extern "C" fn(status: *mut RimeStatus) -> Bool>,

    // 28
    pub set_option:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, option: *const c_char, value: Bool)>,
    // 29
    pub get_option:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, option: *const c_char) -> Bool>,

    // 30
    pub set_property: Option<
        unsafe extern "C" fn(session_id: RimeSessionId, prop: *const c_char, value: *const c_char),
    >,
    // 31
    pub get_property: Option<
        unsafe extern "C" fn(
            session_id: RimeSessionId,
            prop: *const c_char,
            value: *mut c_char,
            buffer_size: usize,
        ) -> Bool,
    >,

    // 32
    pub get_schema_list: Option<unsafe extern "C" fn(schema_list: *mut RimeSchemaList) -> Bool>,
    // 33
    pub free_schema_list: Option<unsafe extern "C" fn(schema_list: *mut RimeSchemaList)>,

    // 34
    pub get_current_schema: Option<
        unsafe extern "C" fn(
            session_id: RimeSessionId,
            schema_id: *mut c_char,
            buffer_size: usize,
        ) -> Bool,
    >,
    // 35
    pub select_schema:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, schema_id: *const c_char) -> Bool>,

    // 36
    pub schema_open:
        Option<unsafe extern "C" fn(schema_id: *const c_char, config: *mut RimeConfig) -> Bool>,
    // 37
    pub config_open:
        Option<unsafe extern "C" fn(config_id: *const c_char, config: *mut RimeConfig) -> Bool>,
    // 38
    pub config_close: Option<unsafe extern "C" fn(config: *mut RimeConfig) -> Bool>,
    // 39
    pub config_get_bool: Option<
        unsafe extern "C" fn(config: *mut RimeConfig, key: *const c_char, value: *mut Bool) -> Bool,
    >,
    // 40
    pub config_get_int: Option<
        unsafe extern "C" fn(
            config: *mut RimeConfig,
            key: *const c_char,
            value: *mut c_int,
        ) -> Bool,
    >,
    // 41
    pub config_get_double: Option<
        unsafe extern "C" fn(config: *mut RimeConfig, key: *const c_char, value: *mut f64) -> Bool,
    >,
    // 42
    pub config_get_string: Option<
        unsafe extern "C" fn(
            config: *mut RimeConfig,
            key: *const c_char,
            value: *mut c_char,
            buffer_size: usize,
        ) -> Bool,
    >,
    // 43
    pub config_get_cstring:
        Option<unsafe extern "C" fn(config: *mut RimeConfig, key: *const c_char) -> *const c_char>,
    // 44
    pub config_update_signature:
        Option<unsafe extern "C" fn(config: *mut RimeConfig, signer: *const c_char) -> Bool>,
    // 45
    pub config_begin_map: Option<
        unsafe extern "C" fn(
            iterator: *mut RimeConfigIterator,
            config: *mut RimeConfig,
            key: *const c_char,
        ) -> Bool,
    >,
    // 46
    pub config_next: Option<unsafe extern "C" fn(iterator: *mut RimeConfigIterator) -> Bool>,
    // 47
    pub config_end: Option<unsafe extern "C" fn(iterator: *mut RimeConfigIterator)>,

    // 48
    pub simulate_key_sequence: Option<
        unsafe extern "C" fn(session_id: RimeSessionId, key_sequence: *const c_char) -> Bool,
    >,

    // 49
    pub register_module: Option<unsafe extern "C" fn(module: *mut RimeModule) -> Bool>,
    // 50
    pub find_module: Option<unsafe extern "C" fn(module_name: *const c_char) -> *mut RimeModule>,

    // 51
    pub run_task: Option<unsafe extern "C" fn(task_name: *const c_char) -> Bool>,

    // 52-54 deprecated dir accessors
    pub get_shared_data_dir: Option<unsafe extern "C" fn() -> *const c_char>,
    pub get_user_data_dir: Option<unsafe extern "C" fn() -> *const c_char>,
    pub get_sync_dir: Option<unsafe extern "C" fn() -> *const c_char>,

    // 55
    pub get_user_id: Option<unsafe extern "C" fn() -> *const c_char>,
    // 56
    pub get_user_data_sync_dir: Option<unsafe extern "C" fn(dir: *mut c_char, buffer_size: usize)>,

    // 57
    pub config_init: Option<unsafe extern "C" fn(config: *mut RimeConfig) -> Bool>,
    // 58
    pub config_load_string:
        Option<unsafe extern "C" fn(config: *mut RimeConfig, yaml: *const c_char) -> Bool>,

    // 59-62 config setters
    pub config_set_bool: Option<
        unsafe extern "C" fn(config: *mut RimeConfig, key: *const c_char, value: Bool) -> Bool,
    >,
    pub config_set_int: Option<
        unsafe extern "C" fn(config: *mut RimeConfig, key: *const c_char, value: c_int) -> Bool,
    >,
    pub config_set_double: Option<
        unsafe extern "C" fn(config: *mut RimeConfig, key: *const c_char, value: f64) -> Bool,
    >,
    pub config_set_string: Option<
        unsafe extern "C" fn(
            config: *mut RimeConfig,
            key: *const c_char,
            value: *const c_char,
        ) -> Bool,
    >,

    // 63-69 config complex manipulation
    pub config_get_item: Option<
        unsafe extern "C" fn(
            config: *mut RimeConfig,
            key: *const c_char,
            value: *mut RimeConfig,
        ) -> Bool,
    >,
    pub config_set_item: Option<
        unsafe extern "C" fn(
            config: *mut RimeConfig,
            key: *const c_char,
            value: *mut RimeConfig,
        ) -> Bool,
    >,
    pub config_clear:
        Option<unsafe extern "C" fn(config: *mut RimeConfig, key: *const c_char) -> Bool>,
    pub config_create_list:
        Option<unsafe extern "C" fn(config: *mut RimeConfig, key: *const c_char) -> Bool>,
    pub config_create_map:
        Option<unsafe extern "C" fn(config: *mut RimeConfig, key: *const c_char) -> Bool>,
    pub config_list_size:
        Option<unsafe extern "C" fn(config: *mut RimeConfig, key: *const c_char) -> usize>,
    pub config_begin_list: Option<
        unsafe extern "C" fn(
            iterator: *mut RimeConfigIterator,
            config: *mut RimeConfig,
            key: *const c_char,
        ) -> Bool,
    >,

    // 70
    pub get_input: Option<unsafe extern "C" fn(session_id: RimeSessionId) -> *const c_char>,
    // 71
    pub get_caret_pos: Option<unsafe extern "C" fn(session_id: RimeSessionId) -> usize>,
    // 72
    pub select_candidate:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, index: usize) -> Bool>,
    // 73
    pub get_version: Option<unsafe extern "C" fn() -> *const c_char>,
    // 74
    pub set_caret_pos: Option<unsafe extern "C" fn(session_id: RimeSessionId, caret_pos: usize)>,
    // 75
    pub select_candidate_on_current_page:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, index: usize) -> Bool>,

    // 76-78 candidate list iterator
    pub candidate_list_begin: Option<
        unsafe extern "C" fn(
            session_id: RimeSessionId,
            iterator: *mut RimeCandidateListIterator,
        ) -> Bool,
    >,
    pub candidate_list_next:
        Option<unsafe extern "C" fn(iterator: *mut RimeCandidateListIterator) -> Bool>,
    pub candidate_list_end: Option<unsafe extern "C" fn(iterator: *mut RimeCandidateListIterator)>,

    // 79
    pub user_config_open:
        Option<unsafe extern "C" fn(config_id: *const c_char, config: *mut RimeConfig) -> Bool>,

    // 80
    pub candidate_list_from_index: Option<
        unsafe extern "C" fn(
            session_id: RimeSessionId,
            iterator: *mut RimeCandidateListIterator,
            index: c_int,
        ) -> Bool,
    >,

    // 81-82 deprecated dir accessors
    pub get_prebuilt_data_dir: Option<unsafe extern "C" fn() -> *const c_char>,
    pub get_staging_dir: Option<unsafe extern "C" fn() -> *const c_char>,

    // 83-85 proto (deprecated)
    pub commit_proto:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, commit_builder: *mut c_void)>,
    pub context_proto:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, context_builder: *mut c_void)>,
    pub status_proto:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, status_builder: *mut c_void)>,

    // 86
    pub get_state_label: Option<
        unsafe extern "C" fn(
            session_id: RimeSessionId,
            option_name: *const c_char,
            state: Bool,
        ) -> *const c_char,
    >,

    // 87
    pub delete_candidate:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, index: usize) -> Bool>,
    // 88
    pub delete_candidate_on_current_page:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, index: usize) -> Bool>,

    // 89
    pub get_state_label_abbreviated: Option<
        unsafe extern "C" fn(
            session_id: RimeSessionId,
            option_name: *const c_char,
            state: Bool,
            abbreviated: Bool,
        ) -> RimeStringSlice,
    >,

    // 90
    pub set_input:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, input: *const c_char) -> Bool>,

    // 91-95 dir accessors with buffer
    pub get_shared_data_dir_s: Option<unsafe extern "C" fn(dir: *mut c_char, buffer_size: usize)>,
    pub get_user_data_dir_s: Option<unsafe extern "C" fn(dir: *mut c_char, buffer_size: usize)>,
    pub get_prebuilt_data_dir_s: Option<unsafe extern "C" fn(dir: *mut c_char, buffer_size: usize)>,
    pub get_staging_dir_s: Option<unsafe extern "C" fn(dir: *mut c_char, buffer_size: usize)>,
    pub get_sync_dir_s: Option<unsafe extern "C" fn(dir: *mut c_char, buffer_size: usize)>,

    // 96
    pub highlight_candidate:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, index: usize) -> Bool>,
    // 97
    pub highlight_candidate_on_current_page:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, index: usize) -> Bool>,

    // 98
    pub change_page:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, backward: Bool) -> Bool>,
}

// the rime_get_api is a standalone function exported from the DLL
#[cfg(target_os = "windows")]
pub fn rime_get_api() -> Option<*const RimeApi> {
    use std::ffi::CString;
    use std::ptr;
    use windows::core::PCSTR;
    use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA, SetDllDirectoryW};

    static mut API_CACHE: *const RimeApi = ptr::null();

    unsafe {
        if !API_CACHE.is_null() {
            return Some(API_CACHE);
        }

        let exe_path = std::env::current_exe().ok()?;
        let exe_dir = exe_path.parent()?;
        let exe_dir_wide: Vec<u16> = exe_dir.to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect();
        
        let _ = SetDllDirectoryW(windows::core::PCWSTR(exe_dir_wide.as_ptr()));

        let lib_name = CString::new("rime.dll").ok()?;
        let hmodule = LoadLibraryA(PCSTR(lib_name.as_ptr() as *const u8));

        if let Ok(hmodule) = hmodule {
            let fn_name = CString::new("rime_get_api").ok()?;
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
pub const XK_Left: c_int = 65361;
pub const XK_Up: c_int = 65362;
pub const XK_Right: c_int = 65363;
pub const XK_Down: c_int = 65364;
pub const XK_Prior: c_int = 65365;
pub const XK_Next: c_int = 65366;
pub const XK_Home: c_int = 65360;
pub const XK_End: c_int = 65367;
pub const XK_Shift_L: c_int = 65505;
pub const XK_Shift_R: c_int = 65506;

// TSF virtual key codes
pub const VK_A: u16 = 0x41;
pub const VK_B: u16 = 0x42;
pub const VK_C: u16 = 0x43;
pub const VK_D: u16 = 0x44;
pub const VK_E: u16 = 0x45;
pub const VK_F: u16 = 0x46;
pub const VK_G: u16 = 0x47;
pub const VK_H: u16 = 0x48;
pub const VK_I: u16 = 0x49;
pub const VK_J: u16 = 0x4A;
pub const VK_K: u16 = 0x4B;
pub const VK_L: u16 = 0x4C;
pub const VK_M: u16 = 0x4D;
pub const VK_N: u16 = 0x4E;
pub const VK_O: u16 = 0x4F;
pub const VK_P: u16 = 0x50;
pub const VK_Q: u16 = 0x51;
pub const VK_R: u16 = 0x52;
pub const VK_S: u16 = 0x53;
pub const VK_T: u16 = 0x54;
pub const VK_U: u16 = 0x55;
pub const VK_V: u16 = 0x56;
pub const VK_W: u16 = 0x57;
pub const VK_X: u16 = 0x58;
pub const VK_Y: u16 = 0x59;
pub const VK_Z: u16 = 0x5A;
pub const VK_0: u16 = 0x30;
pub const VK_9: u16 = 0x39;
pub const VK_SPACE: u16 = 0x20;
pub const VK_RETURN: u16 = 0x0D;
pub const VK_BACK: u16 = 0x08;
pub const VK_ESCAPE: u16 = 0x1B;
pub const VK_DELETE: u16 = 0x2E;
pub const VK_UP: u16 = 0x26;
pub const VK_DOWN: u16 = 0x28;
pub const VK_LEFT: u16 = 0x25;
pub const VK_RIGHT: u16 = 0x27;
pub const VK_PRIOR: u16 = 0x21; // PageUp
pub const VK_NEXT: u16 = 0x22; // PageDown
pub const VK_SHIFT: u16 = 0x10;
pub const VK_CONTROL: u16 = 0x11;
pub const VK_MENU: u16 = 0x12;

pub const XK_a: c_int = 0x61;
pub const XK_b: c_int = 0x62;
pub const XK_c: c_int = 0x63;
pub const XK_d: c_int = 0x64;
pub const XK_e: c_int = 0x65;
pub const XK_f: c_int = 0x66;
pub const XK_g: c_int = 0x67;
pub const XK_h: c_int = 0x68;
pub const XK_i: c_int = 0x69;
pub const XK_j: c_int = 0x6A;
pub const XK_k: c_int = 0x6B;
pub const XK_l: c_int = 0x6C;
pub const XK_m: c_int = 0x6D;
pub const XK_n: c_int = 0x6E;
pub const XK_o: c_int = 0x6F;
pub const XK_p: c_int = 0x70;
pub const XK_q: c_int = 0x71;
pub const XK_r: c_int = 0x72;
pub const XK_s: c_int = 0x73;
pub const XK_t: c_int = 0x74;
pub const XK_u: c_int = 0x75;
pub const XK_v: c_int = 0x76;
pub const XK_w: c_int = 0x77;
pub const XK_x: c_int = 0x78;
pub const XK_y: c_int = 0x79;
pub const XK_z: c_int = 0x7A;

pub fn vk_to_xk(vk: u16) -> c_int {
    match vk {
        VK_SPACE => XK_space,
        VK_RETURN => XK_Return,
        VK_BACK => XK_BackSpace,
        VK_ESCAPE => XK_Escape,
        VK_DELETE => XK_Delete,
        VK_LEFT => XK_Left,
        VK_UP => XK_Up,
        VK_RIGHT => XK_Right,
        VK_DOWN => XK_Down,
        VK_PRIOR => XK_Prior,
        VK_NEXT => XK_Next,
        VK_SHIFT => XK_Shift_L,
        VK_A => XK_a,
        VK_B => XK_b,
        VK_C => XK_c,
        VK_D => XK_d,
        VK_E => XK_e,
        VK_F => XK_f,
        VK_G => XK_g,
        VK_H => XK_h,
        VK_I => XK_i,
        VK_J => XK_j,
        VK_K => XK_k,
        VK_L => XK_l,
        VK_M => XK_m,
        VK_N => XK_n,
        VK_O => XK_o,
        VK_P => XK_p,
        VK_Q => XK_q,
        VK_R => XK_r,
        VK_S => XK_s,
        VK_T => XK_t,
        VK_U => XK_u,
        VK_V => XK_v,
        VK_W => XK_w,
        VK_X => XK_x,
        VK_Y => XK_y,
        VK_Z => XK_z,
        VK_0..=VK_9 => vk as c_int,
        _ => vk as c_int,
    }
}

// Get modifier mask from TSF key event
pub fn get_key_modifiers() -> c_int {
    unsafe {
        let mut mods = 0;
        if GetAsyncKeyState(VK_SHIFT as c_int) < 0 {
            mods |= RIME_MODIFIER_SHIFT;
        }
        if GetAsyncKeyState(VK_CONTROL as c_int) < 0 {
            mods |= RIME_MODIFIER_CTRL;
        }
        if GetAsyncKeyState(VK_MENU as c_int) < 0 {
            mods |= RIME_MODIFIER_ALT;
        }
        mods
    }
}

extern "system" {
    fn GetAsyncKeyState(vKey: c_int) -> i16;
}
