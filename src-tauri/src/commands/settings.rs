use crate::db;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub language: String,
    pub ytdlp_path: String,
    pub download_dir: String,
    pub whisper_path: String,
    pub ffmpeg_path: String,
}

fn get_map(conn: &rusqlite::Connection) -> Result<HashMap<String, String>, String> {
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| e.to_string())?;
    let iter = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(|e| e.to_string())?;
    let mut map = HashMap::new();
    for row in iter {
        let (k, v) = row.map_err(|e| e.to_string())?;
        map.insert(k, v);
    }
    Ok(map)
}

#[tauri::command]
pub fn get_settings() -> Result<AppSettings, String> {
    db::with_db(|conn| {
        let m = get_map(conn)?;
        Ok(AppSettings {
            language: m.get("language").cloned().unwrap_or_else(|| "zh".into()),
            ytdlp_path: m.get("ytdlp_path").cloned().unwrap_or_else(|| "yt-dlp".into()),
            download_dir: m.get("download_dir").cloned().unwrap_or_default(),
            whisper_path: m.get("whisper_path").cloned().unwrap_or_else(|| "whisper".into()),
            ffmpeg_path: m
                .get("ffmpeg_path")
                .cloned()
                .unwrap_or_else(|| "/usr/bin/ffmpeg".into()),
        })
    })
}

#[tauri::command]
pub fn save_settings(settings: AppSettings) -> Result<AppSettings, String> {
    db::with_db(|conn| {
        let pairs = [
            ("language", settings.language.as_str()),
            ("ytdlp_path", settings.ytdlp_path.as_str()),
            ("download_dir", settings.download_dir.as_str()),
            ("whisper_path", settings.whisper_path.as_str()),
            ("ffmpeg_path", settings.ffmpeg_path.as_str()),
        ];
        for (k, v) in pairs {
            conn.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![k, v],
            )
            .map_err(|e| e.to_string())?;
        }
        Ok(())
    })?;
    get_settings()
}

#[tauri::command]
pub fn get_setting(key: String) -> Result<String, String> {
    db::with_db(|conn| {
        conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn app_data_path() -> Result<String, String> {
    Ok(db::app_data_dir().to_string_lossy().to_string())
}
