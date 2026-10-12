use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn default_view_mode() -> String {
    "list".to_string()
}

fn deserialize_shortcuts<'de, D>(deserializer: D) -> Result<HashMap<String, Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let stored = serde_json::Value::deserialize(deserializer)?;
    let Some(entries) = stored.as_object() else {
        return Err(serde::de::Error::custom("shortcuts must be an object"));
    };
    let mut shortcuts = HashMap::new();
    for (action, value) in entries {
        let bindings = if let Some(binding) = value.as_str() {
            vec![binding.to_string(), String::new()]
        } else if let Some(bindings) = value.as_array() {
            (0..2)
                .map(|index| bindings.get(index).and_then(|v| v.as_str()).unwrap_or("").to_string())
                .collect()
        } else {
            vec![String::new(), String::new()]
        };
        shortcuts.insert(action.clone(), bindings);
    }
    Ok(shortcuts)
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct AppSettings {
    pub start_path: Option<String>,
    pub hide_dotfiles: bool,
    pub favorites: Vec<String>,
    #[serde(default = "default_view_mode")]
    pub view_mode: String,
    pub theme: String,
    pub sidebar_collapsed: bool,
    pub sidebar_width: u32,
    pub preview_width: u32,
    pub split_ratio: u32,
    pub split_mode: bool,
    pub split_path: Option<String>,
    #[serde(alias = "mouse_middle_new_tab")]
    pub mouse_middle_other_preview: bool,
    pub mouse_side_navigation: bool,
    #[serde(default, deserialize_with = "deserialize_shortcuts")]
    pub shortcuts: HashMap<String, Vec<String>>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            start_path: None,
            hide_dotfiles: false,
            favorites: Vec::new(),
            view_mode: default_view_mode(),
            theme: "system".to_string(),
            sidebar_collapsed: false,
            sidebar_width: 190,
            preview_width: 400,
            split_ratio: 50,
            split_mode: false,
            split_path: None,
            mouse_middle_other_preview: true,
            mouse_side_navigation: true,
            shortcuts: HashMap::new(),
        }
    }
}

fn settings_path_for(app_folder: &str) -> Result<PathBuf, String> {
    let base_dir = std::env::var("APPDATA")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map_err(|_| "Could not resolve a settings directory".to_string())?;

    Ok(base_dir.join(app_folder).join("settings.json"))
}

#[tauri::command]
pub fn load_settings() -> Result<AppSettings, String> {
    let current_path = settings_path_for("file-explorer-preview")?;
    let old_path = settings_path_for("file-explorerpreview")?;
    let path = if current_path.exists() {
        current_path
    } else if old_path.exists() {
        old_path
    } else {
        return Ok(AppSettings::default());
    };

    let contents = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&contents).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_settings(settings: AppSettings) -> Result<(), String> {
    let path = settings_path_for("file-explorer-preview")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let contents = serde_json::to_string_pretty(&settings).map_err(|error| error.to_string())?;
    fs::write(path, contents).map_err(|error| error.to_string())
}
