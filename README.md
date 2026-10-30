# Lumina / 简影库

本地视频合集管理桌面应用。基于 **Tauri 2 + React + TypeScript + Tailwind CSS + SQLite**，默认中文界面。

## 功能

- **本地视频库**：添加扫描目录，递归扫描常见视频扩展名（mp4 / mkv / avi / mov / webm 等），元数据写入 SQLite
- **海报墙 / 详情**：网格浏览、详情页、收藏、标签、标题/路径/标签搜索
- **播放**：调用系统默认播放器打开文件；详情页提供 HTML5 播放入口
- **封面**：通过 FFmpeg 抽帧生成封面；失败时使用占位图
- **平台接入（v0.1.1）**：哔哩哔哩 / 抖音 / YouTube / 自定义链接；下载页可选平台或粘贴链接自动识别；支持 BV/av、YouTube ID；yt-dlp 进度事件；下载完成后自动扫入本地视频库（SQLite）
- **yt-dlp 集成**：下载视频/音频，解析进度百分比并通过事件推送到前端；未安装时友好提示
- **字幕翻译**：Whisper CLI 调用框架（可配置路径）；未安装时提示
- **界面语言**：设置中支持中/英切换
- **资源嗅探**：占位页面，说明后续扩展方向
- **设置页**：库路径、yt-dlp / Whisper / FFmpeg 路径、下载目录、语言

## 环境要求

| 组件 | 说明 |
|------|------|
| Node.js 18+ | 前端构建 |
| Rust 1.77+ | 后端（本仓库在 1.85 验证） |
| 系统依赖 | 见 [Tauri Linux 前置条件](https://tauri.app/start/prerequisites/)（WebKitGTK、GTK 等） |
| FFmpeg（可选） | 封面抽帧，常见路径 `/usr/bin/ffmpeg` |
| yt-dlp（可选） | 下载功能 |
| Whisper CLI（可选） | 字幕转录，如 `pip install openai-whisper` |

### 可选依赖安装示例

```bash
# yt-dlp
pip install yt-dlp
# 或
brew install yt-dlp

# Whisper
pip install openai-whisper

# FFmpeg（Debian/Ubuntu）
sudo apt install ffmpeg
```

## 开发运行

```bash
git clone https://github.com/<你的账号>/lumina-video-library.git
cd lumina-video-library
npm install
npm run tauri dev
```

仅预览前端（无 Rust 命令）：

```bash
npm run dev
```

## 构建

```bash
# 前端类型检查 + 打包
npm run build

# Rust 静态检查（在 src-tauri）
cd src-tauri && cargo check

# 完整桌面安装包（需要本机已安装 WebKitGTK 等）
npm run tauri build
```

### 关于本 CI / 无头 Linux 环境

部分精简 Linux 环境可能缺少 `pkg-config`、 `webkit2gtk` / `librsvg2` 等系统库，导致无法完成完整的 `tauri build` 链接阶段。此时：

- `npm run build`（前端）应可通过
- `cargo check`（在 `src-tauri`）应尽量通过（`bundled` SQLite，无需系统 libsqlite）
- 代码结构完整，在已安装 Tauri 官方前置依赖的 Windows / macOS / 完整 Linux 桌面上可正常 `tauri build`

Debian/Ubuntu 可参考：

```bash
sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev patchelf
```

## 项目结构

```
lumina-video-library/
  README.md
  package.json
  src/                 # React 前端
    pages/             # 库 / 详情 / 下载 / 字幕 / 嗅探 / 设置
    components/
    lib/               # Tauri invoke 封装与类型
    i18n/              # 中英文案
  src-tauri/
    Cargo.toml
    src/
      main.rs
      lib.rs
      commands/        # library, scan, ytdlp, whisper, settings, ffmpeg
      db/              # SQLite 初始化与模型
```

## 数据位置

应用数据（SQLite、封面、默认下载目录）位于系统用户数据目录下的 `lumina-video-library/`（可在设置页查看完整路径）。数据库文件已被 `.gitignore` 排除。

## 许可证

MIT
