pub mod macro_core;

use std::thread;

#[tauri::command]
fn start_macro_recording() -> String {
    macro_core::start_recording();
    "Recording started".to_string()
}

#[tauri::command]
fn stop_macro_recording() -> String {
    let count = macro_core::stop_recording();
    format!("Recording stopped. {} actions recorded.", count)
}

#[tauri::command]
fn get_macro_actions() -> Vec<macro_core::MacroAction> {
    let state = macro_core::MACRO_STATE.lock().unwrap();
    state.actions.clone()
}

#[tauri::command]
fn play_macro_command() -> String {
    macro_core::play_macro();
    "Playback started".to_string()
}

#[tauri::command]
fn stop_macro_playback() -> String {
    macro_core::stop_playback();
    "Playback stopped".to_string()
}

#[tauri::command]
fn set_macro_actions(actions: Vec<macro_core::MacroAction>) {
    let mut state = macro_core::MACRO_STATE.lock().unwrap();
    state.actions = actions;
}

#[tauri::command]
fn set_loop_playback(looping: bool) {
    macro_core::set_loop_playback(looping);
}
#[tauri::command]
fn get_loop_playback() -> bool {
    macro_core::get_loop_playback()
}

#[tauri::command]
fn set_stop_image(path: Option<String>, timeout: u64) {
    macro_core::set_stop_image(path, timeout);
}

#[tauri::command]
fn get_stop_image() -> (Option<String>, u64) {
    macro_core::get_stop_image()
}

#[tauri::command]
fn save_macro(path: String) -> Result<String, String> {
    let state = macro_core::MACRO_STATE.lock().unwrap();
    let json = serde_json::to_string(&state.actions).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok("Macro saved successfully".to_string())
}

#[tauri::command]
fn load_macro(path: String) -> Result<String, String> {
    let data = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let actions: Vec<macro_core::MacroAction> =
        serde_json::from_str(&data).map_err(|e| e.to_string())?;
    let mut state = macro_core::MACRO_STATE.lock().unwrap();
    state.actions = actions;
    Ok("Macro loaded successfully".to_string())
}

#[tauri::command]
fn close_toolbar(app: tauri::AppHandle) {
    use tauri::Manager;
    let main_window = app.get_webview_window("main");
    let main_visible = main_window.map(|w| w.is_visible().unwrap_or(false)).unwrap_or(false);

    if !main_visible {
        // Si la fenêtre principale est déjà cachée/fermée, on quitte l'app
        app.exit(0);
    } else if let Some(window) = app.get_webview_window("toolbar") {
        let _ = window.hide();
    }
}

#[tauri::command]
fn open_toolbar(app: tauri::AppHandle) {
    use tauri::Manager;
    // On affiche la toolbar
    if let Some(window) = app.get_webview_window("toolbar") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
fn show_main_window(app: tauri::AppHandle) {
    use tauri::Manager;
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    use tauri::Manager;
                    let app = window.app_handle();
                    let toolbar = app.get_webview_window("toolbar");
                    let toolbar_visible = toolbar.map(|w| w.is_visible().unwrap_or(false)).unwrap_or(false);

                    if !toolbar_visible {
                        // Si la toolbar est déjà cachée/fermée, on quitte l'app au lieu de juste cacher
                        app.exit(0);
                    } else {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                }
            }
        })
        .setup(|app| {
            macro_core::set_app_handle(app.handle().clone());

            #[cfg(windows)]
            macro_core::start_focus_tracker();

            thread::spawn(|| {
                if let Err(error) = rdev::listen(macro_core::handle_rdev_event) {
                    println!("Error listening to rdev: {:?}", error);
                }
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            start_macro_recording,
            stop_macro_recording,
            get_macro_actions,
            play_macro_command,
            stop_macro_playback,
            set_macro_actions,
            save_macro,
            load_macro,
            close_toolbar,
            open_toolbar,
            show_main_window,
            set_loop_playback,
            get_loop_playback,
            set_stop_image,
            get_stop_image
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
