import { t, type Lang } from "../i18n";

export function SnifferPage({ lang }: { lang: Lang }) {
  return (
    <div className="max-w-2xl flex flex-col gap-4">
      <h1 className="text-xl font-semibold">{t(lang, "snifferTitle")}</h1>
      <p className="text-sm text-white/60 leading-relaxed">{t(lang, "snifferDesc")}</p>
      <div className="rounded-xl border border-dashed border-white/20 bg-white/[0.03] p-8 text-center text-white/40 text-sm">
        <div className="text-3xl mb-3 opacity-50">◎</div>
        <div>API stub · `commands` 可扩展网络嗅探 / 浏览器扩展桥接</div>
        <ul className="mt-4 text-left max-w-md mx-auto space-y-1 text-xs text-white/35">
          <li>· 网页媒体直链提取</li>
          <li>· M3U8 / DASH 清单解析</li>
          <li>· 一键加入 yt-dlp 下载队列</li>
        </ul>
      </div>
    </div>
  );
}
