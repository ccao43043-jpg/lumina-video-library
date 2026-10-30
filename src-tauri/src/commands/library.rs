use crate::db::{self, LibraryPath, Video, VIDEO_SELECT};
use rusqlite::params;
use serde::Serialize;

#[tauri::command]
pub fn list_videos(query: Option<String>, favorites_only: Option<bool>) -> Result<Vec<Video>, String> {
    db::with_db(|conn| {
        let q = query.unwrap_or_default().trim().to_lowercase();
        let fav = favorites_only.unwrap_or(false);
        let mut sql = format!("{VIDEO_SELECT} WHERE 1=1");
        if fav {
            sql.push_str(" AND favorite = 1");
        }
        sql.push_str(" ORDER BY updated_at DESC");

        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| db::video_from_row(conn, row))
            .map_err(|e| e.to_string())?;

        let mut videos = Vec::new();
        for r in rows {
            let v = r.map_err(|e| e.to_string())?;
            if q.is_empty() {
                videos.push(v);
                continue;
            }
            let in_title = v.title.to_lowercase().contains(&q);
            let in_path = v.path.to_lowercase().contains(&q);
            let in_tags = v.tags.iter().any(|t| t.to_lowercase().contains(&q));
            if in_title || in_path || in_tags {
                videos.push(v);
            }
        }
        Ok(videos)
    })
}

#[tauri::command]
pub fn get_video(id: i64) -> Result<Option<Video>, String> {
    db::with_db(|conn| {
        let mut stmt = conn
            .prepare(&format!("{VIDEO_SELECT} WHERE id = ?1"))
            .map_err(|e| e.to_string())?;
        let mut rows = stmt
            .query_map(params![id], |row| db::video_from_row(conn, row))
            .map_err(|e| e.to_string())?;
        match rows.next() {
            Some(Ok(v)) => Ok(Some(v)),
            Some(Err(e)) => Err(e.to_string()),
            None => Ok(None),
        }
    })
}

#[tauri::command]
pub fn toggle_favorite(id: i64) -> Result<Video, String> {
    db::with_db(|conn| {
        conn.execute(
            "UPDATE videos SET favorite = CASE WHEN favorite = 0 THEN 1 ELSE 0 END,
             updated_at = datetime('now') WHERE id = ?1",
            params![id],
        )
        .map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(&format!("{VIDEO_SELECT} WHERE id = ?1"))
            .map_err(|e| e.to_string())?;
        let v = stmt
            .query_row(params![id], |row| db::video_from_row(conn, row))
            .map_err(|e| e.to_string())?;
        Ok(v)
    })
}

#[tauri::command]
pub fn set_video_tags(id: i64, tags: Vec<String>) -> Result<Video, String> {
    db::with_db(|conn| {
        conn.execute("DELETE FROM video_tags WHERE video_id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        for name in tags {
            let name = name.trim();
            if name.is_empty() {
                continue;
            }
            conn.execute(
                "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
                params![name],
            )
            .map_err(|e| e.to_string())?;
            let tag_id: i64 = conn
                .query_row("SELECT id FROM tags WHERE name = ?1", params![name], |r| {
                    r.get(0)
                })
                .map_err(|e| e.to_string())?;
            conn.execute(
                "INSERT OR IGNORE INTO video_tags (video_id, tag_id) VALUES (?1, ?2)",
                params![id, tag_id],
            )
            .map_err(|e| e.to_string())?;
        }
        conn.execute(
            "UPDATE videos SET updated_at = datetime('now') WHERE id = ?1",
            params![id],
        )
        .map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(&format!("{VIDEO_SELECT} WHERE id = ?1"))
            .map_err(|e| e.to_string())?;
        let v = stmt
            .query_row(params![id], |row| db::video_from_row(conn, row))
            .map_err(|e| e.to_string())?;
        Ok(v)
    })
}

#[tauri::command]
pub fn list_all_tags() -> Result<Vec<String>, String> {
    db::with_db(|conn| {
        let mut stmt = conn
            .prepare("SELECT name FROM tags ORDER BY name")
            .map_err(|e| e.to_string())?;
        let iter = stmt
            .query_map([], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for t in iter {
            out.push(t.map_err(|e| e.to_string())?);
        }
        Ok(out)
    })
}

#[tauri::command]
pub fn list_library_paths() -> Result<Vec<LibraryPath>, String> {
    db::with_db(|conn| {
        let mut stmt = conn
            .prepare("SELECT id, path, added_at FROM library_paths ORDER BY id")
            .map_err(|e| e.to_string())?;
        let iter = stmt
            .query_map([], |r| {
                Ok(LibraryPath {
                    id: r.get(0)?,
                    path: r.get(1)?,
                    added_at: r.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for p in iter {
            out.push(p.map_err(|e| e.to_string())?);
        }
        Ok(out)
    })
}

#[tauri::command]
pub fn add_library_path(path: String) -> Result<LibraryPath, String> {
    db::with_db(|conn| {
        conn.execute(
            "INSERT OR IGNORE INTO library_paths (path, added_at) VALUES (?1, datetime('now'))",
            params![path],
        )
        .map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, path, added_at FROM library_paths WHERE path = ?1")
            .map_err(|e| e.to_string())?;
        let p = stmt
            .query_row(params![path], |r| {
                Ok(LibraryPath {
                    id: r.get(0)?,
                    path: r.get(1)?,
                    added_at: r.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?;
        Ok(p)
    })
}

#[tauri::command]
pub fn remove_library_path(id: i64) -> Result<(), String> {
    db::with_db(|conn| {
        conn.execute("DELETE FROM library_paths WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    })
}

#[derive(Serialize)]
pub struct PlayResult {
    pub opened: bool,
    pub path: String,
    pub message: String,
}

#[tauri::command]
pub fn open_in_player(path: String) -> Result<PlayResult, String> {
    if !std::path::Path::new(&path).exists() {
        return Err(format!("文件不存在: {path}"));
    }
    // Prefer opener plugin from frontend; this is a Rust fallback via OS command
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(&path).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", &path])
            .spawn();
    }
    Ok(PlayResult {
        opened: true,
        path,
        message: "已请求系统默认播放器打开".into(),
    })
}

#[tauri::command]
pub fn delete_video(id: i64) -> Result<(), String> {
    db::with_db(|conn| {
        conn.execute("DELETE FROM videos WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    })
}
