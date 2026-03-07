use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    AppHandle, Manager, Runtime, WindowEvent,
};
use chrono::{Local, Timelike};
use directories::ProjectDirs;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppSettings {
    pub wallpaper_url: String,
    pub update_time: String, // format "HH:mm"
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            wallpaper_url: "".into(),
            update_time: "00:00".into(),
        }
    }
}

pub struct AppState {
    pub settings: Mutex<AppSettings>,
}

fn get_config_path() -> std::path::PathBuf {
    let proj_dirs = ProjectDirs::from("com", "", "thecalendarwallpaper").unwrap();
    let config_dir = proj_dirs.config_dir();
    fs::create_dir_all(config_dir).unwrap();
    config_dir.join("settings.json")
}

#[tauri::command]
fn get_settings(state: tauri::State<AppState>) -> AppSettings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
fn save_settings(state: tauri::State<AppState>, settings: AppSettings) -> Result<(), String> {
    let path = get_config_path();
    let content = serde_json::to_string(&settings).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
    *state.settings.lock().unwrap() = settings;
    Ok(())
}

#[tauri::command]
async fn set_wallpaper(url: String) -> Result<String, String> {
    if url.is_empty() {
        return Err("URL is empty".into());
    }

    // Download image to temp file
    let response = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    
    let cache_dir = std::env::temp_dir();
    let file_path = cache_dir.join("current_wallpaper.png");
    fs::write(&file_path, bytes).map_err(|e| e.to_string())?;

    // Set wallpaper using crate
    wallpaper::set_from_path(file_path.to_str().unwrap()).map_err(|e| e.to_string())?;
    
    Ok("Wallpaper updated successfully".into())
}

fn start_scheduler<R: Runtime>(app: AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        loop {
            let (url, update_time) = {
                let state = app.state::<AppState>();
                let settings = state.settings.lock().unwrap();
                (settings.wallpaper_url.clone(), settings.update_time.clone())
            };
            
            if !url.is_empty() {
                let now = Local::now();
                let current_time = format!("{:02}:{:02}", now.hour(), now.minute());
                
                if current_time == update_time {
                    println!("Scheduled update triggered at {}", current_time);
                    let _ = set_wallpaper(url).await;
                    // Sleep for a minute to avoid multiple triggers
                    tokio::time::sleep(Duration::from_secs(61)).await;
                    continue;
                }
            }
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load initial settings
    let initial_settings = if let Ok(content) = fs::read_to_string(get_config_path()) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        AppSettings::default()
    };

    tauri::Builder::default()
        .manage(AppState {
            settings: Mutex::new(initial_settings),
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                let _ = window.hide();
                api.prevent_close();
            }
            _ => {}
        })
        .setup(|app| {
            let args: Vec<String> = std::env::args().collect();
            if !args.contains(&"--hidden".to_string()) {
                if let Some(main_window) = app.get_webview_window("main") {
                    let _ = main_window.show();
                    let _ = main_window.set_focus();
                }
            }

            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let update_i = MenuItem::with_id(app, "update", "Update Now", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show App", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &update_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event {
                        let app = tray.app_handle();
                        let state = app.state::<AppState>();
                        let url = state.settings.lock().unwrap().wallpaper_url.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = set_wallpaper(url).await;
                        });
                    }
                })
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "update" => {
                        let state = app.state::<AppState>();
                        let url = state.settings.lock().unwrap().wallpaper_url.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = set_wallpaper(url).await;
                        });
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => (),
                })
                .build(app)?;

            // Start the background scheduler
            start_scheduler(app.handle().clone());

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--hidden"])))
        .invoke_handler(tauri::generate_handler![get_settings, save_settings, set_wallpaper])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
