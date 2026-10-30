use once_cell::sync::Lazy;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

pub static DB: Lazy<Mutex<Option<Connection>>> = Lazy::new(|| Mutex::new(None));

pub fn app_data_dir() -> PathBuf {
    let dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("lumina-video-library");
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::create_dir_all(dir.join("covers"));
    let _ = std::fs::create_dir_all(dir.join("downloads"));
    dir
}

pub fn db_path() -> PathBuf {
    app_data_dir().join("lumina.db")
}

pub fn covers_dir() -> PathBuf {
    app_data_dir().join("covers")
}

pub fn init_db() -> Result<(), String> {
    let conn = Connection::open(db_path()).map_err(|e| e.to_string())?;
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS library_paths (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            added_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS videos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            path TEXT NOT NULL UNIQUE,
            size_bytes INTEGER DEFAULT 0,
            duration_secs REAL DEFAULT 0,
            cover_path TEXT,
            favorite INTEGER DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        );
        CREATE TABLE IF NOT EXISTS video_tags (
            video_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (video_id, tag_id),
            FOREIGN KEY (video_id) REFERENCES videos(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );
        "#,
    )
    .map_err(|e| e.to_string())?;

    // default settings
    let defaults = [
        ("language", "zh"),
        ("ytdlp_path", "yt-dlp"),
        ("download_dir", ""),
        ("whisper_path", "whisper"),
        ("ffmpeg_path", "/usr/bin/ffmpeg"),
    ];
    for (k, v) in defaults {
        conn.execute(
            "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)",
            params![k, v],
        )
        .map_err(|e| e.to_string())?;
    }

    // set default download dir if empty
    let dl: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'download_dir'",
            [],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if dl.as_deref().unwrap_or("").is_empty() {
        let p = app_data_dir().join("downloads").to_string_lossy().to_string();
        conn.execute(
            "UPDATE settings SET value = ?1 WHERE key = 'download_dir'",
            params![p],
        )
        .map_err(|e| e.to_string())?;
    }

    *DB.lock().map_err(|e| e.to_string())? = Some(conn);
    Ok(())
}

pub fn with_db<F, T>(f: F) -> Result<T, String>
where
    F: FnOnce(&Connection) -> Result<T, String>,
{
    let guard = DB.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or_else(|| "数据库未初始化".to_string())?;
    f(conn)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Video {
    pub id: i64,
    pub title: String,
    pub path: String,
    pub size_bytes: i64,
    pub duration_secs: f64,
    pub cover_path: Option<String>,
    pub favorite: bool,
    pub created_at: String,
    pub updated_at: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryPath {
    pub id: i64,
    pub path: String,
    pub added_at: String,
}

pub fn video_from_row(conn: &Connection, row: &rusqlite::Row<'_>) -> Result<Video, rusqlite::Error> {
    let id: i64 = row.get(0)?;
    let tags = load_tags(conn, id).unwrap_or_default();
    Ok(Video {
        id,
        title: row.get(1)?,
        path: row.get(2)?,
        size_bytes: row.get(3)?,
        duration_secs: row.get(4)?,
        cover_path: row.get(5)?,
        favorite: row.get::<_, i64>(6)? != 0,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
        tags,
    })
}

pub fn load_tags(conn: &Connection, video_id: i64) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT t.name FROM tags t
             JOIN video_tags vt ON vt.tag_id = t.id
             WHERE vt.video_id = ?1
             ORDER BY t.name",
        )
        .map_err(|e| e.to_string())?;
    let iter = stmt
        .query_map(params![video_id], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let mut tags = Vec::new();
    for t in iter {
        tags.push(t.map_err(|e| e.to_string())?);
    }
    Ok(tags)
}

pub const VIDEO_SELECT: &str = "SELECT id, title, path, size_bytes, duration_secs, cover_path, favorite, created_at, updated_at FROM videos";
