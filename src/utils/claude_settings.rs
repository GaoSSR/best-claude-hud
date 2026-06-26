use serde_json::{json, Map, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn install_statusline() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let settings_path = settings_path()?;
    let command = resolve_command_path();
    let mut settings = read_settings(&settings_path)?;

    let status_line = json!({
        "type": "command",
        "command": command,
        "padding": 0
    });

    let changed = settings.get("statusLine") != Some(&status_line);
    settings["statusLine"] = status_line;

    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent)?;
    }

    if changed && settings_path.exists() {
        backup_settings(&settings_path)?;
    }

    write_settings(&settings_path, &settings)?;
    Ok(settings_path)
}

fn settings_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Ok(config_dir) = env::var("CLAUDE_CONFIG_DIR") {
        return Ok(PathBuf::from(config_dir).join("settings.json"));
    }

    let home = dirs::home_dir().ok_or("could not determine home directory")?;
    Ok(home.join(".claude").join("settings.json"))
}

fn read_settings(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(Value::Object(Map::new()));
    }

    let content = fs::read_to_string(path)?;
    if content.trim().is_empty() {
        return Ok(Value::Object(Map::new()));
    }

    let value: Value = serde_json::from_str(&content)?;
    if value.is_object() {
        Ok(value)
    } else {
        Err("Claude Code settings.json must contain a JSON object".into())
    }
}

fn backup_settings(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S");
    let backup_path =
        path.with_file_name(format!("settings.json.best-claude-hud-{}.bak", timestamp));
    fs::copy(path, backup_path)?;
    Ok(())
}

fn write_settings(path: &Path, settings: &Value) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(
        path,
        format!("{}\n", serde_json::to_string_pretty(settings)?),
    )?;
    set_private_permissions(path)?;
    Ok(())
}

#[cfg(unix)]
fn set_private_permissions(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn set_private_permissions(_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

fn resolve_command_path() -> String {
    find_in_path("best-claude-hud")
        .or_else(|| env::current_exe().ok())
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|| "best-claude-hud".to_string())
}

fn find_in_path(command: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    env::split_paths(&path_var)
        .map(|dir| dir.join(command))
        .find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_missing_settings_returns_object() {
        let path = env::temp_dir().join("best-claude-hud-missing-settings.json");
        let _ = fs::remove_file(&path);
        let settings = read_settings(&path).expect("missing settings should be allowed");
        assert!(settings.is_object());
    }

    #[test]
    fn reject_non_object_settings() {
        let path = env::temp_dir().join("best-claude-hud-array-settings.json");
        fs::write(&path, "[]").expect("write test settings");
        let result = read_settings(&path);
        let _ = fs::remove_file(&path);
        assert!(result.is_err());
    }
}
