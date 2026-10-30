export interface Video {
  id: number;
  title: string;
  path: string;
  size_bytes: number;
  duration_secs: number;
  cover_path: string | null;
  favorite: boolean;
  created_at: string;
  updated_at: string;
  tags: string[];
}

export interface LibraryPath {
  id: number;
  path: string;
  added_at: string;
}

export interface AppSettings {
  language: string;
  ytdlp_path: string;
  download_dir: string;
  whisper_path: string;
  ffmpeg_path: string;
}

export interface ScanResult {
  scanned: number;
  added: number;
  updated: number;
  skipped: number;
}

export interface ToolCheck {
  available: boolean;
  path: string;
  version?: string | null;
  message: string;
}
