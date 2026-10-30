import { useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api } from "../lib/api";
import type { ToolCheck } from "../lib/types";
import { t, type Lang } from "../i18n";
import {
  PLATFORMS,
  detectPlatform,
  resolveDownloadUrl,
  type PlatformId,
} from "../lib/platforms";

export function DownloadsPage({ lang }: { lang: Lang }) {
  const [url, setUrl] = useState("");
  const [platform, setPlatform] = useState<PlatformId>("auto");
  const [check, setCheck] = useState<ToolCheck | null>(null);
  const [percent, setPercent] = useState(0);
  const [status, setStatus] = useState("");
  const [busy, setBusy] = useState(false);
  const [importNote, setImportNote] = useState("");

  const detected = useMemo(() => {
    if (!url.trim()) return null;
    return detectPlatform(url);
  }, [url]);

  const effectivePlatform: Exclude<PlatformId, "auto"> =
    platform === "auto" ? detected ?? "custom" : platform;

  const placeholder = useMemo(() => {
    switch (effectivePlatform) {
      case "bilibili":
        return lang === "zh"
          ? "哔哩哔哩链接，或 BV / av 号"
          : "Bilibili URL or BV / av id";
      case "douyin":
        return lang === "zh" ? "抖音分享链接或视频页 URL" : "Douyin share / video URL";
      case "youtube":
        return lang === "zh"
          ? "YouTube 链接、youtu.be 或 11 位视频 ID"
          : "YouTube URL, youtu.be, or video id";
      default:
        return t(lang, "downloadUrl");
    }
  }, [effectivePlatform, lang]);

  useEffect(() => {
    api.checkYtdlp().then(setCheck).catch((e) =>
      setCheck({ available: false, path: "yt-dlp", message: String(e) }),
    );
    const un = listen<{ percent: number; status: string; message: string }>("ytdlp-progress", (e) => {
      setPercent(e.payload.percent);
      setStatus(`${e.payload.status}: ${e.payload.message}`);
    });
    return () => {
      un.then((f) => f());
    };
  }, []);

  const run = async (audioOnly: boolean) => {
    if (!url.trim()) return;
    setBusy(true);
    setPercent(0);
    setImportNote("");
    setStatus("starting…");
    try {
      const resolved = resolveDownloadUrl(url, platform);
      const out = await api.downloadWithYtdlp(resolved, audioOnly, platform, true);
      setStatus(`done: ${out.path}`);
      setPercent(100);
      if (out.imported) {
        setImportNote(out.import_message || t(lang, "importedToLibrary"));
      } else if (out.import_message) {
        setImportNote(out.import_message);
      }
    } catch (e) {
      setStatus(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="max-w-2xl flex flex-col gap-4">
      <h1 className="text-xl font-semibold">{t(lang, "addVideo")}</h1>
      <p className="text-sm text-white/50">{t(lang, "addVideoDesc")}</p>

      {check && !check.available && (
        <div className="rounded-lg border border-amber-500/30 bg-amber-500/10 p-3 text-sm whitespace-pre-wrap text-amber-100/90">
          {check.message}
        </div>
      )}
      {check?.available && (
        <div className="text-xs text-emerald-300/80">
          yt-dlp OK · {check.path} {check.version ? `· ${check.version}` : ""}
        </div>
      )}

      <div>
        <div className="text-xs text-white/40 mb-2">{t(lang, "platform")}</div>
        <div className="flex flex-wrap gap-2">
          {PLATFORMS.map((p) => {
            const active = platform === p.id;
            const label = lang === "zh" ? p.labelZh : p.labelEn;
            return (
              <button
                key={p.id}
                type="button"
                onClick={() => setPlatform(p.id)}
                className={`rounded-full px-3 py-1.5 text-xs border transition-colors ${
                  active
                    ? "bg-violet-600 border-violet-400 text-white"
                    : "bg-white/5 border-white/10 text-white/70 hover:bg-white/10"
                }`}
              >
                {label}
              </button>
            );
          })}
        </div>
        <div className="mt-2 text-xs text-white/40">
          {lang === "zh"
            ? PLATFORMS.find((p) => p.id === platform)?.hintZh
            : PLATFORMS.find((p) => p.id === platform)?.hintEn}
          {platform === "auto" && detected && (
            <span className="ml-2 text-cyan-300/80">
              → {lang === "zh"
                ? PLATFORMS.find((p) => p.id === detected)?.labelZh
                : PLATFORMS.find((p) => p.id === detected)?.labelEn}
            </span>
          )}
        </div>
      </div>

      <input
        className="rounded-lg bg-black/30 border border-white/10 px-3 py-2 text-sm"
        placeholder={placeholder}
        value={url}
        onChange={(e) => setUrl(e.target.value)}
      />

      <div className="flex gap-2">
        <button
          type="button"
          disabled={busy || !check?.available}
          onClick={() => run(false)}
          className="rounded-lg bg-violet-600 hover:bg-violet-500 px-3 py-2 text-sm disabled:opacity-40"
        >
          {t(lang, "downloadVideo")}
        </button>
        <button
          type="button"
          disabled={busy || !check?.available}
          onClick={() => run(true)}
          className="rounded-lg bg-white/10 px-3 py-2 text-sm disabled:opacity-40"
        >
          {t(lang, "downloadAudio")}
        </button>
      </div>

      <div className="h-2 rounded-full bg-white/10 overflow-hidden">
        <div
          className="h-full bg-gradient-to-r from-violet-500 to-cyan-400 transition-all"
          style={{ width: `${percent}%` }}
        />
      </div>
      <div className="text-xs text-white/50 whitespace-pre-wrap">{status}</div>
      {importNote && (
        <div className="rounded-lg border border-emerald-500/30 bg-emerald-500/10 p-3 text-sm text-emerald-100/90">
          {importNote}
        </div>
      )}
    </div>
  );
}
