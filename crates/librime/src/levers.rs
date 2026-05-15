use crate::get_api;
use crate::error::{Error, Result};
use std::ffi::{CStr, CString};
use std::ptr;

fn get_levers_api() -> Option<*const librime_sys2::RimeLeversApi> {
    librime_sys2::rime_get_levers_api()
}

pub struct CustomSettings {
    settings: *mut librime_sys2::RimeCustomSettings,
    api: *const librime_sys2::RimeLeversApi,
}

impl CustomSettings {
    pub fn new(config_id: &str, generator_id: &str) -> Result<Self> {
        let api = get_levers_api().ok_or(Error::FunctionNotAvailable("rime_get_levers_api"))?;
        
        let config_id_c = CString::new(config_id)?;
        let generator_id_c = CString::new(generator_id)?;
        
        unsafe {
            let init_func = (*api).custom_settings_init.ok_or(Error::FunctionNotAvailable("custom_settings_init"))?;
            let settings = init_func(config_id_c.as_ptr(), generator_id_c.as_ptr());
            if settings.is_null() {
                return Err(Error::FunctionNotAvailable("custom_settings_init returned null"));
            }
            
            let load_func = (*api).load_settings.ok_or(Error::FunctionNotAvailable("load_settings"))?;
            load_func(settings);
            
            Ok(Self { settings, api })
        }
    }
    
    pub fn get_string(&self, key: &str) -> Option<String> {
        unsafe {
            let key_c = CString::new(key).ok()?;
            let rime_api = get_api();
            if rime_api.is_null() {
                return None;
            }
            
            let get_config = (*self.api).settings_get_config?;
            let mut config = librime_sys2::RimeConfig { ptr: ptr::null_mut() };
            if get_config(self.settings, &mut config) == 0 {
                return None;
            }
            
            let get_cstring = (*rime_api).config_get_cstring?;
            let value_ptr = get_cstring(&mut config, key_c.as_ptr());
            if value_ptr.is_null() {
                return None;
            }
            
            CStr::from_ptr(value_ptr).to_str().ok().map(|s| s.to_owned())
        }
    }
    
    pub fn get_int(&self, key: &str) -> Option<i32> {
        unsafe {
            let key_c = CString::new(key).ok()?;
            let rime_api = get_api();
            if rime_api.is_null() {
                return None;
            }
            
            let get_config = (*self.api).settings_get_config?;
            let mut config = librime_sys2::RimeConfig { ptr: ptr::null_mut() };
            if get_config(self.settings, &mut config) == 0 {
                return None;
            }
            
            let get_int = (*rime_api).config_get_int?;
            let mut value: std::os::raw::c_int = 0;
            if get_int(&mut config, key_c.as_ptr(), &mut value) == 0 {
                return None;
            }
            
            Some(value)
        }
    }
    
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        unsafe {
            let key_c = CString::new(key).ok()?;
            let rime_api = get_api();
            if rime_api.is_null() {
                return None;
            }
            
            let get_config = (*self.api).settings_get_config?;
            let mut config = librime_sys2::RimeConfig { ptr: ptr::null_mut() };
            if get_config(self.settings, &mut config) == 0 {
                return None;
            }
            
            let get_bool = (*rime_api).config_get_bool?;
            let mut value: librime_sys2::Bool = 0;
            if get_bool(&mut config, key_c.as_ptr(), &mut value) == 0 {
                return None;
            }
            
            Some(value != 0)
        }
    }
    
    pub fn get_double(&self, key: &str) -> Option<f64> {
        unsafe {
            let key_c = CString::new(key).ok()?;
            let rime_api = get_api();
            if rime_api.is_null() {
                return None;
            }
            
            let get_config = (*self.api).settings_get_config?;
            let mut config = librime_sys2::RimeConfig { ptr: ptr::null_mut() };
            if get_config(self.settings, &mut config) == 0 {
                return None;
            }
            
            let get_double = (*rime_api).config_get_double?;
            let mut value: f64 = 0.0;
            if get_double(&mut config, key_c.as_ptr(), &mut value) == 0 {
                return None;
            }
            
            Some(value)
        }
    }
    
    pub fn set_string(&self, key: &str, value: &str) -> Result<()> {
        unsafe {
            let customize_string = (*self.api).customize_string.ok_or(Error::FunctionNotAvailable("customize_string"))?;
            let key_c = CString::new(key)?;
            let value_c = CString::new(value)?;
            
            if customize_string(self.settings, key_c.as_ptr(), value_c.as_ptr()) == 0 {
                return Err(Error::FunctionNotAvailable("customize_string failed"));
            }
            
            Ok(())
        }
    }
    
    pub fn set_int(&self, key: &str, value: i32) -> Result<()> {
        unsafe {
            let customize_int = (*self.api).customize_int.ok_or(Error::FunctionNotAvailable("customize_int"))?;
            let key_c = CString::new(key)?;
            
            if customize_int(self.settings, key_c.as_ptr(), value) == 0 {
                return Err(Error::FunctionNotAvailable("customize_int failed"));
            }
            
            Ok(())
        }
    }
    
    pub fn set_bool(&self, key: &str, value: bool) -> Result<()> {
        unsafe {
            let customize_bool = (*self.api).customize_bool.ok_or(Error::FunctionNotAvailable("customize_bool"))?;
            let key_c = CString::new(key)?;
            let bool_value: librime_sys2::Bool = if value { 1 } else { 0 };
            
            if customize_bool(self.settings, key_c.as_ptr(), bool_value) == 0 {
                return Err(Error::FunctionNotAvailable("customize_bool failed"));
            }
            
            Ok(())
        }
    }
    
    pub fn set_double(&self, key: &str, value: f64) -> Result<()> {
        unsafe {
            let customize_double = (*self.api).customize_double.ok_or(Error::FunctionNotAvailable("customize_double"))?;
            let key_c = CString::new(key)?;
            
            if customize_double(self.settings, key_c.as_ptr(), value) == 0 {
                return Err(Error::FunctionNotAvailable("customize_double failed"));
            }
            
            Ok(())
        }
    }
    
    pub fn save(&self) -> Result<()> {
        unsafe {
            let save_settings = (*self.api).save_settings.ok_or(Error::FunctionNotAvailable("save_settings"))?;
            
            if save_settings(self.settings) == 0 {
                return Err(Error::FunctionNotAvailable("save_settings failed"));
            }
            
            Ok(())
        }
    }
}

impl Drop for CustomSettings {
    fn drop(&mut self) {
        unsafe {
            if let Some(destroy) = (*self.api).custom_settings_destroy {
                destroy(self.settings);
            }
        }
    }
}

pub struct SwitcherSettings {
    settings: *mut librime_sys2::RimeSwitcherSettings,
    api: *const librime_sys2::RimeLeversApi,
}

#[derive(Debug, Clone)]
pub struct SchemaInfo {
    pub schema_id: String,
    pub name: String,
}

impl SwitcherSettings {
    pub fn new() -> Result<Self> {
        let api = get_levers_api().ok_or(Error::FunctionNotAvailable("rime_get_levers_api"))?;
        
        unsafe {
            let init_func = (*api).switcher_settings_init.ok_or(Error::FunctionNotAvailable("switcher_settings_init"))?;
            let settings = init_func();
            if settings.is_null() {
                return Err(Error::FunctionNotAvailable("switcher_settings_init returned null"));
            }
            
            let load_func = (*api).load_settings.ok_or(Error::FunctionNotAvailable("load_settings"))?;
            load_func(settings as *mut librime_sys2::RimeCustomSettings);
            
            Ok(Self { settings, api })
        }
    }
    
    pub fn get_schema_list(&self) -> Vec<SchemaInfo> {
        unsafe {
            let get_list = (*self.api).get_available_schema_list;
            if get_list.is_none() {
                return Vec::new();
            }
            
            let mut list = librime_sys2::RimeSchemaList { size: 0, list: ptr::null_mut() };
            
            if get_list.unwrap()(self.settings, &mut list) == 0 {
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
    
    pub fn set_schema_list(&self, schema_ids: &[&str]) -> Result<()> {
        unsafe {
            let select_func = (*self.api).select_schemas.ok_or(Error::FunctionNotAvailable("select_schemas"))?;
            
            let c_strings: Vec<CString> = schema_ids
                .iter()
                .filter_map(|id| CString::new(*id).ok())
                .collect();
            
            let ptrs: Vec<*const i8> = c_strings.iter().map(|s| s.as_ptr()).collect();
            
            if select_func(self.settings, ptrs.as_ptr(), ptrs.len() as i32) == 0 {
                return Err(Error::FunctionNotAvailable("select_schemas failed"));
            }
            
            Ok(())
        }
    }
    
    pub fn get_selected_schema(&self) -> Option<String> {
        unsafe {
            let get_selected = (*self.api).get_selected_schema_list?;
            let mut list = librime_sys2::RimeSchemaList { size: 0, list: ptr::null_mut() };
            if get_selected(self.settings, &mut list) == 0 || list.size == 0 || list.list.is_null() {
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
    
    pub fn save(&self) -> Result<()> {
        unsafe {
            let save_func = (*self.api).save_settings.ok_or(Error::FunctionNotAvailable("save_settings"))?;
            if save_func(self.settings as *mut librime_sys2::RimeCustomSettings) == 0 {
                return Err(Error::FunctionNotAvailable("save_settings failed"));
            }
            Ok(())
        }
    }
}

impl Drop for SwitcherSettings {
    fn drop(&mut self) {
        unsafe {
            if let Some(destroy) = (*self.api).custom_settings_destroy {
                destroy(self.settings as *mut librime_sys2::RimeCustomSettings);
            }
        }
    }
}

pub fn deploy_all() -> Result<()> {
    unsafe {
        let api = get_api();
        if api.is_null() {
            return Err(Error::ApiNotInitialized);
        }
        
        if let Some(deploy) = (*api).deploy {
            if deploy() == 0 {
                return Err(Error::StartMaintenance);
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