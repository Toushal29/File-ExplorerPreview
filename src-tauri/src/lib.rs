// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;


// A structure representing a single file or folder entry.
// Derives `Serialize` so Tauri can convert it to a JSON object for the frontend
#[derive(Serialize)]
struct FileEntry {
    name: String,
    path: String,
    is_dir: bool,
    file_type: String,
    size: u64,
    modified: Option<u64>,
}

#[derive(Deserialize, Serialize)]
struct AppSettings {
    start_path: Option<String>,
    hide_dotfiles: bool,
}

// lists and sorts the contents of a directory
// Returns a list of `FileEntry` items sorted so that directories appear first, followed by files, with both groups sorted alphabetically
#[tauri::command]
fn list_directory(path: String) -> Result<Vec<FileEntry>, String> {
    let mut entries = Vec::new();
    // Attempt to open the directory. The `?` operator immediately returns the error as a String if the path doesn't exist or permissions are denied.
    let read_dir = fs::read_dir(&path).map_err(|e| e.to_string())?;

    // iterate through the items in the directory
    for entry in read_dir {
        let entry = entry.map_err(|e| e.to_string())?;
        let metadata = entry.metadata().map_err(|e| e.to_string())?;

        entries.push(FileEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            path: entry.path().to_string_lossy().to_string(),
            is_dir: metadata.is_dir(),
            file_type: describe_file_type(&entry.path(), metadata.is_dir()),
            size: if metadata.is_file() { metadata.len() } else { 0 },
            modified: metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs()),
        });
    }

    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    // Return the successfully populated and sorted vector
    Ok(entries)
}

fn describe_file_type(path: &Path, is_dir: bool) -> String {
    if is_dir {
        return "Folder".to_string();
    }

    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| format!("{} file", extension.to_uppercase()))
        .unwrap_or_else(|| "File".to_string())
}


// Container for the frontend response.
// Derives `serde::Serialize` so Tauri can convert this into a JSON object automatically
#[derive(serde::Serialize)]
struct PreviewInfo {
    preview_type: String,      // Holds categories like "text", "image", "audio", etc.
}

// analyzes a file's extension to determine its categorical type for allowing proper preview
#[tauri::command]
fn get_preview_type(path: String) -> PreviewInfo {
    // Extract and normalize the file extension
    let ext = Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Map the extension to a frontend-friendly preview category
    let preview_type = match ext.as_str() {
        // Text-based files and code syntax languages
        "txt" | "text" | "log" | "csv" | "tsv" | "ini" | "conf" | "cfg" | "json" | "jsonl"
        | "xml" | "yaml" | "yml" | "toml" | "lock" | "env" | "gitignore" | "dockerfile"
        | "js" | "mjs" | "cjs" | "ts" | "jsx" | "tsx" | "rs" | "py" | "java" | "kt"
        | "swift" | "c" | "cpp" | "cc" | "h" | "hpp" | "cs" | "go" | "php" | "rb" | "sh"
        | "bat" | "ps1" | "html" | "htm" | "css" | "scss" | "sass" | "less" | "md"
        | "markdown" | "svelte" | "sql" | "r" | "lua" | "pl" | "dart" | "vue" => "text",
        // Standard web-supported image formats
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "ico" | "avif" | "apng"
        | "tif" | "tiff" => "image",
        // Portable Document Format (often needs a custom iframe or canvas renderer)
        "pdf" => "pdf",
        // Audio media formats
        "mp3" | "wav" | "ogg" | "oga" | "opus" | "flac" | "m4a" | "aac" | "weba" => "audio",
        // Video media formats
        "mp4" | "webm" | "mkv" | "mov" | "avi" | "m4v" | "ogv" | "wmv" => "video",
        _ => "unsupported",
    };

    // Construct and return the structural payload
    PreviewInfo {
        preview_type: preview_type.to_string(),
    }
}

// reads the contents of a text file at the specified path.
#[tauri::command]
fn read_text_file(path: String) -> Result<String, String> {
    // Attempt to read the file to a String. 
    // If successful, it wraps the String in `Ok`.
    // If it fails, `map_err` converts the standard `std::io::Error` into a `String` so it can be passed back
    fs::read_to_string(path).map_err(|e| e.to_string())
}

// checks if a file exists at the given path
#[tauri::command]
fn file_exists(path: String) -> bool {
    // Create a new Path reference from the input String and check if it points to an existing regular file
    // Returns `true` if it exists and is a file; `false` otherwise
    Path::new(&path).is_file()
}

// attempts to retrieve the current user's home directory path
// checks common environment variables for Windows (`USERPROFILE`) and Unix/macOS (`HOME`)
#[tauri::command]
fn get_home_dir() -> Result<String, String> {
    std::env::var("USERPROFILE")        // try to get the Windows home directory variable
        .or_else(|_| std::env::var("HOME")) // If fails, fallback to the Unix variable
        // If both environment lookups fail, return a user-friendly error message string
        .map_err(|_| "Could not resolve the home directory".to_string())
}

fn settings_path() -> Result<PathBuf, String> {
    let base_dir = std::env::var("APPDATA")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map_err(|_| "Could not resolve a settings directory".to_string())?;

    Ok(base_dir.join("file-explorerpreview").join("settings.json"))
}

#[tauri::command]
fn load_settings() -> Result<AppSettings, String> {
    let path = settings_path()?;

    if !path.exists() {
        return Ok(AppSettings {
            start_path: None,
            hide_dotfiles: false,
        });
    }

    let contents = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&contents).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_settings(settings: AppSettings) -> Result<(), String> {
    let path = settings_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let contents = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(path, contents).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(
            tauri::generate_handler![
                list_directory,
                get_preview_type,
                read_text_file,
                file_exists,
                get_home_dir,
                load_settings,
                save_settings,
            ]
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
