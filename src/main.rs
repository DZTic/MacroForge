// Prevents console window in release mode on Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod app;
pub mod events;
pub mod macro_core;
pub mod ui;

use app::MacroForgeApp;
use eframe::egui;
use std::sync::mpsc;
use std::thread;

fn main() -> eframe::Result<()> {
    // Logging gated : niveau par defaut debug en dev, warn en release,
    // surchargeable sans recompilation via la variable RUST_LOG (issue #30).
    #[cfg(debug_assertions)]
    let default_level = "debug";
    #[cfg(not(debug_assertions))]
    let default_level = "warn";
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default_level))
        .format_timestamp_millis()
        .init();

    // 1. Initialiser le canal d'événements du moteur vers l'UI
    let (tx_events, rx_events) = mpsc::channel();
    macro_core::set_event_sender(tx_events);

    // 2. Démarrer le focus tracker Windows
    #[cfg(windows)]
    macro_core::start_focus_tracker();

    // 3. Démarrer l'écouteur global clavier/souris rdev (F8 = Rec, F9 = Stop, F4 = Stop Playback)
    thread::spawn(|| {
        if let Err(error) = rdev::listen(macro_core::handle_rdev_event) {
            log::error!("Erreur lors de l'écoute rdev: {:?}", error);
        }
    });

    // 3. bis Démarrer l'écouteur global RegisterHotKey Win32 pour garantir la capture globale des touches F8, F9, F4
    #[cfg(windows)]
    macro_core::start_global_hotkey_listener();

    // 4. Configurer les options de fenêtre native
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("MacroForge (Full Natif Windows)")
            .with_inner_size([900.0, 650.0])
            .with_min_inner_size([600.0, 450.0]),
        ..Default::default()
    };

    // 5. Lancer l'application egui / eframe
    eframe::run_native(
        "MacroForge",
        native_options,
        Box::new(|cc| Ok(Box::new(MacroForgeApp::new(cc, rx_events)))),
    )
}
