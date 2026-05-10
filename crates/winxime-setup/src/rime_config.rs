use librime_sys::*;
use std::ffi::{CStr, CString};
use std::os::raw::c_int;
use std::ptr;
use std::sync::Once;

static RIME_INIT: Once = Once::new();

fn init_rime_deployer() -> Result<(), String> {
    RIME_INIT.call_once(|| {
        unsafe {
            let api = rime_get_api();
            if api.is_none() {
                return;
            }
            let api = api.unwrap();

            let (shared_data_dir, user_data_dir) = get_data_dirs();
            
            ensure_user_config_files(&user_data_dir);
            
            let shared = CString::new(shared_data_dir.to_str().unwrap_or("")).unwrap_or_default();
            let user = CString::new(user_data_dir.to_str().unwrap_or("")).unwrap_or_default();
            let dist_name = CString::new("Xime").unwrap_or_default();
            let app_name = CString::new("rime.xime.setup").unwrap_or_default();

            rime_struct!(traits: RimeTraits);
            traits.shared_data_dir = shared.as_ptr();
            traits.user_data_dir = user.as_ptr();
            traits.distribution_name = dist_name.as_ptr();
            traits.distribution_code_name = dist_name.as_ptr();
            traits.distribution_version = b"1.0\0".as_ptr() as *const i8;
            traits.app_name = app_name.as_ptr();
            traits.min_log_level = 2;

            if let Some(setup) = (*api).setup {
                setup(&mut traits);
            }

            if let Some(init) = (*api).initialize {
                init(&mut traits);
            }

            if let Some(deployer_init) = (*api).deployer_initialize {
                deployer_init(ptr::null_mut());
            }

            if let Some(deploy) = (*api).deploy {
                deploy();
            }

            if let Some(deploy_config) = (*api).deploy_config_file {
                let config_file = CString::new("xime.yaml").unwrap_or_default();
                let version_key = CString::new("config_version").unwrap_or_default();
                deploy_config(config_file.as_ptr(), version_key.as_ptr());
            }
        }
    });

    if rime_get_api().is_none() {
        return Err("Rime API not available".to_string());
    }

    Ok(())
}

fn ensure_user_config_files(user_data_dir: &std::path::Path) {
    if !user_data_dir.exists() {
        std::fs::create_dir_all(user_data_dir).ok();
    }
    
    let config_source_dir = get_config_source_dir();
    
    let xime_yaml = user_data_dir.join("xime.yaml");
    if !xime_yaml.exists() {
        let source = config_source_dir.join("xime.yaml");
        if source.exists() {
            std::fs::copy(&source, &xime_yaml).ok();
        }
    }
    
    let default_custom = user_data_dir.join("default.custom.yaml");
    if !default_custom.exists() {
        std::fs::write(&default_custom, 
r#"customization:
  distribution_code_name: Xime
  distribution_version: 1.0
  generator: "Rime::SwitcherSettings"
  rime_version: 1.16.1

patch:
  schema_list:
    - schema: wubi86_jidian
"#).ok();
    }
    
    let xime_custom = user_data_dir.join("xime.custom.yaml");
    if !xime_custom.exists() {
        std::fs::write(&xime_custom, 
r#"customization:
  distribution_code_name: Xime
  distribution_version: 1.0
  generator: "Xime::ConfigManager"
  rime_version: 1.16.1

patch: {}
"#).ok();
    }
}

fn get_data_dirs() -> (std::path::PathBuf, std::path::PathBuf) {
    #[cfg(debug_assertions)]
    {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_dir = manifest_dir.parent().unwrap().parent().unwrap();
        (
            workspace_dir.join("config"),
            workspace_dir.join("target").join("debug").join("user-data"),
        )
    }

    #[cfg(not(debug_assertions))]
    {
        let exe_path = std::env::current_exe().ok().unwrap_or_else(|| std::path::PathBuf::from("."));
        let exe_dir = exe_path.parent().unwrap_or_else(|| std::path::Path::new("."));
        
        let user_data_dir = std::env::var("APPDATA")
            .ok()
            .map(|p| std::path::PathBuf::from(p).join("Rime"))
            .unwrap_or_else(|| exe_dir.join("user-data"));
        
        (
            exe_dir.join("config"),
            user_data_dir,
        )
    }
}

fn get_config_source_dir() -> std::path::PathBuf {
    get_data_dirs().0
}

pub struct RimeConfigManager {
    api: *const RimeLeversApi,
    settings: *mut RimeCustomSettings,
}

impl RimeConfigManager {
    pub fn new() -> Result<Self, String> {
        init_rime_deployer()?;

        let api = rime_get_levers_api();
        if api.is_none() {
            return Err("Failed to get Rime levers API".to_string());
        }
        let api = api.unwrap();

        unsafe {
            let init_func = (*api).custom_settings_init.ok_or("custom_settings_init not available")?;
            let config_id = CString::new("xime").map_err(|_| "Failed to create config_id string")?;
            let generator_id = CString::new("Xime::ConfigManager").map_err(|_| "Failed to create generator_id string")?;

            let settings = init_func(config_id.as_ptr(), generator_id.as_ptr());
            if settings.is_null() {
                return Err("Failed to initialize custom settings".to_string());
            }

            let load_func = (*api).load_settings.ok_or("load_settings not available")?;
            if load_func(settings) == FALSE {
                eprintln!("Warning: load_settings returned false, config may not exist yet");
            }

            Ok(Self { api, settings })
        }
    }

    fn get_config(&self) -> Option<RimeConfig> {
        unsafe {
            let get_config = (*self.api).settings_get_config?;
            let mut config = RimeConfig { ptr: ptr::null_mut() };
            if get_config(self.settings, &mut config) == FALSE {
                return None;
            }
            Some(config)
        }
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        let mut config = self.get_config()?;
        unsafe {
            let key_c = CString::new(key).ok()?;
            let rime_api = rime_get_api()?;
            let get_cstring = (*rime_api).config_get_cstring?;

            let value_ptr = get_cstring(&mut config, key_c.as_ptr());
            if value_ptr.is_null() {
                return None;
            }

            CStr::from_ptr(value_ptr).to_str().ok().map(|s| s.to_owned())
        }
    }

    pub fn get_int(&self, key: &str) -> Option<i32> {
        let mut config = self.get_config()?;
        unsafe {
            let key_c = CString::new(key).ok()?;
            let rime_api = rime_get_api()?;
            let get_int = (*rime_api).config_get_int?;

            let mut value: c_int = 0;
            if get_int(&mut config, key_c.as_ptr(), &mut value) == FALSE {
                return None;
            }

            Some(value)
        }
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        let mut config = self.get_config()?;
        unsafe {
            let key_c = CString::new(key).ok()?;
            let rime_api = rime_get_api()?;
            let get_bool = (*rime_api).config_get_bool?;

            let mut value: Bool = FALSE;
            if get_bool(&mut config, key_c.as_ptr(), &mut value) == FALSE {
                return None;
            }

            Some(value != FALSE)
        }
    }

    pub fn get_double(&self, key: &str) -> Option<f64> {
        let mut config = self.get_config()?;
        unsafe {
            let key_c = CString::new(key).ok()?;
            let rime_api = rime_get_api()?;
            let get_double = (*rime_api).config_get_double?;

            let mut value: f64 = 0.0;
            if get_double(&mut config, key_c.as_ptr(), &mut value) == FALSE {
                return None;
            }

            Some(value)
        }
    }

    pub fn set_string(&self, key: &str, value: &str) -> Result<(), String> {
        unsafe {
            let customize_string = (*self.api).customize_string.ok_or("customize_string not available")?;
            let key_c = CString::new(key).map_err(|_| "Failed to create key string")?;
            let value_c = CString::new(value).map_err(|_| "Failed to create value string")?;

            if customize_string(self.settings, key_c.as_ptr(), value_c.as_ptr()) == FALSE {
                return Err(format!("Failed to set {} = {}", key, value));
            }

            Ok(())
        }
    }

    pub fn set_int(&self, key: &str, value: i32) -> Result<(), String> {
        unsafe {
            let customize_int = (*self.api).customize_int.ok_or("customize_int not available")?;
            let key_c = CString::new(key).map_err(|_| "Failed to create key string")?;

            if customize_int(self.settings, key_c.as_ptr(), value) == FALSE {
                return Err(format!("Failed to set {} = {}", key, value));
            }

            Ok(())
        }
    }

    pub fn set_bool(&self, key: &str, value: bool) -> Result<(), String> {
        unsafe {
            let customize_bool = (*self.api).customize_bool.ok_or("customize_bool not available")?;
            let key_c = CString::new(key).map_err(|_| "Failed to create key string")?;
            let bool_value = if value { TRUE } else { FALSE };

            if customize_bool(self.settings, key_c.as_ptr(), bool_value) == FALSE {
                return Err(format!("Failed to set {} = {}", key, value));
            }

            Ok(())
        }
    }

    pub fn set_double(&self, key: &str, value: f64) -> Result<(), String> {
        unsafe {
            let customize_double = (*self.api).customize_double.ok_or("customize_double not available")?;
            let key_c = CString::new(key).map_err(|_| "Failed to create key string")?;

            if customize_double(self.settings, key_c.as_ptr(), value) == FALSE {
                return Err(format!("Failed to set {} = {}", key, value));
            }

            Ok(())
        }
    }

    pub fn save(&self) -> Result<(), String> {
        unsafe {
            let save_settings = (*self.api).save_settings.ok_or("save_settings not available")?;

            if save_settings(self.settings) == FALSE {
                return Err("Failed to save settings".to_string());
            }

            Ok(())
        }
    }
}

impl Drop for RimeConfigManager {
    fn drop(&mut self) {
        unsafe {
            if let Some(destroy_settings) = (*self.api).custom_settings_destroy {
                destroy_settings(self.settings);
            }
        }
    }
}

pub fn deploy_all() -> Result<(), String> {
    init_rime_deployer()?;
    
    unsafe {
        let api = rime_get_api().ok_or("Rime API not available")?;
        
        if let Some(deploy) = (*api).deploy {
            if deploy() == FALSE {
                return Err("Deploy failed".to_string());
            }
        }
        
        if let Some(deploy_config) = (*api).deploy_config_file {
            let xime_yaml = CString::new("xime.yaml").unwrap_or_default();
            let version_key = CString::new("config_version").unwrap_or_default();
            deploy_config(xime_yaml.as_ptr(), version_key.as_ptr());
        }
    }
    
    Ok(())
}

pub struct SchemaManager {
    api: *const RimeLeversApi,
    settings: *mut RimeSwitcherSettings,
}

impl SchemaManager {
    pub fn new() -> Result<Self, String> {
        init_rime_deployer()?;
        
        let api = rime_get_levers_api().ok_or("Levers API not available")?;
        
        unsafe {
            let init_func = (*api).switcher_settings_init.ok_or("switcher_settings_init not available")?;
            let settings = init_func();
            if settings.is_null() {
                return Err("Failed to init switcher settings".to_string());
            }
            
            let load_func = (*api).load_settings.ok_or("load_settings not available")?;
            load_func(settings as *mut RimeCustomSettings);
            
            Ok(Self { api, settings })
        }
    }
    
    pub fn get_schema_list(&self) -> Vec<SchemaInfo> {
        unsafe {
            let get_list = (*self.api).get_available_schema_list;
            if get_list.is_none() {
                return Vec::new();
            }
            
            let mut list = RimeSchemaList { size: 0, list: ptr::null_mut() };
            if get_list.unwrap()(self.settings, &mut list) == FALSE {
                return Vec::new();
            }
            
            if list.list.is_null() {
                return Vec::new();
            }
            
            let schemas: Vec<SchemaInfo> = (0..list.size)
                .filter_map(|i| {
                    let item = list.list.add(i);
                    let schema_id_ptr = (*item).schema_id;
                    if schema_id_ptr.is_null() {
                        return None;
                    }
                    let schema_id = CStr::from_ptr(schema_id_ptr).to_string_lossy().to_string();
                    let name_ptr = (*item).name;
                    let name = if name_ptr.is_null() {
                        schema_id.clone()
                    } else {
                        CStr::from_ptr(name_ptr).to_string_lossy().to_string()
                    };
                    Some(SchemaInfo { schema_id, name })
                })
                .collect();
            
            if let Some(destroy) = (*self.api).schema_list_destroy {
                destroy(&mut list);
            }
            
            schemas
        }
    }
    
    pub fn set_schema_list(&self, schema_ids: &[&str]) -> Result<(), String> {
        unsafe {
            let select_func = (*self.api).select_schemas.ok_or("select_schemas not available")?;
            
            let c_strings: Vec<CString> = schema_ids
                .iter()
                .filter_map(|id| CString::new(*id).ok())
                .collect();
            
            let ptrs: Vec<*const i8> = c_strings.iter().map(|s| s.as_ptr()).collect();
            
            if select_func(self.settings, ptrs.as_ptr(), ptrs.len() as i32) == FALSE {
                return Err("Failed to set schema list".to_string());
            }
            
            Ok(())
        }
    }
    
    pub fn save(&self) -> Result<(), String> {
        unsafe {
            let save_func = (*self.api).save_settings.ok_or("save_settings not available")?;
            if save_func(self.settings as *mut RimeCustomSettings) == FALSE {
                return Err("Failed to save schema settings".to_string());
            }
            Ok(())
        }
    }
    
    pub fn get_selected_schema(&self) -> Option<String> {
        unsafe {
            let get_selected = (*self.api).get_selected_schema_list?;
            let mut list = RimeSchemaList { size: 0, list: ptr::null_mut() };
            if get_selected(self.settings, &mut list) == FALSE || list.size == 0 || list.list.is_null() {
                return None;
            }
            let schema_id_ptr = (*list.list).schema_id;
            if schema_id_ptr.is_null() {
                return None;
            }
            let schema_id = CStr::from_ptr(schema_id_ptr).to_string_lossy().to_string();
            if let Some(destroy) = (*self.api).schema_list_destroy {
                destroy(&mut list);
            }
            Some(schema_id)
        }
    }
}

impl Drop for SchemaManager {
    fn drop(&mut self) {
        unsafe {
            if let Some(destroy) = (*self.api).custom_settings_destroy {
                destroy(self.settings as *mut RimeCustomSettings);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct SchemaInfo {
    pub schema_id: String,
    pub name: String,
}