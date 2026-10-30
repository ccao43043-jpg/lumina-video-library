import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type { AppSettings, LibraryPath, ScanResult, ToolCheck, Video } from "./types";

export const api = {
  listVideos: (query?: string, favoritesOnly?: boolean) =>
    invoke<Video[]>("list_videos", { query: query ?? null, favoritesOnly: favoritesOnly ?? null }),
  getVideo: (id: number) => invoke<Video | null>("get_video", { id }),
  toggleFavorite: (id: number) => invoke<Video>("toggle_favorite", { id }),
  setVideoTags: (id: number, tags: string[]) => invoke<Video>("set_video_tags", { id, tags }),
  listAllTags: () => invoke<string[]>("list_all_tags"),
  listLibraryPaths: () => invoke<LibraryPath[]>("list_library_paths"),
  addLibraryPath: (path: string) => invoke<LibraryPath>("add_library_path", { path }),
  removeLibraryPath: (id: number) => invoke<void>("remove_library_path", { id }),
  openInPlayer: (path: string) =>
    invoke<{ opened: boolean; path: string; message: string }>("open_in_player", { path }),
  deleteVideo: (id: number) => invoke<void>("delete_video", { id }),
  scanLibrary: () => invoke<ScanResult>("scan_library"),
  scanPath: (path: string) => invoke<ScanResult>("scan_path", { path }),
  getSettings: () => invoke<AppSettings>("get_settings"),
  saveSettings: (settings: AppSettings) => invoke<AppSettings>("save_settings", { settings }),
  appDataPath: () => invoke<string>("app_data_path"),
  checkYtdlp: () => invoke<ToolCheck>("check_ytdlp"),
  downloadWithYtdlp: (url: string, audioOnly: boolean) =>
    invoke<string>("download_with_ytdlp", { url, audioOnly }),
  checkWhisper: () => invoke<ToolCheck>("check_whisper"),
  transcribeWithWhisper: (mediaPath: string, language?: string) =>
    invoke<string>("transcribe_with_whisper", { mediaPath, language: language ?? null }),
  checkFfmpeg: () => invoke<ToolCheck>("check_ffmpeg"),
  regenerateCover: (videoId: number) =>
    invoke<string | null>("regenerate_cover", { videoId }),
};

export function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 ** 2) return `${(n / 1024).toFixed(1)} KB`;
  if (n < 1024 ** 3) return `${(n / 1024 ** 2).toFixed(1)} MB`;
  return `${(n / 1024 ** 3).toFixed(2)} GB`;
}

export function coverSrc(path: string | null): string | null {
  if (!path) return null;
  try {
    return convertFileSrc(path);
  } catch {
    return null;
  }
}
