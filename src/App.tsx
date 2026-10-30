import { useEffect, useState } from "react";
import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import { Sidebar } from "./components/Sidebar";
import { LibraryPage } from "./pages/LibraryPage";
import { DetailPage } from "./pages/DetailPage";
import { DownloadsPage } from "./pages/DownloadsPage";
import { SubtitlesPage } from "./pages/SubtitlesPage";
import { SnifferPage } from "./pages/SnifferPage";
import { SettingsPage } from "./pages/SettingsPage";
import { api } from "./lib/api";
import type { Lang } from "./i18n";

export default function App() {
  const [lang, setLang] = useState<Lang>("zh");

  useEffect(() => {
    api
      .getSettings()
      .then((s) => setLang(s.language === "en" ? "en" : "zh"))
      .catch(() => setLang("zh"));
  }, []);

  return (
    <BrowserRouter>
      <div className="flex h-screen overflow-hidden">
        <Sidebar lang={lang} />
        <main className="flex-1 overflow-y-auto p-6">
          <Routes>
            <Route path="/" element={<LibraryPage lang={lang} />} />
            <Route path="/favorites" element={<LibraryPage lang={lang} favoritesOnly />} />
            <Route path="/video/:id" element={<DetailPage lang={lang} />} />
            <Route path="/downloads" element={<DownloadsPage lang={lang} />} />
            <Route path="/subtitles" element={<SubtitlesPage lang={lang} />} />
            <Route path="/sniffer" element={<SnifferPage lang={lang} />} />
            <Route
              path="/settings"
              element={<SettingsPage lang={lang} onLangChange={setLang} />}
            />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </main>
      </div>
    </BrowserRouter>
  );
}
