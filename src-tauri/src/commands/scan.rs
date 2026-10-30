use crate::commands::ffmpeg;
use crate::db;
use rusqlite::params;
use serde::Serialize;
use std::path::Path;
use walkdir::WalkDir;

const EXT: &[&str] = &[
    "mp4", "mkv", "avi", "mov", "webm", "flv", "wmv", "m4v", "ts", "m2ts", "mpg", "mpeg",
];

#[derive(Serialize)]
pub struct ScanResult {
    pub scanned: usize,
    pub added: usize,
    pub updated: usize,
    pub skipped: usize,
}

fn is_video(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| EXT.iter().any(|x| x.eq_ignore_ascii_case(e)))
        .unwrap_or(false)
}

#[tauri::command]
pub fn scan_library() -> Result<ScanResult, String> {
    let paths: Vec<String> = db::with_db(|conn| {
        let mut stmt = conn
            .prepare("SELECT path FROM library_paths")
            .map_err(|e| e.to_string())?;
        let iter = stmt
            .query_map([], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for p in iter {
            out.push(p.map_err(|e| e.to_string())?);
        }
        Ok(out)
    })?;

    let mut scanned = 0usize;
    let mut added = 0usize;
    let mut updated = 0usize;
    let skipped = 0usize;

    for root in paths {
        let root_path = Path::new(&root);
        if !root_path.exists() {
            continue;
        }
        for entry in WalkDir::new(root_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() || !is_video(path) {
                continue;
            }
            scanned += 1;
            let path_str = path.to_string_lossy().to_string();
            let title = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("未命名")
                .to_string();
            let size = std::fs::metadata(path).map(|m| m.len() as i64).unwrap_or(0);

            let exists: Option<i64> = db::with_db(|conn| {
                Ok(conn
                    .query_row(
                        "SELECT id FROM videos WHERE path = ?1",
                        params![&path_str],
                        |r| r.get(0),
                    )
                    .ok())
            })?;

            if let Some(id) = exists {
                db::with_db(|conn| {
                    conn.execute(
                        "UPDATE videos SET title = ?1, size_bytes = ?2, updated_at = datetime('now') WHERE id = ?3",
                        params![title, size, id],
                    )
                    .map_err(|e| e.to_string())?;
                    Ok(())
                })?;
                updated += 1;
            } else {
                let id: i64 = db::with_db(|conn| {
                    conn.execute(
                        "INSERT INTO videos (title, path, size_bytes, duration_secs, cover_path, favorite, created_at, updated_at)
                         VALUES (?1, ?2, ?3, 0, NULL, 0, datetime('now'), datetime('now'))",
                        params![title, path_str, size],
                    )
                    .map_err(|e| e.to_string())?;
                    Ok(conn.last_insert_rowid())
                })?;
                if let Ok(Some(cover)) = ffmpeg::generate_cover_for_video(id, &path_str) {
                    let _ = db::with_db(|conn| {
                        conn.execute(
                            "UPDATE videos SET cover_path = ?1 WHERE id = ?2",
                            params![cover, id],
                        )
                        .map_err(|e| e.to_string())?;
                        Ok(())
                    });
                }
                added += 1;
            }
        }
    }

    Ok(ScanResult {
        scanned,
        added,
        updated,
        skipped,
    })
}

#[tauri::command]
pub fn scan_path(path: String) -> Result<ScanResult, String> {
    db::with_db(|conn| {
        conn.execute(
            "INSERT OR IGNORE INTO library_paths (path, added_at) VALUES (?1, datetime('now'))",
            params![path],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    })?;
    scan_library()
}
