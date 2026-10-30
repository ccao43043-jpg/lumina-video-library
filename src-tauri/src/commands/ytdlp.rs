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

fn resolve_ytdlp() -> Result<String, String> {
    let s = settings::get_settings()?;
    Ok(s.ytdlp_path)
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
pub async fn download_with_ytdlp(
    app: AppHandle,
    url: String,
    audio_only: bool,
) -> Result<String, String> {
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

    let template = out_dir
        .join("%(title)s.%(ext)s")
        .to_string_lossy()
        .to_string();

    let mut args = vec![
        "--newline".to_string(),
        "--progress".to_string(),
        "-o".to_string(),
        template,
    ];
    if audio_only {
        args.extend(["-x".into(), "--audio-format".into(), "mp3".into()]);
    }
    args.push(url);

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

    let _ = app.emit(
        "ytdlp-progress",
        YtdlpProgress {
            percent: 100.0,
            status: "done".into(),
            message: last_file.clone(),
        },
    );
    Ok(if last_file.is_empty() {
        out_dir.to_string_lossy().to_string()
    } else {
        last_file
    })
}
