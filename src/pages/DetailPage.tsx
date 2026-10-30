import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { convertFileSrc } from "@tauri-apps/api/core";
import { open as openPath } from "@tauri-apps/plugin-shell";
import { api, coverSrc, formatBytes } from "../lib/api";
import type { Video } from "../lib/types";
import { t, type Lang } from "../i18n";

export function DetailPage({ lang }: { lang: Lang }) {
  const { id } = useParams();
  const [video, setVideo] = useState<Video | null>(null);
  const [tags, setTags] = useState("");
  const [src, setSrc] = useState<string | null>(null);
  const [mediaSrc, setMediaSrc] = useState<string | null>(null);
  const [msg, setMsg] = useState("");
  const [useEmbed, setUseEmbed] = useState(false);

  const load = async () => {
    if (!id) return;
    const v = await api.getVideo(Number(id));
    setVideo(v);
    if (v) {
      setTags(v.tags.join(", "));
      setSrc(coverSrc(v.cover_path));
      try {
        setMediaSrc(convertFileSrc(v.path));
      } catch {
        setMediaSrc(null);
      }
    }
  };

  useEffect(() => {
    load().catch((e) => setMsg(String(e)));
  }, [id]);

  if (!video) {
    return <div className="text-white/50 text-sm">{msg || "…"}</div>;
  }

  const playExternal = async () => {
    try {
      await api.openInPlayer(video.path);
      try {
        await openPath(video.path);
      } catch {
        /* fallback already in Rust */
      }
      setMsg(t(lang, "openExternal"));
    } catch (e) {
      setMsg(String(e));
    }
  };

  const saveTags = async () => {
    const list = tags
      .split(/[,，]/)
      .map((s) => s.trim())
      .filter(Boolean);
    const v = await api.setVideoTags(video.id, list);
    setVideo(v);
    setMsg(t(lang, "saved"));
  };

  const toggleFav = async () => {
    setVideo(await api.toggleFavorite(video.id));
  };

  const regen = async () => {
    const cover = await api.regenerateCover(video.id);
    setMsg(cover ? `封面: ${cover}` : "封面生成失败，将使用占位图");
    await load();
  };

  return (
    <div className="flex flex-col gap-4 max-w-4xl">
      <Link to="/" className="text-sm text-violet-300 hover:underline w-fit">
        ← {t(lang, "back")}
      </Link>
      <div className="grid md:grid-cols-2 gap-6">
        <div className="rounded-xl overflow-hidden border border-white/10 bg-black/30 aspect-video flex items-center justify-center">
          {useEmbed && mediaSrc ? (
            <video className="w-full h-full" controls src={mediaSrc} />
          ) : src ? (
            <img src={src} alt="" className="w-full h-full object-cover" />
          ) : (
            <span className="text-white/30">{t(lang, "placeholderCover")}</span>
          )}
        </div>
        <div className="flex flex-col gap-3">
          <h1 className="text-2xl font-semibold">{video.title}</h1>
          <div className="text-xs text-white/50 break-all">
            <div>
              <span className="text-white/30">{t(lang, "path")}: </span>
              {video.path}
            </div>
            <div>
              <span className="text-white/30">{t(lang, "size")}: </span>
              {formatBytes(video.size_bytes)}
            </div>
          </div>
          <div className="flex flex-wrap gap-2">
            <button
              type="button"
              onClick={playExternal}
              className="rounded-lg bg-violet-600 hover:bg-violet-500 px-3 py-2 text-sm"
            >
              {t(lang, "openExternal")}
            </button>
            <button
              type="button"
              onClick={() => setUseEmbed((x) => !x)}
              className="rounded-lg bg-white/10 hover:bg-white/15 px-3 py-2 text-sm"
            >
              {t(lang, "play")} (HTML5)
            </button>
            <button type="button" onClick={toggleFav} className="rounded-lg bg-white/10 px-3 py-2 text-sm">
              {video.favorite ? t(lang, "unfavorite") : t(lang, "favorite")} {video.favorite ? "★" : "☆"}
            </button>
            <button type="button" onClick={regen} className="rounded-lg bg-white/10 px-3 py-2 text-sm">
              {t(lang, "regenCover")}
            </button>
          </div>
          <div>
            <label className="text-xs text-white/40">{t(lang, "tags")}（逗号分隔）</label>
            <div className="flex gap-2 mt-1">
              <input
                className="flex-1 rounded-lg bg-black/30 border border-white/10 px-3 py-2 text-sm"
                value={tags}
                onChange={(e) => setTags(e.target.value)}
              />
              <button type="button" onClick={saveTags} className="rounded-lg bg-cyan-700/80 px-3 py-2 text-sm">
                {t(lang, "saveTags")}
              </button>
            </div>
          </div>
          {msg && <div className="text-xs text-cyan-200/70">{msg}</div>}
        </div>
      </div>
    </div>
  );
}
