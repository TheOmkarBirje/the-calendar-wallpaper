import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface AppSettings {
  wallpaper_url: string;
  update_time: string;
}

function App() {
  const [url, setUrl] = useState("");
  const [time, setTime] = useState("00:00");
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    // Load existing settings on mount
    invoke<AppSettings>("get_settings")
      .then((settings) => {
        setUrl(settings.wallpaper_url);
        setTime(settings.update_time);
      })
      .catch(console.error);
  }, []);

  const handleUpdate = async () => {
    setLoading(true);
    setError(null);
    setStatus(null);

    try {
      // Save settings first
      await invoke("save_settings", {
        settings: { wallpaper_url: url, update_time: time }
      });

      // Trigger immediate update
      const msg = await invoke<string>("set_wallpaper", { url });
      setStatus(msg);
    } catch (e: any) {
      setError(e.toString());
    } finally {
      setLoading(false);
    }
  };

  return (
    <main className="container">
      <h1>The Calendar Wallpaper</h1>
      <p className="description">Automate your desktop background with ease.</p>

      <div className="input-group">
        <label htmlFor="url-input">Wallpaper URL (PNG/JPG)</label>
        <input
          id="url-input"
          type="text"
          placeholder="https://example.com/wallpaper.png"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
        />
      </div>

      <div className="input-group">
        <label htmlFor="time-input">Daily Update Time (24h)</label>
        <input
          id="time-input"
          type="text"
          placeholder="HH:MM (e.g. 14:30)"
          value={time}
          onChange={(e) => setTime(e.target.value)}
          pattern="^([0-1]?[0-9]|2[0-3]):[0-5][0-9]$"
        />
      </div>

      <button
        className="button-primary"
        onClick={handleUpdate}
        disabled={loading}
      >
        {loading ? "Updating..." : "Set & Save"}
      </button>

      {error && <p className="error-text">{error}</p>}

      {status && (
        <div className="status-badge">
          <div style={{ width: 8, height: 8, borderRadius: '50%', background: '#10b981' }} />
          {status}
        </div>
      )}
    </main>
  );
}

export default App;
