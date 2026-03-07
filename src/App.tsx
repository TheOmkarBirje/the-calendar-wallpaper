import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart';
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
  const [autoStart, setAutoStart] = useState(false);

  useEffect(() => {
    // Load existing settings on mount
    invoke<AppSettings>("get_settings")
      .then((settings) => {
        setUrl(settings.wallpaper_url);
        setTime(settings.update_time);
      })
      .catch(console.error);

    // Check autostart status
    isEnabled().then(setAutoStart).catch(console.error);
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

      // Handle Autostart toggle
      if (autoStart) {
        await enable();
      } else {
        await disable();
      }

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
      <details className="how-to-accordion">
        <summary>How to use this?</summary>
        <p className="description">
          Paste your wallpaper link and set the time you want the wallpaper to update daily. In case it doesn't update (e.g., if your PC was off or the network was unavailable at the scheduled time), click the <strong>^</strong> arrow in the bottom right Windows System Tray, right-click the app icon, and click <strong>Update Now</strong>, or simply <strong>Left-Click</strong> the icon to update it instantly!
        </p>
      </details>

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

      <div style={{ display: 'flex', alignItems: 'center', gap: '0.8rem', marginBottom: '1.5rem', justifyContent: 'flex-start' }}>
        <input
          type="checkbox"
          id="autostart"
          checked={autoStart}
          onChange={(e) => setAutoStart(e.target.checked)}
          style={{ width: 'auto', margin: 0 }}
        />
        <label htmlFor="autostart" style={{ margin: 0, textTransform: 'none', letterSpacing: 'normal', cursor: 'pointer', color: '#000000', fontSize: '0.9rem' }}>
          Start app silently when Windows starts
        </label>
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
