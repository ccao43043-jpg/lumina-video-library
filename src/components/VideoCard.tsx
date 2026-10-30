import { Link } from "react-router-dom";
import type { Video } from "../lib/types";
import { coverSrc, formatBytes } from "../lib/api";
import { t, type Lang } from "../i18n";

export function VideoCard({
  video,
  lang,
  onToggleFavorite,
}: {
  video: Video;
  lang: Lang;
  onToggleFavorite: (id: number) => void;
}) {
  const src = coverSrc(video.cover_path);

  return (
    <div className="group rounded-xl overflow-hidden border border-white/10 bg-white/5 hover:border-violet-400/40 hover:bg-white/[0.07] transition shadow-lg shadow-black/20">
      <Link to={`/video/${video.id}`} className="block aspect-[16/10] bg-slate-900 relative">
        {src ? (
          <img src={src} alt={video.title} className="w-full h-full object-cover" />
        ) : (
          <div className="w-full h-full flex items-center justify-center text-white/30 text-sm">
            {t(lang, "placeholderCover")}
          </div>
        )}
        {video.favorite && (
          <span className="absolute top-2 right-2 text-amber-300 text-lg drop-shadow">★</span>
        )}
      </Link>
      <div className="p-3">
        <Link to={`/video/${video.id}`} className="font-medium text-sm line-clamp-2 hover:text-violet-200">
          {video.title}
        </Link>
        <div className="mt-1 text-[11px] text-white/40 flex justify-between gap-2">
          <span>{formatBytes(video.size_bytes)}</span>
          <button
            type="button"
            className="hover:text-amber-300"
            onClick={() => onToggleFavorite(video.id)}
            title={video.favorite ? t(lang, "unfavorite") : t(lang, "favorite")}
          >
            {video.favorite ? "★" : "☆"}
          </button>
        </div>
        {video.tags.length > 0 && (
          <div className="mt-2 flex flex-wrap gap-1">
            {video.tags.slice(0, 4).map((tag) => (
              <span key={tag} className="text-[10px] px-1.5 py-0.5 rounded bg-cyan-500/15 text-cyan-200/80">
                {tag}
              </span>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
