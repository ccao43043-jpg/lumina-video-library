import { useCallback, useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { api } from "../lib/api";
import type { Video } from "../lib/types";
import { VideoCard } from "../components/VideoCard";
import { t, type Lang } from "../i18n";

export function LibraryPage({ lang, favoritesOnly = false }: { lang: Lang; favoritesOnly?: boolean }) {
  const [videos, setVideos] = useState<Video[]>([]);
  const [q, setQ] = useState("");
  const [busy, setBusy] = useState(false);
  const [msg, setMsg] = useState("");

  const load = useCallback(async () => {
    try {
      const list = await api.listVideos(q || undefined, favoritesOnly);
      setVideos(list);
    } catch (e) {
      setMsg(String(e));
    }
  }, [q, favoritesOnly]);

  useEffect(() => {
    load();
  }, [load]);

  const scan = async () => {
    setBusy(true);
    setMsg(t(lang, "scanning"));
    try {
      const r = await api.scanLibrary();
      setMsg(`扫描 ${r.scanned} · 新增 ${r.added} · 更新 ${r.updated}`);
      await load();
    } catch (e) {
      setMsg(String(e));
    } finally {
      setBusy(false);
    }
  };

  const addFolder = async () => {
    try {
      const selected = await open({ directory: true, multiple: false });
      if (!selected || typeof selected !== "string") return;
      setBusy(true);
      const r = await api.scanPath(selected);
      setMsg(`已添加并扫描：新增 ${r.added}`);
      await load();
    } catch (e) {
      setMsg(String(e));
    } finally {
      setBusy(false);
    }
  };

  const toggleFav = async (id: number) => {
    await api.toggleFavorite(id);
    await load();
  };

  return (
    <div className="flex flex-col gap-4 h-full">
      <div className="flex flex-wrap items-center gap-2">
        <input
          className="flex-1 min-w-[200px] rounded-lg bg-black/30 border border-white/10 px-3 py-2 text-sm outline-none focus:border-violet-400/50"
          placeholder={t(lang, "searchPlaceholder")}
          value={q}
          onChange={(e) => setQ(e.target.value)}
        />
        {!favoritesOnly && (
          <>
            <button
              type="button"
              disabled={busy}
              onClick={addFolder}
              className="rounded-lg bg-white/10 hover:bg-white/15 px-3 py-2 text-sm disabled:opacity-50"
            >
              {t(lang, "addFolder")}
            </button>
            <button
              type="button"
              disabled={busy}
              onClick={scan}
              className="rounded-lg bg-violet-600 hover:bg-violet-500 px-3 py-2 text-sm font-medium disabled:opacity-50"
            >
              {busy ? t(lang, "scanning") : t(lang, "scan")}
            </button>
          </>
        )}
      </div>
      {msg && <div className="text-xs text-cyan-200/70">{msg}</div>}
      {videos.length === 0 ? (
        <div className="flex-1 flex items-center justify-center text-white/40 text-sm">{t(lang, "noVideos")}</div>
      ) : (
        <div className="grid grid-cols-2 md:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-4 overflow-y-auto pb-8">
          {videos.map((v) => (
            <VideoCard key={v.id} video={v} lang={lang} onToggleFavorite={toggleFav} />
          ))}
        </div>
      )}
    </div>
  );
}
