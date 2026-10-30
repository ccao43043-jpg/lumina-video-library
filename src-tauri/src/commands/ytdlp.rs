use crate::commands::scan;
use crate::commands::settings;
use regex::Regex;
use serde::Serialize;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Clone)]
pub struct YtdlpProgress {
    pub percent: f64,
    pub status: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct YtdlpCheck {
    pub available: bool,
    pub path: String,
    pub version: Option<String>,
    pub message: String,
}

#[derive(Serialize)]
pub struct DownloadResult {
    pub path: String,
    pub imported: bool,
    pub import_message: String,
}

fn resolve_ytdlp() -> Result<String, String> {
    let s = settings::get_settings()?;
    Ok(s.ytdlp_path)
}

/// Normalize platform + raw input into a URL yt-dlp can fetch.
fn resolve_url(url: &str, platform: Option<&str>) -> String {
    let raw = url.trim();
    let p = platform.unwrap_or("auto").to_ascii_lowercase();
    let detected = detect_platform(raw);
    let plat = if p == "auto" || p.is_empty() {
        detected.as_str()
    } else {
        p.as_str()
    };

    match plat {
        "bilibili" => {
            if raw.to_ascii_lowercase().starts_with("http://")
                || raw.to_ascii_lowercase().starts_with("https://")
            {
                raw.to_string()
            } else if Regex::new(r"(?i)^bv[\w]+$").unwrap().is_match(raw) {
                format!("https://www.bilibili.com/video/{raw}")
            } else if let Some(caps) = Regex::new(r"(?i)^av(\d+)$").unwrap().captures(raw) {
                format!("https://www.bilibili.com/video/av{}", &caps[1])
            } else {
                raw.to_string()
            }
        }
        "youtube" => {
            if raw.to_ascii_lowercase().starts_with("http://")
                || raw.to_ascii_lowercase().starts_with("https://")
            {
                raw.to_string()
            } else if Regex::new(r"^[\w-]{11}$").unwrap().is_match(raw) {
                format!("https://www.youtube.com/watch?v={raw}")
            } else {
                raw.to_string()
            }
        }
        _ => raw.to_string(),
    }
}

fn detect_platform(input: &str) -> String {
    let lower = input.to_ascii_lowercase();
    if lower.contains("bilibili.com")
        || lower.contains("b23.tv")
        || Regex::new(r"(?i)^bv[\w]+$").unwrap().is_match(input.trim())
        || Regex::new(r"(?i)^av\d+$").unwrap().is_match(input.trim())
    {
        return "bilibili".into();
    }
    if lower.contains("douyin.com")
        || lower.contains("iesdouyin.com")
        || lower.contains("v.douyin.com")
    {
        return "douyin".into();
    }
    if lower.contains("youtube.com")
        || lower.contains("youtu.be")
        || lower.contains("youtube-nocookie.com")
        || Regex::new(r"^[\w-]{11}$").unwrap().is_match(input.trim())
    {
        return "youtube".into();
    }
    "custom".into()
}

#[tauri::command]
pub fn check_ytdlp() -> Result<YtdlpCheck, String> {
    let path = resolve_ytdlp().unwrap_or_else(|_| "yt-dlp".into());
    let output = Command::new(&path).arg("--version").output();
    match output {
        Ok(o) if o.status.success() => {
            let ver = String::from_utf8_lossy(&o.stdout).trim().to_string();
            Ok(YtdlpCheck {
                available: true,
                path,
                version: Some(ver),
                message: "yt-dlp 可用".into(),
            })
        }
        Ok(_) | Err(_) => Ok(YtdlpCheck {
            available: false,
            path: path.clone(),
            version: None,
            message: format!(
                "未找到 yt-dlp（尝试路径: {path}）。请安装后在设置中配置路径。\n\
                 macOS/Linux: brew install yt-dlp 或 pip install yt-dlp\n\
                 Windows: winget install yt-dlp"
            ),
        }),
    }
}

#[tauri::command]
pub fn detect_video_platform(input: String) -> Result<String, String> {
    Ok(detect_platform(&input))
}

#[tauri::command]
pub async fn download_with_ytdlp(
    app: AppHandle,
    url: String,
    audio_only: bool,
    platform: Option<String>,
    auto_import: Option<bool>,
) -> Result<DownloadResult, String> {
    let check = check_ytdlp()?;
    if !check.available {
        return Err(check.message);
    }
    let s = settings::get_settings()?;
    let out_dir = if s.download_dir.is_empty() {
        crate::db::app_data_dir().join("downloads")
    } else {
        std::path::PathBuf::from(&s.download_dir)
    };
    let _ = std::fs::create_dir_all(&out_dir);

    let resolved = resolve_url(&url, platform.as_deref());

    let template = out_dir
        .join("%(title)s [%(id)s].%(ext)s")
        .to_string_lossy()
        .to_string();

    let mut args = vec![
        "--newline".to_string(),
        "--progress".to_string(),
        "-o".to_string(),
        template,
        "--no-playlist".to_string(),
    ];
    if audio_only {
        args.extend(["-x".into(), "--audio-format".into(), "mp3".into()]);
    }
    args.push(resolved.clone());

    let _ = app.emit(
        "ytdlp-progress",
        YtdlpProgress {
            percent: 0.0,
            status: "starting".into(),
            message: format!("平台解析: {resolved}"),
        },
    );

    let mut child = Command::new(&check.path)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("启动 yt-dlp 失败: {e}"))?;

    let stdout = child.stdout.take().ok_or("无法读取 stdout")?;
    let stderr = child.stderr.take();
    let re = Regex::new(r"(\d+(?:\.\d+)?)%").unwrap();

    let reader = BufReader::new(stdout);
    let mut last_file = String::new();
    for line in reader.lines().flatten() {
        if let Some(caps) = re.captures(&line) {
            if let Ok(pct) = caps[1].parse::<f64>() {
                let _ = app.emit(
                    "ytdlp-progress",
                    YtdlpProgress {
                        percent: pct,
                        status: "downloading".into(),
                        message: line.clone(),
                    },
                );
            }
        }
        if line.contains("[download] Destination:") {
            if let Some(p) = line.split("Destination:").nth(1) {
                last_file = p.trim().to_string();
            }
        }
        if line.contains("[Merger] Merging formats into") {
            if let Some(p) = line.split('"').nth(1) {
                last_file = p.trim().to_string();
            }
        }
        if line.contains("[ExtractAudio] Destination:") {
            if let Some(p) = line.split("Destination:").nth(1) {
                last_file = p.trim().to_string();
            }
        }
    }

    let status = child.wait().map_err(|e| e.to_string())?;
    if !status.success() {
        let err = stderr
            .map(|s| {
                let mut buf = String::new();
                for line in BufReader::new(s).lines().flatten() {
                    buf.push_str(&line);
                    buf.push('\n');
                }
                buf
            })
            .unwrap_or_default();
        let _ = app.emit(
            "ytdlp-progress",
            YtdlpProgress {
                percent: 0.0,
                status: "error".into(),
                message: err.clone(),
            },
        );
        return Err(if err.is_empty() {
            "yt-dlp 下载失败".into()
        } else {
            err
        });
    }

    let file_path = if last_file.is_empty() {
        out_dir.to_string_lossy().to_string()
    } else {
        last_file
    };

    let do_import = auto_import.unwrap_or(true);
    let mut imported = false;
    let mut import_message = String::new();

    if do_import {
        let out_dir_str = out_dir.to_string_lossy().to_string();
        // Ensure download dir is a library path, then scan it (or import single file)
        let path_obj = std::path::Path::new(&file_path);
        if path_obj.is_file() {
            match scan::import_video_file(file_path.clone()) {
                Ok(msg) => {
                    imported = true;
                    import_message = msg;
                }
                Err(e) => {
                    import_message = format!("单文件导入失败，尝试扫描目录: {e}");
                    let _ = scan::ensure_library_path(out_dir_str.clone());
                    match scan::scan_path(out_dir_str) {
                        Ok(r) => {
                            imported = true;
                            import_message =
                                format!("已扫描下载目录：新增 {}，更新 {}", r.added, r.updated);
                        }
                        Err(e2) => import_message = format!("导入失败: {e2}"),
                    }
                }
            }
        } else {
            let _ = scan::ensure_library_path(out_dir_str.clone());
            match scan::scan_path(out_dir_str) {
                Ok(r) => {
                    imported = true;
                    import_message =
                        format!("已扫描下载目录：新增 {}，更新 {}", r.added, r.updated);
                }
                Err(e) => import_message = format!("导入失败: {e}"),
            }
        }
    }

    let _ = app.emit(
        "ytdlp-progress",
        YtdlpProgress {
            percent: 100.0,
            status: "done".into(),
            message: if import_message.is_empty() {
                file_path.clone()
            } else {
                format!("{file_path}\n{import_message}")
            },
        },
    );

    Ok(DownloadResult {
        path: file_path,
        imported,
        import_message,
    })
}
