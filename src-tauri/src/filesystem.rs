use serde::Serialize;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

#[derive(Serialize)]
pub struct FileEntry {
    name: String,
    path: String,
    is_dir: bool,
    file_type: String,
    size: u64,
    modified: Option<u64>,
}

#[tauri::command]
pub fn list_directory(path: String) -> Result<Vec<FileEntry>, String> {
    let mut entries = Vec::new();
    let read_dir = fs::read_dir(&path).map_err(|error| error.to_string())?;

    for entry in read_dir {
        let entry = entry.map_err(|error| error.to_string())?;
        let metadata = entry.metadata().map_err(|error| error.to_string())?;
        let entry_path = entry.path();
        let is_dir = metadata.is_dir();

        entries.push(FileEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            path: entry_path.to_string_lossy().to_string(),
            is_dir,
            file_type: describe_file_type(&entry_path, is_dir),
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

#[derive(Serialize)]
pub struct PreviewInfo {
    preview_type: String,
}

#[tauri::command]
pub fn get_preview_type(path: String) -> PreviewInfo {
    let file_path = Path::new(&path);
    let file_name = file_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_lowercase();
    let extension = if matches!(file_name.as_str(), "dockerfile" | "makefile" | "readme" | ".gitignore" | ".env")
        || file_name.starts_with(".env.")
    {
        "txt".to_string()
    } else {
        file_path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("")
        .to_lowercase()
    };

    let preview_type = match extension.as_str() {
        "txt" | "text" | "log" | "csv" | "tsv" | "rst" | "nfo" | "org" | "adoc" | "rtf" | "note" | "ini" | "conf" | "cfg" | "json"
        | "jsonl" | "xml" | "yaml" | "yml" | "toml" | "lock" | "env" | "gitignore"
        | "dockerfile" | "js" | "mjs" | "cjs" | "ts" | "jsx" | "tsx" | "rs" | "py"
        | "java" | "kt" | "swift" | "c" | "cpp" | "cc" | "h" | "hpp" | "cs" | "go"
        | "php" | "rb" | "sh" | "bat" | "ps1" | "html" | "htm" | "css" | "scss"
        | "sass" | "less" | "md" | "markdown" | "svelte" | "sql" | "r" | "lua" | "pl"
        | "dart" | "vue" => "text",
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "ico" | "avif"
        | "apng" | "tif" | "tiff" => "image",
        "pdf" => "pdf",
        "mp3" | "wav" | "ogg" | "oga" | "opus" | "flac" | "m4a" | "aac" | "weba" => {
            "audio"
        }
        "mp4" | "webm" | "mkv" | "mov" | "avi" | "m4v" | "ogv" | "wmv" => "video",
        _ => "unsupported",
    };

    PreviewInfo {
        preview_type: preview_type.to_string(),
    }
}

#[tauri::command]
pub fn read_text_file(path: String) -> Result<String, String> {
    const PREVIEW_LIMIT: usize = 1_000_000;
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|error| error.to_string())?
        .take((PREVIEW_LIMIT + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;

    let truncated = bytes.len() > PREVIEW_LIMIT;
    if truncated {
        bytes.truncate(PREVIEW_LIMIT);
    }

    let mut contents = String::from_utf8_lossy(&bytes).into_owned();
    if truncated {
        contents.push_str("\n\n[Preview limited to the first 1 MB]");
    }
    Ok(contents)
}

#[tauri::command]
pub fn file_exists(path: String) -> bool {
    Path::new(&path).is_file()
}

#[tauri::command]
pub fn get_home_dir() -> Result<String, String> {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|_| "Could not resolve the home directory".to_string())
}

#[tauri::command]
pub fn get_parent_directory(path: String) -> Option<String> {
    PathBuf::from(path)
        .parent()
        .map(|parent| parent.to_string_lossy().to_string())
}

fn child_path(directory: &Path, name: &str) -> Result<PathBuf, String> {
    let candidate = Path::new(name);
    if name.trim().is_empty()
        || candidate.components().count() != 1
        || candidate.file_name().and_then(|value| value.to_str()) != Some(name)
    {
        return Err("Enter a single file or folder name".to_string());
    }
    Ok(directory.join(candidate))
}

#[tauri::command]
pub fn create_directory(path: String) -> Result<(), String> {
    let path = PathBuf::from(path);
    let parent = path
        .parent()
        .ok_or_else(|| "Could not resolve the parent folder".to_string())?;
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Enter a valid folder name".to_string())?;
    fs::create_dir(child_path(parent, name)?).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn rename_path(path: String, new_name: String) -> Result<(), String> {
    let old_path = PathBuf::from(path);
    let parent = old_path
        .parent()
        .ok_or_else(|| "This item cannot be renamed here".to_string())?;
    let new_path = child_path(parent, &new_name)?;
    if new_path == old_path {
        return Ok(());
    }
    fs::rename(old_path, new_path).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_path(path: String) -> Result<(), String> {
    let path = PathBuf::from(path);
    let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_dir() {
        fs::remove_dir_all(path).map_err(|error| error.to_string())
    } else {
        fs::remove_file(path).map_err(|error| error.to_string())
    }
}

