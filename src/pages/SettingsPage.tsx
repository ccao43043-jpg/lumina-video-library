import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { api } from "../lib/api";
import type { AppSettings, LibraryPath, ToolCheck } from "../lib/types";
import { t, type Lang } from "../i18n";

export function SettingsPage({
  lang,
  onLangChange,
}: {
  lang: Lang;
  onLangChange: (l: Lang) => void;
}) {
  const [s, setS] = useState<AppSettings | null>(null);
  const [paths, setPaths] = useState<LibraryPath[]>([]);
  const [appData, setAppData] = useState("");
  const [tools, setTools] = useState<{ ytdlp?: ToolCheck; whisper?: ToolCheck; ffmpeg?: ToolCheck }>({});
  const [msg, setMsg] = useState("");

  const reload = async () => {
    const settings = await api.getSettings();
    setS(settings);
    setPaths(await api.listLibraryPaths());
    setAppData(await api.appDataPath());
    const [ytdlp, whisper, ffmpeg] = await Promise.all([
      api.checkYtdlp(),
      api.checkWhisper(),
      api.checkFfmpeg(),
    ]);
    setTools({ ytdlp, whisper, ffmpeg });
  };

  useEffect(() => {
    reload().catch((e) => setMsg(String(e)));
  }, []);

  if (!s) return <div className="text-sm text-white/40">…</div>;

  const save = async () => {
    const next = await api.saveSettings(s);
    setS(next);
    onLangChange(next.language === "en" ? "en" : "zh");
    setMsg(t(lang, "saved"));
    await reload();
  };

  const addFolder = async () => {
    const selected = await open({ directory: true, multiple: false });
    if (!selected || typeof selected !== "string") return;
    await api.addLibraryPath(selected);
    setPaths(await api.listLibraryPaths());
  };

  return (
    <div className="max-w-2xl flex flex-col gap-6">
      <h1 className="text-xl font-semibold">{t(lang, "settings")}</h1>

      <section className="space-y-3">
        <label className="block text-xs text-white/40">{t(lang, "language")}</label>
        <select
          className="rounded-lg bg-black/30 border border-white/10 px-3 py-2 text-sm"
          value={s.language}
          onChange={(e) => {
            const language = e.target.value;
            setS({ ...s, language });
            onLangChange(language === "en" ? "en" : "zh");
          }}
        >
          <option value="zh">中文</option>
          <option value="en">English</option>
        </select>
      </section>

      <section className="space-y-2">
        <div className="text-xs text-white/40">{t(lang, "libraryPaths")}</div>
        <ul className="space-y-1">
          {paths.map((p) => (
            <li key={p.id} className="flex items-center gap-2 text-sm bg-black/20 rounded-lg px-3 py-2">
              <span className="flex-1 break-all text-white/70">{p.path}</span>
              <button
                type="button"
                className="text-xs text-rose-300/80"
                onClick={async () => {
                  await api.removeLibraryPath(p.id);
                  setPaths(await api.listLibraryPaths());
                }}
              >
                {t(lang, "removePath")}
              </button>
            </li>
          ))}
        </ul>
        <button type="button" onClick={addFolder} className="rounded-lg bg-white/10 px-3 py-2 text-sm">
          {t(lang, "addFolder")}
        </button>
      </section>

      {(
        [
          ["ytdlp_path", "ytdlpPath"],
          ["download_dir", "downloadDir"],
          ["whisper_path", "whisperPath"],
          ["ffmpeg_path", "ffmpegPath"],
        ] as const
      ).map(([field, label]) => (
        <section key={field} className="space-y-1">
          <label className="text-xs text-white/40">{t(lang, label)}</label>
          <input
            className="w-full rounded-lg bg-black/30 border border-white/10 px-3 py-2 text-sm"
            value={s[field]}
            onChange={(e) => setS({ ...s, [field]: e.target.value })}
          />
        </section>
      ))}

      <section>
        <div className="text-xs text-white/40 mb-2">{t(lang, "toolsStatus")}</div>
        <div className="space-y-1 text-xs">
          {tools.ytdlp && (
            <div className={tools.ytdlp.available ? "text-emerald-300/80" : "text-amber-200/80"}>
              yt-dlp: {tools.ytdlp.message.split("\n")[0]}
            </div>
          )}
          {tools.whisper && (
            <div className={tools.whisper.available ? "text-emerald-300/80" : "text-amber-200/80"}>
              Whisper: {tools.whisper.message.split("\n")[0]}
            </div>
          )}
          {tools.ffmpeg && (
            <div className={tools.ffmpeg.available ? "text-emerald-300/80" : "text-amber-200/80"}>
              FFmpeg: {tools.ffmpeg.message.split("\n")[0]}
            </div>
          )}
        </div>
      </section>

      <div className="text-xs text-white/35">
        {t(lang, "appData")}: {appData}
      </div>

      <button type="button" onClick={save} className="w-fit rounded-lg bg-violet-600 hover:bg-violet-500 px-4 py-2 text-sm">
        {t(lang, "save")}
      </button>
      {msg && <div className="text-xs text-cyan-200/70">{msg}</div>}
    </div>
  );
}
