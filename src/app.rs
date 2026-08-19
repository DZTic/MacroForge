use crate::events::EngineEvent;
use crate::macro_core::{self, ActionType, MacroAction, MACRO_STATE};
use eframe::egui;
use std::sync::mpsc::Receiver;

pub struct MacroForgeApp {
    rx_events: Receiver<EngineEvent>,
    is_recording: bool,
    is_playing: bool,
    loop_playback: bool,
    actions_cache: Vec<MacroAction>,
    status_message: String,
}

impl MacroForgeApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, rx_events: Receiver<EngineEvent>) -> Self {
        let initial_loop = macro_core::get_loop_playback();
        let initial_actions = {
            let state = MACRO_STATE.lock().unwrap();
            state.actions.clone()
        };

        Self {
            rx_events,
            is_recording: false,
            is_playing: false,
            loop_playback: initial_loop,
            actions_cache: initial_actions,
            status_message: "Prêt. Appuyez sur F8 pour enregistrer.".to_string(),
        }
    }

    fn update_from_events(&mut self) {
        while let Ok(event) = self.rx_events.try_recv() {
            match event {
                EngineEvent::RecordingStateChanged(rec) => {
                    self.is_recording = rec;
                    if rec {
                        self.status_message = "🔴 Enregistrement en cours (F9 pour arrêter)...".to_string();
                    } else {
                        self.refresh_actions();
                        self.status_message = format!(
                            "⏹️ Enregistrement arrêté. {} actions enregistrées.",
                            self.actions_cache.len()
                        );
                    }
                }
                EngineEvent::PlaybackStateChanged(play) => {
                    self.is_playing = play;
                    if play {
                        self.status_message = "▶️ Lecture en cours (F4 pour arrêt d'urgence)...".to_string();
                    } else {
                        self.status_message = "⏹️ Lecture terminée.".to_string();
                    }
                }
                EngineEvent::PlaybackAction(action) => {
                    self.status_message = format!(
                        "▶️ Exécution [{}/{}]: {} ({})",
                        action.index, action.total, action.action_type, action.detail
                    );
                }
            }
        }
    }

    fn refresh_actions(&mut self) {
        let state = MACRO_STATE.lock().unwrap();
        self.actions_cache = state.actions.clone();
    }
}

impl eframe::App for MacroForgeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_from_events();

        egui::TopBottomPanel::top("header_panel").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading("🛠️ MacroForge (Natif Windows)");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.is_recording {
                        ui.colored_label(egui::Color32::RED, "🔴 ENREGISTREMENT ACTIF");
                    } else if self.is_playing {
                        ui.colored_label(egui::Color32::GREEN, "▶️ LECTURE EN COURS");
                    } else {
                        ui.colored_label(egui::Color32::GRAY, "⚪ INACTIF");
                    }
                });
            });
            ui.add_space(4.0);
        });

        egui::TopBottomPanel::bottom("footer_panel").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                // Bouton Enregistrer / Arrêter
                if !self.is_recording {
                    if ui.button("🔴 Enregistrer (F8)").clicked() {
                        macro_core::start_recording();
                    }
                } else if ui.button("⏹️ Arrêter Enregistrement (F9)").clicked() {
                    macro_core::stop_recording();
                }

                // Bouton Jouer / Arrêter
                if !self.is_playing {
                    if ui.button("▶️ Jouer (F4 stop)").clicked() {
                        macro_core::play_macro();
                    }
                } else if ui.button("⏹️ Arrêt d'urgence (F4)").clicked() {
                    macro_core::stop_playback();
                }

                ui.separator();

                // Option Boucle
                if ui.checkbox(&mut self.loop_playback, "🔁 Boucler").changed() {
                    macro_core::set_loop_playback(self.loop_playback);
                }

                ui.separator();

                // Sauvegarder
                if ui.button("💾 Sauvegarder").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("MacroForge Profile", &["mforge", "json"])
                        .save_file()
                    {
                        if let Some(path_str) = path.to_str() {
                            if let Err(e) = macro_core::save_macro_to_file(path_str) {
                                self.status_message = format!("❌ Erreur sauvegarde: {}", e);
                            } else {
                                self.status_message = "✅ Macro sauvegardée avec succès!".to_string();
                            }
                        }
                    }
                }

                // Ouvrir
                if ui.button("📂 Ouvrir").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("MacroForge Profile", &["mforge", "json"])
                        .pick_file()
                    {
                        if let Some(path_str) = path.to_str() {
                            match macro_core::load_macro_from_file(path_str) {
                                Ok(count) => {
                                    self.refresh_actions();
                                    self.status_message = format!("✅ {} actions chargées.", count);
                                }
                                Err(e) => {
                                    self.status_message = format!("❌ Erreur chargement: {}", e);
                                }
                            }
                        }
                    }
                }

                // Vider
                if ui.button("🗑️ Vider").clicked() {
                    let mut state = MACRO_STATE.lock().unwrap();
                    state.actions.clear();
                    self.actions_cache.clear();
                    self.status_message = "Actions vidées.".to_string();
                }
            });

            ui.add_space(4.0);
            ui.separator();
            ui.label(format!("ℹ️ {}", self.status_message));
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("📋 Actions de la Macro");
                ui.label(format!("({} actions)", self.actions_cache.len()));
                if ui.button("🔄 Rafraîchir").clicked() {
                    self.refresh_actions();
                }
            });

            ui.separator();

            if self.actions_cache.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label("Aucune action enregistrée.");
                    ui.label("Appuyez sur F8 pour lancer l'enregistrement.");
                });
            } else {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for (idx, action) in self.actions_cache.iter().enumerate() {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(format!("#{:<4}", idx + 1));
                                    match &action.action_type {
                                        ActionType::KeyPress(name, vk, _) => {
                                            ui.colored_label(egui::Color32::LIGHT_BLUE, "⌨️ Touche Pressée");
                                            ui.monospace(format!("{} (VK: {:#X})", name, vk));
                                        }
                                        ActionType::KeyRelease(name, vk, _) => {
                                            ui.colored_label(egui::Color32::GRAY, "⌨️ Touche Relâchée");
                                            ui.monospace(format!("{} (VK: {:#X})", name, vk));
                                        }
                                        ActionType::MouseMove(x, y) => {
                                            ui.colored_label(egui::Color32::KHAKI, "🖱️ Déplacement");
                                            ui.monospace(format!("X: {:.0}, Y: {:.0}", x, y));
                                        }
                                        ActionType::MouseMoveRelative(dx, dy) => {
                                            ui.colored_label(egui::Color32::GOLD, "🖱️ Déplacement Relatif");
                                            ui.monospace(format!("dX: {}, dY: {}", dx, dy));
                                        }
                                        ActionType::MousePress(btn, x, y) => {
                                            ui.colored_label(egui::Color32::LIGHT_GREEN, "🖱️ Clic Pressé");
                                            ui.monospace(format!("Bouton {} à ({:.0}, {:.0})", btn, x, y));
                                        }
                                        ActionType::MouseRelease(btn, x, y) => {
                                            ui.colored_label(egui::Color32::GRAY, "🖱️ Clic Relâché");
                                            ui.monospace(format!("Bouton {} à ({:.0}, {:.0})", btn, x, y));
                                        }
                                        ActionType::Scroll(dx, dy) => {
                                            ui.colored_label(egui::Color32::LIGHT_YELLOW, "📜 Molette");
                                            ui.monospace(format!("dX: {:.1}, dY: {:.1}", dx, dy));
                                        }
                                        ActionType::Wait(ms) => {
                                            ui.colored_label(egui::Color32::ORANGE, "⏱️ Pause");
                                            ui.monospace(format!("{} ms", ms));
                                        }
                                        ActionType::WaitImage(path, timeout) => {
                                            ui.colored_label(
                                                egui::Color32::from_rgb(217, 70, 239),
                                                "🖼️ Attente Image",
                                            );
                                            ui.monospace(format!("{} (timeout: {}ms)", path, timeout));
                                        }
                                    }

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(format!("+{} ms", action.delay_ms));
                                    });
                                });
                            });
                        }
                    });
            }
        });

        // Demander un repaint régulier si on est en train d'enregistrer ou de jouer pour fluidité
        if self.is_recording || self.is_playing {
            ctx.request_repaint_after(std::time::Duration::from_millis(33));
        }
    }
}
