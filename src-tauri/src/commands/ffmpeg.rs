use crate::commands::settings;
use crate::db;
use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;

#[derive(Serialize)]
pub struct FfmpegCheck {
    pub available: bool,
    pub path: String,
    pub message: String,
}

fn ffmpeg_bin() -> String {
    settings::get_settings()
        .map(|s| s.ffmpeg_path)
        .unwrap_or_else(|_| "/usr/bin/ffmpeg".into())
}

#[tauri::command]
pub fn check_ffmpeg() -> Result<FfmpegCheck, String> {
    let path = ffmpeg_bin();
    let output = Command::new(&path).arg("-version").output();
    match output {
        Ok(o) if o.status.success() => Ok(FfmpegCheck {
            available: true,
            path,
            message: "FFmpeg 可用".into(),
        }),
        _ => Ok(FfmpegCheck {
            available: false,
            path: path.clone(),
            message: format!("未找到 FFmpeg（{path}）。封面生成将使用占位图。"),
        }),
    }
}

/// 从视频抽一帧作为封面；失败返回 None（前端用占位图）。
pub fn generate_cover_for_video(video_id: i64, video_path: &str) -> Result<Option<String>, String> {
    let check = check_ffmpeg()?;
    if !check.available {
        return Ok(None);
    }
    let out: PathBuf = db::covers_dir().join(format!("{video_id}.jpg"));
    let status = Command::new(&check.path)
        .args([
            "-y",
            "-ss",
            "5",
            "-i",
            video_path,
            "-frames:v",
            "1",
            "-q:v",
            "3",
            out.to_str().unwrap_or(""),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| e.to_string())?;

    if status.success() && out.exists() {
        Ok(Some(out.to_string_lossy().to_string()))
    } else {
        // retry at 1s
        let status = Command::new(&check.path)
            .args([
                "-y",
                "-ss",
                "1",
                "-i",
                video_path,
                "-frames:v",
                "1",
                "-q:v",
                "3",
                out.to_str().unwrap_or(""),
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() && out.exists() {
            Ok(Some(out.to_string_lossy().to_string()))
        } else {
            Ok(None)
        }
    }
}

#[tauri::command]
pub fn regenerate_cover(video_id: i64) -> Result<Option<String>, String> {
    let path: String = db::with_db(|conn| {
        conn.query_row(
            "SELECT path FROM videos WHERE id = ?1",
            rusqlite::params![video_id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())
    })?;
    let cover = generate_cover_for_video(video_id, &path)?;
    if let Some(ref c) = cover {
        db::with_db(|conn| {
            conn.execute(
                "UPDATE videos SET cover_path = ?1, updated_at = datetime('now') WHERE id = ?2",
                rusqlite::params![c, video_id],
            )
            .map_err(|e| e.to_string())?;
            Ok(())
        })?;
    }
    Ok(cover)
}
