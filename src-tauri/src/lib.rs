mod filesystem;
mod settings;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            filesystem::list_directory,
            filesystem::get_preview_type,
            filesystem::read_text_file,
            filesystem::file_exists,
            filesystem::get_home_dir,
            filesystem::get_parent_directory,
            filesystem::create_directory,
            filesystem::rename_path,
            filesystem::delete_path,
            settings::load_settings,
            settings::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
