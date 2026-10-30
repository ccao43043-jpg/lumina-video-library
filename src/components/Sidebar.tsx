import { NavLink } from "react-router-dom";
import { t, type Lang } from "../i18n";

const links: { to: string; key: "library" | "favorites" | "downloads" | "subtitles" | "sniffer" | "settings" }[] = [
  { to: "/", key: "library" },
  { to: "/favorites", key: "favorites" },
  { to: "/downloads", key: "downloads" },
  { to: "/subtitles", key: "subtitles" },
  { to: "/sniffer", key: "sniffer" },
  { to: "/settings", key: "settings" },
];

export function Sidebar({ lang }: { lang: Lang }) {
  return (
    <aside className="w-56 shrink-0 border-r border-white/10 bg-black/20 backdrop-blur-md p-4 flex flex-col gap-6">
      <div>
        <div className="text-xl font-bold tracking-wide bg-gradient-to-r from-violet-300 to-cyan-300 bg-clip-text text-transparent">
          Lumina
        </div>
        <div className="text-xs text-white/50 mt-0.5">{t(lang, "appName")}</div>
      </div>
      <nav className="flex flex-col gap-1">
        {links.map((l) => (
          <NavLink
            key={l.to}
            to={l.to}
            end={l.to === "/"}
            className={({ isActive }) =>
              `rounded-lg px-3 py-2 text-sm transition ${
                isActive
                  ? "bg-violet-500/25 text-violet-100 border border-violet-400/30"
                  : "text-white/70 hover:bg-white/5 hover:text-white"
              }`
            }
          >
            {t(lang, l.key)}
          </NavLink>
        ))}
      </nav>
      <div className="mt-auto text-[10px] text-white/30">v0.1.0 · Tauri 2</div>
    </aside>
  );
}
