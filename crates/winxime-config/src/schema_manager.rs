use crate::rime_deploy::{get_data_dirs, init_rime_deployer, SchemaInfo};
use std::collections::HashSet;

pub struct SchemaManager {
    user_dir: std::path::PathBuf,
}

impl SchemaManager {
    pub fn new() -> Result<Self, String> {
        init_rime_deployer()?;
        let (_, user_dir) = get_data_dirs();
        Ok(Self { user_dir })
    }

    pub fn get_schema_list(&self) -> Vec<SchemaInfo> {
        let (shared_data_dir, user_data_dir) = get_data_dirs();
        let mut schemas = Vec::new();
        let mut seen_ids = HashSet::new();

        if let Ok(entries) = std::fs::read_dir(&user_data_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".schema.yaml") {
                        let schema_id = name.replace(".schema.yaml", "");

                        if let Ok(content) = std::fs::read_to_string(&path) {
                            let schema_name = extract_schema_name(&content, &schema_id);
                            schemas.push(SchemaInfo {
                                schema_id: schema_id.clone(),
                                name: schema_name,
                            });
                            seen_ids.insert(schema_id);
                        }
                    }
                }
            }
        }

        if let Ok(entries) = std::fs::read_dir(&shared_data_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".schema.yaml") {
                        let schema_id = name.replace(".schema.yaml", "");

                        if seen_ids.contains(&schema_id) {
                            continue;
                        }

                        if let Ok(content) = std::fs::read_to_string(&path) {
                            let schema_name = extract_schema_name(&content, &schema_id);
                            schemas.push(SchemaInfo {
                                schema_id,
                                name: schema_name,
                            });
                        }
                    }
                }
            }
        }

        schemas.sort_by(|a, b| a.name.cmp(&b.name));
        schemas
    }

    pub fn set_schema_list(&self, schema_ids: &[&str]) -> Result<(), String> {
        let default_custom = self.user_dir.join("default.custom.yaml");

        let schema_list_yaml = schema_ids
            .iter()
            .map(|id| format!("    - schema: {}", id))
            .collect::<Vec<_>>()
            .join("\n");

        let content = format!(
            r#"customization:
  distribution_code_name: Xime
  distribution_version: 1.0

patch:
  schema_list:
{}
"#,
            schema_list_yaml
        );

        std::fs::write(&default_custom, content)
            .map_err(|e| format!("Failed to write default.custom.yaml: {}", e))?;

        Ok(())
    }

    pub fn save(&self) -> Result<(), String> {
        Ok(())
    }

    pub fn get_selected_schema(&self) -> Option<String> {
        let default_custom = self.user_dir.join("default.custom.yaml");
        if !default_custom.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&default_custom).ok()?;
        extract_selected_schema(&content)
    }
}

fn extract_selected_schema(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.contains("schema:") {
            let schema = line
                .split("schema:")
                .nth(1)
                .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string());
            if let Some(s) = schema {
                if !s.is_empty() {
                    return Some(s);
                }
            }
        }
    }
    None
}

fn extract_schema_name(content: &str, schema_id: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_schema_block = false;
    let mut indent_level = 0;

    for line in &lines {
        let trimmed = line.trim();

        if trimmed == "schema:" {
            in_schema_block = true;
            indent_level = line.len() - line.trim_start().len();
            continue;
        }

        if in_schema_block {
            let current_indent = line.len() - line.trim_start().len();

            if current_indent <= indent_level && !trimmed.is_empty() && !trimmed.starts_with('#') {
                break;
            }

            if trimmed.starts_with("name:") {
                return trimmed
                    .split(':')
                    .nth(1)
                    .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                    .unwrap_or_else(|| schema_id.to_string());
            }
        }
    }

    schema_id.to_string()
}
