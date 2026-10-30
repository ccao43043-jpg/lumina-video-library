import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api } from "../lib/api";
import type { ToolCheck } from "../lib/types";
import { t, type Lang } from "../i18n";

export function DownloadsPage({ lang }: { lang: Lang }) {
  const [url, setUrl] = useState("");
  const [check, setCheck] = useState<ToolCheck | null>(null);
  const [percent, setPercent] = useState(0);
  const [status, setStatus] = useState("");
  const [busy, setBusy] = useState(false);

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
    setStatus("starting…");
    try {
      const out = await api.downloadWithYtdlp(url.trim(), audioOnly);
      setStatus(`done: ${out}`);
      setPercent(100);
    } catch (e) {
      setStatus(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="max-w-2xl flex flex-col gap-4">
      <h1 className="text-xl font-semibold">{t(lang, "downloads")}</h1>
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
      <input
        className="rounded-lg bg-black/30 border border-white/10 px-3 py-2 text-sm"
        placeholder={t(lang, "downloadUrl")}
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
        <div className="h-full bg-gradient-to-r from-violet-500 to-cyan-400 transition-all" style={{ width: `${percent}%` }} />
      </div>
      <div className="text-xs text-white/50 whitespace-pre-wrap">{status}</div>
    </div>
  );
}
