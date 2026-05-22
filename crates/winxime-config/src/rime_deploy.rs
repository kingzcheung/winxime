pub use librime::levers::deploy_all;
pub use librime::levers::SchemaInfo;
use librime::{
    create_session, get_api, initialize, join_maintenance_thread, setup, start_maintenance, Traits,
};
use std::ffi::CString;
use std::sync::Once;

static RIME_INIT: Once = Once::new();

pub fn init_rime_deployer() -> Result<(), String> {
    RIME_INIT.call_once(|| {
        let (shared_data_dir, user_data_dir) = get_data_dirs();

        ensure_user_config_files(&user_data_dir);
        ensure_schemas_in_user_dir(&shared_data_dir, &user_data_dir);

        let mut traits = Traits::new();
        traits
            .set_shared_data_dir(shared_data_dir.to_str().unwrap_or(""))
            .set_user_data_dir(user_data_dir.to_str().unwrap_or(""))
            .set_distribution_name("Xime")
            .set_distribution_code_name("Xime")
            .set_distribution_version("1.0")
            .set_app_name("rime.xime.setup")
            .set_min_log_level(2);

        setup(&mut traits);

        if initialize(&mut traits).is_err() {
            return;
        }

        if start_maintenance(true).is_ok() {
            join_maintenance_thread();
        }

        if let Ok(session) = create_session() {
            drop(session);
        }

        unsafe {
            let api = get_api();
            if !api.is_null() {
                if let Some(deploy_config) = (*api).deploy_config_file {
                    let config_file = CString::new("xime.yaml").unwrap_or_default();
                    let version_key = CString::new("config_version").unwrap_or_default();
                    deploy_config(config_file.as_ptr(), version_key.as_ptr());
                }
            }
        }
    });

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
}

fn ensure_schemas_in_user_dir(shared_data_dir: &std::path::Path, user_data_dir: &std::path::Path) {
    let default_custom = user_data_dir.join("default.custom.yaml");
    if !default_custom.exists() {
        let schema_list = if let Ok(entries) = std::fs::read_dir(shared_data_dir) {
            let schemas: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.ends_with(".schema.yaml") {
                        Some(name.replace(".schema.yaml", ""))
                    } else {
                        None
                    }
                })
                .collect();

            if schemas.is_empty() {
                "    - schema: wubi86\n".to_string()
            } else {
                schemas
                    .iter()
                    .map(|s| format!("    - schema: {}\n", s))
                    .collect::<Vec<_>>()
                    .join("")
            }
        } else {
            "    - schema: wubi86\n".to_string()
        };

        let content = format!(
            r#"customization:
  distribution_code_name: Xime
  distribution_version: 1.0

patch:
  schema_list:
{}
"#,
            schema_list
        );

        std::fs::write(&default_custom, content).ok();
    }
}

pub fn get_data_dirs() -> (std::path::PathBuf, std::path::PathBuf) {
    let exe_path = std::env::current_exe()
        .ok()
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let exe_dir = exe_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    (exe_dir.join("data"), exe_dir.join("user-data"))
}

fn get_config_source_dir() -> std::path::PathBuf {
    let exe_path = std::env::current_exe()
        .ok()
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    exe_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("resources")
}

pub fn deploy_all_schemas() -> Result<(), String> {
    init_rime_deployer()?;
    deploy_all().map_err(|e| e.to_string())
}
