use crate::commands::settings;
use serde::Serialize;
use std::process::Command;
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Clone)]
pub struct WhisperProgress {
    pub status: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct WhisperCheck {
    pub available: bool,
    pub path: String,
    pub message: String,
}

#[tauri::command]
pub fn check_whisper() -> Result<WhisperCheck, String> {
    let s = settings::get_settings()?;
    let path = s.whisper_path;
    let output = Command::new(&path).arg("--help").output();
    match output {
        Ok(o) if o.status.success() || !o.stdout.is_empty() || !o.stderr.is_empty() => {
            Ok(WhisperCheck {
                available: true,
                path,
                message: "Whisper CLI 可用（字幕翻译框架已就绪）".into(),
            })
        }
        _ => Ok(WhisperCheck {
            available: false,
            path: path.clone(),
            message: format!(
                "未找到 Whisper CLI（尝试路径: {path}）。\n\
                 安装示例: pip install openai-whisper\n\
                 或在设置中配置 whisper / whisper.cpp 可执行文件路径。"
            ),
        }),
    }
}

/// 字幕转录/翻译占位：调用配置的 Whisper CLI。
/// 真实环境中可扩展为目标语言翻译流水线。
#[tauri::command]
pub async fn transcribe_with_whisper(
    app: AppHandle,
    media_path: String,
    language: Option<String>,
) -> Result<String, String> {
    let check = check_whisper()?;
    if !check.available {
        let _ = app.emit(
            "whisper-progress",
            WhisperProgress {
                status: "unavailable".into(),
                message: check.message.clone(),
            },
        );
        return Err(check.message);
    }
    if !std::path::Path::new(&media_path).exists() {
        return Err(format!("媒体文件不存在: {media_path}"));
    }

    let lang = language.unwrap_or_else(|| "zh".into());
    let _ = app.emit(
        "whisper-progress",
        WhisperProgress {
            status: "running".into(),
            message: format!("正在调用 Whisper 处理: {media_path}"),
        },
    );

    // 占位调用：openai-whisper 风格参数；失败时返回友好错误
    let output = Command::new(&check.path)
        .args([
            &media_path,
            "--language",
            &lang,
            "--task",
            "transcribe",
            "--output_format",
            "srt",
        ])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let msg = "Whisper 转录完成（请查看媒体同目录下的 .srt）".to_string();
            let _ = app.emit(
                "whisper-progress",
                WhisperProgress {
                    status: "done".into(),
                    message: msg.clone(),
                },
            );
            Ok(msg)
        }
        Ok(o) => {
            let err = format!(
                "Whisper 执行失败:\n{}",
                String::from_utf8_lossy(&o.stderr)
            );
            let _ = app.emit(
                "whisper-progress",
                WhisperProgress {
                    status: "error".into(),
                    message: err.clone(),
                },
            );
            Err(err)
        }
        Err(e) => {
            let err = format!("无法启动 Whisper: {e}");
            let _ = app.emit(
                "whisper-progress",
                WhisperProgress {
                    status: "error".into(),
                    message: err.clone(),
                },
            );
            Err(err)
        }
    }
}
