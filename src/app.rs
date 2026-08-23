use crate::events::EngineEvent;
use crate::macro_core::{self, MacroAction, MACRO_STATE};
use crate::ui::theme::{self, colors};
use crate::ui::widgets::{
    ActionCard, ButtonVariant, CustomToggleSwitch, GlassButton, StatusBadge, StatusKind,
};
use eframe::egui::{self, Color32, Frame, Margin, Rounding, Stroke};
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
    pub fn new(cc: &eframe::CreationContext<'_>, rx_events: Receiver<EngineEvent>) -> Self {
        // Appliquer le thème Glassmorphism et la typographie au contexte egui
        theme::apply_theme(&cc.egui_ctx);

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
            status_message: "Prêt. Appuyez sur F8 pour démarrer l'enregistrement.".to_string(),
        }
    }

    fn update_from_events(&mut self) {
        while let Ok(event) = self.rx_events.try_recv() {
            match event {
                EngineEvent::RecordingStateChanged(rec) => {
                    self.is_recording = rec;
                    if rec {
                        self.status_message =
                            "🔴 Enregistrement en cours (F9 pour arrêter)...".to_string();
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
                        self.status_message =
                            "▶️ Lecture en cours (F4 pour arrêt d'urgence)...".to_string();
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

        // 1. En-tête supérieur (Header Glassmorphism)
        egui::TopBottomPanel::top("header_panel")
            .frame(theme::header_frame())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("⚡ MacroForge")
                            .heading()
                            .color(colors::TEXT_PRIMARY)
                            .strong(),
                    );

                    // Badge de version
                    let version_badge = Frame::none()
                        .fill(Color32::from_rgba_premultiplied(59, 130, 246, 30))
                        .stroke(Stroke::new(1.0, Color32::from_rgba_premultiplied(59, 130, 246, 80)))
                        .rounding(Rounding::same(4.0))
                        .inner_margin(Margin::symmetric(6.0, 2.0));

                    version_badge.show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("v0.2.0 Native")
                                .monospace()
                                .color(colors::ACCENT_PRIMARY_HOVER)
                                .size(11.0),
                        );
                    });

                    // Badge d'état aligné à droite
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let status_badge = if self.is_recording {
                            StatusBadge::new(StatusKind::Recording)
                        } else if self.is_playing {
                            StatusBadge::new(StatusKind::Playing)
                        } else {
                            StatusBadge::new(StatusKind::Idle)
                        };

                        ui.add(status_badge).on_hover_text(
                            "Indicateur temps réel de l'état du moteur de macro (F8: Rec, F9: Stop, F4: Stop Playback)",
                        );
                    });
                });
            });

        // 2. Barre d'actions inférieure (Footer Glassmorphism)
        egui::TopBottomPanel::bottom("footer_panel")
            .frame(theme::footer_frame())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Bouton Enregistrer / Arrêter
                    if !self.is_recording {
                        let btn = GlassButton::new("Enregistrer")
                            .icon("🔴")
                            .shortcut("F8")
                            .variant(ButtonVariant::Danger);
                        if ui
                            .add(btn)
                            .on_hover_text("Démarrer l'enregistrement global des entrées (F8)")
                            .clicked()
                        {
                            macro_core::start_recording();
                        }
                    } else {
                        let btn = GlassButton::new("Arrêter")
                            .icon("⏹️")
                            .shortcut("F9")
                            .variant(ButtonVariant::Secondary);
                        if ui
                            .add(btn)
                            .on_hover_text("Arrêter l'enregistrement en cours (F9)")
                            .clicked()
                        {
                            macro_core::stop_recording();
                        }
                    }

                    // Bouton Jouer / Arrêter
                    if !self.is_playing {
                        let btn = GlassButton::new("Rejouer")
                            .icon("▶️")
                            .shortcut("F4 stop")
                            .variant(ButtonVariant::Success);
                        if ui
                            .add(btn)
                            .on_hover_text("Exécuter la séquence de macro enregistrée")
                            .clicked()
                        {
                            macro_core::play_macro();
                        }
                    } else {
                        let btn = GlassButton::new("Arrêt Urgence")
                            .icon("⏹️")
                            .shortcut("F4")
                            .variant(ButtonVariant::Warning);
                        if ui
                            .add(btn)
                            .on_hover_text("Arrêter immédiatement la relecture (F4)")
                            .clicked()
                        {
                            macro_core::stop_playback();
                        }
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Switch Mode Boucle
                    let toggle =
                        CustomToggleSwitch::new(&mut self.loop_playback).label("🔁 Mode Boucle");
                    if ui
                        .add(toggle)
                        .on_hover_text("Répéter la macro indéfiniment jusqu'à l'arrêt d'urgence F4")
                        .changed()
                    {
                        macro_core::set_loop_playback(self.loop_playback);
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Sauvegarder
                    let save_btn = GlassButton::new("Sauvegarder")
                        .icon("💾")
                        .variant(ButtonVariant::Secondary);
                    if ui
                        .add(save_btn)
                        .on_hover_text("Exporter le profil de macro (.mforge)")
                        .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("MacroForge Profile", &["mforge", "json"])
                            .save_file()
                        {
                            if let Some(path_str) = path.to_str() {
                                if let Err(e) = macro_core::save_macro_to_file(path_str) {
                                    self.status_message = format!("❌ Erreur sauvegarde: {}", e);
                                } else {
                                    self.status_message =
                                        "✅ Profil sauvegardé avec succès!".to_string();
                                }
                            }
                        }
                    }

                    // Ouvrir
                    let open_btn = GlassButton::new("Ouvrir")
                        .icon("📂")
                        .variant(ButtonVariant::Secondary);
                    if ui
                        .add(open_btn)
                        .on_hover_text("Importer un profil de macro (.mforge)")
                        .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("MacroForge Profile", &["mforge", "json"])
                            .pick_file()
                        {
                            if let Some(path_str) = path.to_str() {
                                match macro_core::load_macro_from_file(path_str) {
                                    Ok(count) => {
                                        self.refresh_actions();
                                        self.status_message =
                                            format!("✅ {} actions chargées.", count);
                                    }
                                    Err(e) => {
                                        self.status_message =
                                            format!("❌ Erreur chargement: {}", e);
                                    }
                                }
                            }
                        }
                    }

                    // Vider
                    let clear_btn = GlassButton::new("Vider")
                        .icon("🗑️")
                        .variant(ButtonVariant::Ghost);
                    if ui
                        .add(clear_btn)
                        .on_hover_text("Effacer toutes les actions enregistrées")
                        .clicked()
                    {
                        let mut state = MACRO_STATE.lock().unwrap();
                        state.actions.clear();
                        self.actions_cache.clear();
                        self.status_message = "Toutes les actions ont été effacées.".to_string();
                    }
                });

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(2.0);

                // Ligne de statut informative
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("ℹ️")
                            .color(colors::ACCENT_PRIMARY_HOVER)
                            .size(12.0),
                    );
                    ui.label(
                        egui::RichText::new(&self.status_message)
                            .color(colors::TEXT_SECONDARY)
                            .size(12.5),
                    );
                });
            });

        // 3. Panneau central (Timeline des Actions)
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(4.0);

            // En-tête de section Timeline
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("📋 Séquence d'Actions")
                        .heading()
                        .color(colors::TEXT_PRIMARY)
                        .strong(),
                );

                // Badge de compteur d'actions
                let count_badge = Frame::none()
                    .fill(colors::BG_CARD)
                    .stroke(Stroke::new(1.0, colors::BORDER_SUBTLE))
                    .rounding(Rounding::same(12.0))
                    .inner_margin(Margin::symmetric(8.0, 2.0));

                count_badge.show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(format!("{} action(s)", self.actions_cache.len()))
                            .size(11.5)
                            .color(colors::TEXT_SECONDARY),
                    );
                });

                // Bouton Rafraîchir aligné à droite
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let refresh_btn = GlassButton::new("Rafraîchir")
                        .icon("🔄")
                        .variant(ButtonVariant::Ghost);
                    if ui.add(refresh_btn).on_hover_text("Synchroniser la liste avec l'état interne").clicked() {
                        self.refresh_actions();
                    }
                });
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            if self.actions_cache.is_empty() {
                // État vide élégant (Glassmorphism Empty State)
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);

                    let empty_card = theme::glass_card_frame();
                    empty_card.show(ui, |ui| {
                        ui.set_max_width(450.0);
                        ui.vertical_centered(|ui| {
                            ui.add_space(10.0);
                            ui.label(
                                egui::RichText::new("⚡")
                                    .size(36.0)
                                    .color(colors::ACCENT_PRIMARY_HOVER),
                            );
                            ui.add_space(6.0);
                            ui.label(
                                egui::RichText::new("Aucune action dans la macro")
                                    .strong()
                                    .size(16.0)
                                    .color(colors::TEXT_PRIMARY),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(
                                    "Appuyez sur la touche F8 ou cliquez sur « Enregistrer » ci-dessous pour capturer vos actions clavier et souris.",
                                )
                                .color(colors::TEXT_SECONDARY)
                                .size(13.0),
                            );
                            ui.add_space(10.0);
                        });
                    });
                });
            } else {
                // Liste scrollable des ActionCard
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(0.0, 4.0);
                        for (idx, action) in self.actions_cache.iter().enumerate() {
                            let card = ActionCard::new(idx, action);
                            ui.add(card);
                        }
                    });
            }
        });

        // Demander un repaint régulier si on enregistre ou joue pour fluidité de l'UI
        if self.is_recording || self.is_playing {
            ctx.request_repaint_after(std::time::Duration::from_millis(33));
        }
    }
}
