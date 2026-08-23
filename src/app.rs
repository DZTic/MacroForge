use crate::events::EngineEvent;
use crate::macro_core::{self, ActionType, MacroAction, MACRO_STATE};
use crate::ui::dialogs::{
    ActionEditorModal, ActionModalTab, ActionModalTarget, StopImageConfigModal,
};
use crate::ui::i18n::Language;
use crate::ui::theme::{self, colors};
use crate::ui::widgets::{
    ActionCard, ActionCardEvent, ButtonVariant, CustomToggleSwitch, FilterBar, GlassButton,
    StatusBadge, StatusKind,
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

    // Internationalisation
    lang: Language,

    // Filtres & Recherche
    hide_mouse_moves: bool,
    search_query: String,
    jump_index: usize,
    scroll_target_index: Option<usize>,

    // Sélection
    selected_action_index: Option<usize>,

    // Modales & dialogues
    action_modal: ActionEditorModal,
    stop_image_modal: StopImageConfigModal,
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

        let lang = Language::Fr;
        let ready_msg = lang.ready_status().to_string();

        Self {
            rx_events,
            is_recording: false,
            is_playing: false,
            loop_playback: initial_loop,
            actions_cache: initial_actions,
            status_message: ready_msg,

            lang,
            hide_mouse_moves: false,
            search_query: String::new(),
            jump_index: 1,
            scroll_target_index: None,
            selected_action_index: None,

            action_modal: ActionEditorModal::new(),
            stop_image_modal: StopImageConfigModal::new(),
        }
    }

    fn update_from_events(&mut self) {
        while let Ok(event) = self.rx_events.try_recv() {
            match event {
                EngineEvent::RecordingStateChanged(rec) => {
                    self.is_recording = rec;
                    if rec {
                        self.status_message = match self.lang {
                            Language::Fr => {
                                "🔴 Enregistrement en cours (F9 pour arrêter)...".to_string()
                            }
                            Language::En => "🔴 Recording in progress (F9 to stop)...".to_string(),
                        };
                    } else {
                        self.refresh_actions();
                        self.status_message = match self.lang {
                            Language::Fr => format!(
                                "⏹️ Enregistrement arrêté. {} actions enregistrées.",
                                self.actions_cache.len()
                            ),
                            Language::En => format!(
                                "⏹️ Recording stopped. {} actions recorded.",
                                self.actions_cache.len()
                            ),
                        };
                    }
                }
                EngineEvent::PlaybackStateChanged(play) => {
                    self.is_playing = play;
                    if play {
                        self.status_message = match self.lang {
                            Language::Fr => {
                                "▶️ Lecture en cours (F4 pour arrêt d'urgence)...".to_string()
                            }
                            Language::En => {
                                "▶️ Playback in progress (F4 for emergency stop)...".to_string()
                            }
                        };
                    } else {
                        self.status_message = match self.lang {
                            Language::Fr => "⏹️ Lecture terminée.".to_string(),
                            Language::En => "⏹️ Playback finished.".to_string(),
                        };
                    }
                }
                EngineEvent::PlaybackAction(action) => {
                    self.status_message = format!(
                        "▶️ [{}/{}] {} ({})",
                        action.index, action.total, action.action_type, action.detail
                    );
                    self.selected_action_index = Some(action.index.saturating_sub(1));
                }
            }
        }
    }

    fn refresh_actions(&mut self) {
        self.actions_cache = macro_core::get_actions();
    }

    fn matches_filter(&self, action: &MacroAction) -> bool {
        // 1. Filtre mouvement souris
        if self.hide_mouse_moves {
            match &action.action_type {
                ActionType::MouseMove(_, _) | ActionType::MouseMoveRelative(_, _) => return false,
                _ => {}
            }
        }

        // 2. Filtre recherche textuelle
        if !self.search_query.trim().is_empty() {
            let q = self.search_query.trim().to_lowercase();
            let matches = match &action.action_type {
                ActionType::KeyPress(name, vk, _) => {
                    name.to_lowercase().contains(&q)
                        || format!("{:x}", vk).contains(&q)
                        || "keypress".contains(&q)
                }
                ActionType::KeyRelease(name, vk, _) => {
                    name.to_lowercase().contains(&q)
                        || format!("{:x}", vk).contains(&q)
                        || "keyrelease".contains(&q)
                }
                ActionType::MouseMove(x, y) => {
                    format!("{} {}", x, y).contains(&q)
                        || "move".contains(&q)
                        || "souris".contains(&q)
                }
                ActionType::MouseMoveRelative(dx, dy) => {
                    format!("{} {}", dx, dy).contains(&q)
                        || "rel".contains(&q)
                        || "relative".contains(&q)
                }
                ActionType::MousePress(btn, _, _) => {
                    format!("btn {}", btn).contains(&q)
                        || "click".contains(&q)
                        || "clic".contains(&q)
                }
                ActionType::MouseRelease(btn, _, _) => {
                    format!("btn {}", btn).contains(&q) || "release".contains(&q)
                }
                ActionType::Scroll(dx, dy) => {
                    format!("{} {}", dx, dy).contains(&q)
                        || "scroll".contains(&q)
                        || "molette".contains(&q)
                }
                ActionType::Wait(ms) => {
                    format!("{}", ms).contains(&q) || "wait".contains(&q) || "pause".contains(&q)
                }
                ActionType::WaitImage(path, timeout) => {
                    path.to_lowercase().contains(&q)
                        || format!("{}", timeout).contains(&q)
                        || "image".contains(&q)
                }
            };
            if !matches {
                return false;
            }
        }

        true
    }
}

impl eframe::App for MacroForgeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_from_events();

        // 1. Modales d'ajout/édition et de configuration d'arrêt
        if let Some((target, action)) = self.action_modal.show(ctx, self.lang) {
            match target {
                ActionModalTarget::New => {
                    macro_core::add_action(action);
                    self.refresh_actions();
                    self.status_message = match self.lang {
                        Language::Fr => "✅ Action ajoutée avec succès.".to_string(),
                        Language::En => "✅ Action added successfully.".to_string(),
                    };
                }
                ActionModalTarget::Edit(idx) => {
                    macro_core::update_action(idx, action);
                    self.refresh_actions();
                    self.status_message = match self.lang {
                        Language::Fr => format!("✅ Action #{} modifiée.", idx + 1),
                        Language::En => format!("✅ Action #{} updated.", idx + 1),
                    };
                }
            }
        }

        if self.stop_image_modal.show(ctx, self.lang) {
            self.status_message = match self.lang {
                Language::Fr => {
                    "✅ Configuration de l'image d'arrêt d'urgence enregistrée.".to_string()
                }
                Language::En => "✅ Emergency stop image configuration saved.".to_string(),
            };
        }

        // 2. En-tête supérieur (Header & Quick Actions)
        egui::TopBottomPanel::top("header_panel")
            .frame(theme::header_frame())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Logo et Titre
                    ui.label(
                        egui::RichText::new(self.lang.app_title())
                            .heading()
                            .color(colors::TEXT_PRIMARY)
                            .strong(),
                    );

                    // Badge de version
                    let version_badge = Frame::none()
                        .fill(Color32::from_rgba_premultiplied(59, 130, 246, 30))
                        .stroke(Stroke::new(
                            1.0_f32,
                            Color32::from_rgba_premultiplied(59, 130, 246, 80),
                        ))
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

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // Boutons d'ajout rapide d'action
                    let key_btn = GlassButton::new(self.lang.quick_add_key())
                        .variant(ButtonVariant::Secondary);
                    if ui.add(key_btn).clicked() {
                        self.action_modal.open_for_new(ActionModalTab::Keyboard);
                    }

                    let mouse_btn = GlassButton::new(self.lang.quick_add_mouse())
                        .variant(ButtonVariant::Secondary);
                    if ui.add(mouse_btn).clicked() {
                        self.action_modal.open_for_new(ActionModalTab::Mouse);
                    }

                    let wait_btn = GlassButton::new(self.lang.quick_add_wait())
                        .variant(ButtonVariant::Secondary);
                    if ui.add(wait_btn).clicked() {
                        self.action_modal.open_for_new(ActionModalTab::Wait);
                    }

                    let img_btn = GlassButton::new(self.lang.quick_add_image())
                        .variant(ButtonVariant::Secondary);
                    if ui.add(img_btn).clicked() {
                        self.action_modal.open_for_new(ActionModalTab::Image);
                    }

                    // Commandes alignées à droite
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Badge d'état dynamique
                        let status_badge = if self.is_recording {
                            StatusBadge::new(StatusKind::Recording)
                        } else if self.is_playing {
                            StatusBadge::new(StatusKind::Playing)
                        } else {
                            StatusBadge::new(StatusKind::Idle)
                        };
                        ui.add(status_badge).on_hover_text(
                            "Indicateur temps réel de l'état du moteur (F8: Rec, F9: Stop, F4: Stop Playback)",
                        );

                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(6.0);

                        // Sélecteur de langue interactif
                        let lang_btn = GlassButton::new(match self.lang {
                            Language::Fr => "🌐 FR",
                            Language::En => "🌐 EN",
                        })
                        .variant(ButtonVariant::Ghost);
                        if ui.add(lang_btn).on_hover_text("Changer de langue (FR / EN)").clicked() {
                            self.lang.toggle();
                        }

                        // Bouton Toolbar flottante
                        let toolbar_btn = GlassButton::new(self.lang.toolbar_window_btn())
                            .variant(ButtonVariant::Ghost);
                        if ui.add(toolbar_btn).on_hover_text("Ouvrir la Toolbar flottante compacte").clicked() {
                            self.status_message = match self.lang {
                                Language::Fr => "🗔 Toolbar flottante native initialisée.".to_string(),
                                Language::En => "🗔 Native floating toolbar initialized.".to_string(),
                            };
                        }
                    });
                });
            });

        // 3. Barre inférieure de contrôle global (Footer)
        egui::TopBottomPanel::bottom("footer_panel")
            .frame(theme::footer_frame())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Bouton Enregistrer / Arrêter
                    if !self.is_recording {
                        let btn = GlassButton::new(self.lang.record_btn())
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
                        let btn = GlassButton::new(self.lang.stop_btn())
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

                    // Bouton Jouer / Arrêt Urgence
                    if !self.is_playing {
                        let btn = GlassButton::new(self.lang.play_btn())
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
                        let btn = GlassButton::new(self.lang.emergency_stop_btn())
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
                    let toggle = CustomToggleSwitch::new(&mut self.loop_playback)
                        .label(self.lang.loop_mode_label());
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

                    // Bouton Configuration Image d'arrêt
                    let (stop_img, _) = macro_core::get_stop_image();
                    let has_stop_img = stop_img.is_some();
                    let stop_img_btn =
                        GlassButton::new(self.lang.stop_image_cfg_btn()).variant(if has_stop_img {
                            ButtonVariant::Primary
                        } else {
                            ButtonVariant::Secondary
                        });
                    if ui
                        .add(stop_img_btn)
                        .on_hover_text("Configurer l'image de détection d'arrêt d'urgence")
                        .clicked()
                    {
                        self.stop_image_modal.open();
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Sauvegarder profil .mforge
                    let save_btn = GlassButton::new(self.lang.save_profile())
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

                    // Ouvrir profil .mforge
                    let open_btn = GlassButton::new(self.lang.open_profile())
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
                    let clear_btn = GlassButton::new(self.lang.clear_actions())
                        .icon("🗑️")
                        .variant(ButtonVariant::Ghost);
                    if ui
                        .add(clear_btn)
                        .on_hover_text("Effacer toutes les actions enregistrées")
                        .clicked()
                    {
                        macro_core::clear_actions();
                        self.actions_cache.clear();
                        self.status_message = match self.lang {
                            Language::Fr => "Toutes les actions ont été effacées.".to_string(),
                            Language::En => "All actions have been cleared.".to_string(),
                        };
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

        // 4. Panneau central (Timeline & Actions Editor)
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(2.0);

            // En-tête de section Timeline & Actions
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(self.lang.timeline_heading())
                        .heading()
                        .color(colors::TEXT_PRIMARY)
                        .strong(),
                );

                // Bouton Rafraîchir
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let refresh_btn = GlassButton::new(self.lang.refresh_btn())
                        .icon("🔄")
                        .variant(ButtonVariant::Ghost);
                    if ui
                        .add(refresh_btn)
                        .on_hover_text("Synchroniser la liste avec le moteur interne")
                        .clicked()
                    {
                        self.refresh_actions();
                    }
                });
            });

            ui.add_space(4.0);

            // Filtrage des actions
            let total_count = self.actions_cache.len();
            let filtered_indices: Vec<usize> = self
                .actions_cache
                .iter()
                .enumerate()
                .filter_map(|(idx, act)| {
                    if self.matches_filter(act) {
                        Some(idx)
                    } else {
                        None
                    }
                })
                .collect();
            let visible_count = filtered_indices.len();

            let mut jump_triggered = false;
            let filter_bar = FilterBar::new(
                &mut self.hide_mouse_moves,
                &mut self.search_query,
                &mut self.jump_index,
                total_count,
                visible_count,
                self.lang,
                &mut jump_triggered,
            );
            ui.add(filter_bar);

            if jump_triggered && self.jump_index > 0 && self.jump_index <= total_count {
                self.scroll_target_index = Some(self.jump_index - 1);
                self.selected_action_index = Some(self.jump_index - 1);
            }

            ui.add_space(6.0);

            if self.actions_cache.is_empty() {
                // État vide élégant
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);

                    let empty_card = theme::glass_card_frame();
                    empty_card.show(ui, |ui| {
                        ui.set_max_width(480.0);
                        ui.vertical_centered(|ui| {
                            ui.add_space(12.0);
                            ui.label(
                                egui::RichText::new("⚡")
                                    .size(38.0)
                                    .color(colors::ACCENT_PRIMARY_HOVER),
                            );
                            ui.add_space(6.0);
                            ui.label(
                                egui::RichText::new(self.lang.empty_state_title())
                                    .strong()
                                    .size(16.0)
                                    .color(colors::TEXT_PRIMARY),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(self.lang.empty_state_desc())
                                    .color(colors::TEXT_SECONDARY)
                                    .size(13.0),
                            );
                            ui.add_space(12.0);
                        });
                    });
                });
            } else {
                // Liste scrollable des ActionCards avec support Drag & Drop
                let mut card_event_to_process = None;
                let is_unfiltered = !self.hide_mouse_moves && self.search_query.trim().is_empty();

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .id_salt("timeline_scroll_area")
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(0.0, 4.0);

                        if is_unfiltered {
                            for (idx, action) in self.actions_cache.iter_mut().enumerate() {
                                let card = ActionCard::new(idx, action)
                                    .selected(self.selected_action_index == Some(idx))
                                    .lang(self.lang)
                                    .bounds(idx == 0, idx == total_count - 1);
                                let (resp, ev) = card.show(ui);
                                if let Some(target) = self.scroll_target_index {
                                    if target == idx {
                                        resp.scroll_to_me(Some(egui::Align::Center));
                                    }
                                }
                                if let Some(e) = ev {
                                    card_event_to_process = Some(e);
                                }
                            }
                        } else {
                            // Affichage de la vue filtrée
                            for &original_idx in &filtered_indices {
                                if let Some(action) = self.actions_cache.get(original_idx) {
                                    let is_selected =
                                        self.selected_action_index == Some(original_idx);
                                    let card = ActionCard::new(original_idx, action)
                                        .selected(is_selected)
                                        .lang(self.lang)
                                        .bounds(original_idx == 0, original_idx == total_count - 1);

                                    let (resp, ev) = card.show(ui);
                                    if let Some(target) = self.scroll_target_index {
                                        if target == original_idx {
                                            resp.scroll_to_me(Some(egui::Align::Center));
                                        }
                                    }
                                    if let Some(e) = ev {
                                        card_event_to_process = Some(e);
                                    }
                                }
                            }
                        }
                    });

                // Réinitialiser le curseur de défilement ciblé
                self.scroll_target_index = None;

                // Traiter les événements déclenchés par les cartes
                if let Some(event) = card_event_to_process {
                    match event {
                        ActionCardEvent::Edit(idx) => {
                            if let Some(action) = self.actions_cache.get(idx) {
                                self.action_modal.open_for_edit(idx, action);
                            }
                        }
                        ActionCardEvent::Duplicate(idx) => {
                            macro_core::duplicate_action(idx);
                            self.refresh_actions();
                            self.status_message = match self.lang {
                                Language::Fr => format!("📋 Action #{} dupliquée.", idx + 1),
                                Language::En => format!("📋 Action #{} duplicated.", idx + 1),
                            };
                        }
                        ActionCardEvent::Delete(idx) => {
                            macro_core::delete_action(idx);
                            self.refresh_actions();
                            self.status_message = match self.lang {
                                Language::Fr => format!("🗑️ Action #{} supprimée.", idx + 1),
                                Language::En => format!("🗑️ Action #{} deleted.", idx + 1),
                            };
                        }
                        ActionCardEvent::MoveUp(idx) => {
                            if idx > 0 {
                                macro_core::move_action(idx, idx - 1);
                                self.refresh_actions();
                                self.selected_action_index = Some(idx - 1);
                            }
                        }
                        ActionCardEvent::MoveDown(idx) => {
                            if idx + 1 < self.actions_cache.len() {
                                macro_core::move_action(idx, idx + 1);
                                self.refresh_actions();
                                self.selected_action_index = Some(idx + 1);
                            }
                        }
                        ActionCardEvent::Reorder { from, to } => {
                            let actual_to = if to > from { to.saturating_sub(1) } else { to };
                            if actual_to < self.actions_cache.len() && from != actual_to {
                                macro_core::move_action(from, actual_to);
                                self.refresh_actions();
                                self.selected_action_index = Some(actual_to);
                                self.status_message = match self.lang {
                                    Language::Fr => format!(
                                        "🔀 Action #{} déplacée vers #{}.",
                                        from + 1,
                                        actual_to + 1
                                    ),
                                    Language::En => format!(
                                        "🔀 Action #{} moved to #{}.",
                                        from + 1,
                                        actual_to + 1
                                    ),
                                };
                            }
                        }
                        ActionCardEvent::DelayChanged(idx, delay) => {
                            if idx < self.actions_cache.len() {
                                self.actions_cache[idx].delay_ms = delay;
                                macro_core::update_action(idx, self.actions_cache[idx].clone());
                            }
                        }
                    }
                }
            }
        });

        // Demander un repaint régulier si en enregistrement ou lecture
        if self.is_recording || self.is_playing {
            ctx.request_repaint_after(std::time::Duration::from_millis(33));
        }
    }
}
