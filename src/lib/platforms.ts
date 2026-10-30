export type PlatformId = "auto" | "bilibili" | "douyin" | "youtube" | "custom";

export const PLATFORMS: { id: PlatformId; labelZh: string; labelEn: string; hintZh: string; hintEn: string }[] = [
  { id: "auto", labelZh: "自动识别", labelEn: "Auto", hintZh: "粘贴链接后自动判断平台", hintEn: "Detect platform from URL" },
  { id: "bilibili", labelZh: "哔哩哔哩", labelEn: "Bilibili", hintZh: "支持完整链接或 BV / av 号", hintEn: "Full URL or BV / av id" },
  { id: "douyin", labelZh: "抖音", labelEn: "Douyin", hintZh: "支持抖音分享链接或视频页 URL", hintEn: "Douyin share or video URL" },
  { id: "youtube", labelZh: "YouTube", labelEn: "YouTube", hintZh: "支持完整链接、youtu.be 或视频 ID", hintEn: "URL, youtu.be, or video id" },
  { id: "custom", labelZh: "自定义", labelEn: "Custom", hintZh: "任意 yt-dlp 支持的链接或模板", hintEn: "Any yt-dlp URL or template" },
];

export function detectPlatform(input: string): Exclude<PlatformId, "auto"> {
  const s = input.trim();
  const lower = s.toLowerCase();
  if (
    lower.includes("bilibili.com") ||
    lower.includes("b23.tv") ||
    /^bv[\w]+$/i.test(s) ||
    /^av\d+$/i.test(s)
  ) {
    return "bilibili";
  }
  if (
    lower.includes("douyin.com") ||
    lower.includes("iesdouyin.com") ||
    lower.includes("v.douyin.com")
  ) {
    return "douyin";
  }
  if (
    lower.includes("youtube.com") ||
    lower.includes("youtu.be") ||
    lower.includes("youtube-nocookie.com") ||
    /^[\w-]{11}$/.test(s)
  ) {
    return "youtube";
  }
  return "custom";
}

/** Normalize user input (URL or id) into a downloadable URL for yt-dlp. */
export function resolveDownloadUrl(input: string, platform: PlatformId): string {
  const raw = input.trim();
  if (!raw) return raw;
  const p = platform === "auto" ? detectPlatform(raw) : platform;

  if (p === "bilibili") {
    if (/^https?:\/\//i.test(raw)) return raw;
    if (/^bv[\w]+$/i.test(raw)) return `https://www.bilibili.com/video/${raw}`;
    if (/^av(\d+)$/i.test(raw)) {
      const aid = raw.replace(/^av/i, "");
      return `https://www.bilibili.com/video/av${aid}`;
    }
    return raw;
  }

  if (p === "youtube") {
    if (/^https?:\/\//i.test(raw)) return raw;
    if (/^[\w-]{11}$/.test(raw)) return `https://www.youtube.com/watch?v=${raw}`;
    return raw;
  }

  if (p === "douyin") {
    if (/^https?:\/\//i.test(raw)) return raw;
    // Douyin IDs alone are unreliable; pass through for custom templates
    return raw;
  }

  // custom: allow any URL or yt-dlp template as-is
  return raw;
}
