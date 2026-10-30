import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api } from "../lib/api";
import type { ToolCheck } from "../lib/types";
import { t, type Lang } from "../i18n";

export function SubtitlesPage({ lang }: { lang: Lang }) {
  const [path, setPath] = useState("");
  const [check, setCheck] = useState<ToolCheck | null>(null);
  const [status, setStatus] = useState("");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    api.checkWhisper().then(setCheck).catch((e) =>
      setCheck({ available: false, path: "whisper", message: String(e) }),
    );
    const un = listen<{ status: string; message: string }>("whisper-progress", (e) => {
      setStatus(`${e.payload.status}: ${e.payload.message}`);
    });
    return () => {
      un.then((f) => f());
    };
  }, []);

  const run = async () => {
    if (!path.trim()) return;
    setBusy(true);
    try {
      const msg = await api.transcribeWithWhisper(path.trim(), lang === "zh" ? "zh" : "en");
      setStatus(msg);
    } catch (e) {
      setStatus(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="max-w-2xl flex flex-col gap-4">
      <h1 className="text-xl font-semibold">{t(lang, "subtitles")}</h1>
      <p className="text-sm text-white/50">{t(lang, "subtitleDesc")}</p>
      {check && !check.available && (
        <div className="rounded-lg border border-amber-500/30 bg-amber-500/10 p-3 text-sm whitespace-pre-wrap">
          {check.message}
        </div>
      )}
      <input
        className="rounded-lg bg-black/30 border border-white/10 px-3 py-2 text-sm"
        placeholder={t(lang, "mediaPath")}
        value={path}
        onChange={(e) => setPath(e.target.value)}
      />
      <button
        type="button"
        disabled={busy || !check?.available}
        onClick={run}
        className="w-fit rounded-lg bg-violet-600 hover:bg-violet-500 px-3 py-2 text-sm disabled:opacity-40"
      >
        {t(lang, "startTranscribe")}
      </button>
      <div className="text-xs text-white/50 whitespace-pre-wrap">{status}</div>
    </div>
  );
}
