mod commands;
mod db;

use commands::{ffmpeg, library, scan, settings, whisper, ytdlp};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(e) = db::init_db() {
        eprintln!("数据库初始化失败: {e}");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            library::list_videos,
            library::get_video,
            library::toggle_favorite,
            library::set_video_tags,
            library::list_all_tags,
            library::list_library_paths,
            library::add_library_path,
            library::remove_library_path,
            library::open_in_player,
            library::delete_video,
            scan::scan_library,
            scan::scan_path,
            settings::get_settings,
            settings::save_settings,
            settings::get_setting,
            settings::app_data_path,
            ytdlp::check_ytdlp,
            ytdlp::detect_video_platform,
            ytdlp::download_with_ytdlp,
            scan::import_video_file,
            whisper::check_whisper,
            whisper::transcribe_with_whisper,
            ffmpeg::check_ffmpeg,
            ffmpeg::regenerate_cover,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
