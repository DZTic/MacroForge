use crate::events::EngineEvent;
use crate::macro_core::{self, ActionType, MacroAction, MACRO_STATE};
use crate::ui::dialogs::{
    ActionEditorModal, ActionModalTab, ActionModalTarget, StopImageConfigModal, WindowLockModal,
};
use crate::ui::i18n::Language;
use crate::ui::theme::{self, colors};
use crate::ui::widgets::{
    ActionCard, ActionCardEvent, ButtonVariant, CustomToggleSwitch, FilterBar, GlassButton,
    StatusBadge, StatusKind,
};
use eframe::egui::{self, Color32, Frame, Margin, Rounding, Stroke};
use std::sync::mpsc::Receiver;

/// Requete de recherche pre-normalisee : construite une seule fois par recalcul du cache.
struct SearchQuery {
    raw: String,
}

/// Mode d'affichage Studio lors de l'intégration d'une fenêtre cible
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioViewMode {
    Split,
    Timeline,
    Game,
}

pub struct MacroForgeApp {
    rx_events: Receiver<EngineEvent>,
    is_recording: bool,
    is_playing: bool,
    loop_playback: bool,
    actions_cache: Vec<MacroAction>,
    status_message: String,

    // Internationalisation
    lang: Language,

    // Mode Studio
    studio_view_mode: StudioViewMode,

    // Filtres & Recherche
    hide_mouse_moves: bool,
    search_query: String,
    // Cache du filtrage : recalcule seulement quand la liste ou le filtre change (issue #32)
    filtered_indices: Vec<usize>,
    filtered_cache_valid: bool,
    last_filter_query: Option<String>,
    last_filter_hide_moves: Option<bool>,
    // Version de la liste d'actions : incrémentée à chaque mutation (issue #15)
    actions_version: u64,
    last_filter_actions_version: u64,
    jump_index: usize,
    scroll_target_index: Option<usize>,

    // Sélection
    selected_action_index: Option<usize>,

    // Modales & dialogues
    action_modal: ActionEditorModal,
    stop_image_modal: StopImageConfigModal,
    window_lock_modal: WindowLockModal,

    // Toolbar flottante native
    toolbar: crate::ui::FloatingToolbar,

    // Visibilité de la fenêtre principale
    main_window_visible: bool,

    // Overlay transparent click-through
    overlay: crate::ui::TransparentOverlay,

    // Horodatage du démarrage de la lecture pour éviter les auto-clics résiduels
    playback_started_at: Option<std::time::Instant>,
}

impl MacroForgeApp {
    pub fn new(cc: &eframe::CreationContext<'_>, rx_events: Receiver<EngineEvent>) -> Self {
        // Appliquer le thème Glassmorphism et la typographie au contexte egui
        theme::apply_theme(&cc.egui_ctx);
        macro_core::set_egui_ctx(cc.egui_ctx.clone());

        let settings = crate::ui::i18n::AppSettings::load();
        let initial_loop = macro_core::get_loop_playback() || settings.loop_playback;
        if settings.loop_playback {
            macro_core::set_loop_playback(true);
        }
        macro_core::set_window_lock(settings.window_lock.clone());

        let initial_actions = {
            let state = MACRO_STATE.lock().unwrap();
            state.actions.clone()
        };

        let lang = settings.language;
        let ready_msg = lang.ready_status().to_string();
        let total_actions = initial_actions.len();

        Self {
            rx_events,
            is_recording: false,
            is_playing: false,
            loop_playback: initial_loop,
            actions_cache: initial_actions,
            status_message: ready_msg,

            lang,
            studio_view_mode: StudioViewMode::Split,
            hide_mouse_moves: settings.hide_mouse_moves,
            search_query: String::new(),
            filtered_indices: Vec::new(),
            filtered_cache_valid: false,
            last_filter_query: None,
            last_filter_hide_moves: None,
            actions_version: 0,
            last_filter_actions_version: 0,
            jump_index: 1,
            scroll_target_index: None,
            selected_action_index: None,

            action_modal: ActionEditorModal::new(),
            stop_image_modal: StopImageConfigModal::new(),
            window_lock_modal: WindowLockModal::new(),

            toolbar: crate::ui::FloatingToolbar {
                is_visible: false,
                current_action_idx: 0,
                total_actions,
                action_detail: String::new(),
            },

            main_window_visible: true,

            overlay: crate::ui::TransparentOverlay {
                is_visible: false,
                current_action_idx: 0,
                total_actions,
                action_type_label: String::new(),
                action_detail: String::new(),
                target_x: None,
                target_y: None,
                win32_configured: false,
            },

            playback_started_at: None,
        }
    }

    fn save_current_settings(&self) {
        let settings = crate::ui::i18n::AppSettings {
            language: self.lang,
            loop_playback: self.loop_playback,
            hide_mouse_moves: self.hide_mouse_moves,
            window_lock: macro_core::get_window_lock(),
        };
        settings.save();
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
                    self.overlay.is_visible = play;
                    if play {
                        self.playback_started_at = Some(std::time::Instant::now());
                        self.status_message = match self.lang {
                            Language::Fr => {
                                "▶️ Lecture en cours (F7: Pause, F4: Arrêt Urgence)...".to_string()
                            }
                            Language::En => {
                                "▶️ Playback in progress (F7: Pause, F4: Emergency Stop)..."
                                    .to_string()
                            }
                        };
                    } else {
                        self.playback_started_at = None;
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
                    self.toolbar.current_action_idx = action.index;
                    self.toolbar.total_actions = action.total;
                    self.toolbar.action_detail =
                        format!("{} ({})", action.action_type, action.detail);

                    self.overlay.current_action_idx = action.index;
                    self.overlay.total_actions = action.total;
                    self.overlay.action_type_label = action.action_type;
                    self.overlay.action_detail = action.detail;
                }
            }
        }
    }

    fn refresh_actions(&mut self) {
        self.actions_cache = macro_core::get_actions();
        self.toolbar.total_actions = self.actions_cache.len();
        self.invalidate_filtered_cache();
    }

    fn invalidate_filtered_cache(&mut self) {
        // Toute mutation de la liste incrémente la version : le cache du filtre
        // est mécaniquement obsolète même si un chemin oubliait ce flag (issue #15).
        self.actions_version = self.actions_version.wrapping_add(1);
        self.filtered_cache_valid = false;
    }

    /// Reconstruit les indices visibles uniquement si le cache est invalide.
    fn ensure_filtered_indices(&mut self) {
        let filter_changed = self.last_filter_query.as_deref() != Some(self.search_query.trim())
            || self.last_filter_hide_moves != Some(self.hide_mouse_moves)
            || self.last_filter_actions_version != self.actions_version;
        if self.filtered_cache_valid && !filter_changed {
            return;
        }

        let query = Self::build_search_query(&self.search_query);
        let hide_moves = self.hide_mouse_moves;

        self.filtered_indices = self
            .actions_cache
            .iter()
            .enumerate()
            .filter_map(|(idx, act)| {
                if Self::action_matches_filter(act, hide_moves, &query) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();

        self.filtered_cache_valid = true;
        self.last_filter_query = Some(self.search_query.trim().to_string());
        self.last_filter_hide_moves = Some(hide_moves);
        self.last_filter_actions_version = self.actions_version;
    }

    /// Normalise la requete une seule fois par recalcul du cache (issue #32).
    fn build_search_query(input: &str) -> SearchQuery {
        SearchQuery {
            raw: input.trim().to_lowercase(),
        }
    }

    /// Matche la forme entiere ou decimale : 10 matche 10.5 comme avant.
    fn number_matches_query(raw: &str, value: f64) -> bool {
        let bytes = raw.as_bytes();
        let as_int = value as i64;
        if Self::contains_int(bytes, as_int) {
            return true;
        }
        let frac = value - as_int as f64;
        if frac <= 0.0 {
            return false;
        }
        // Forme decimale "123.45" construite dans un buffer pile, sans format!.
        let mut buf = [0u8; 32];
        let mut len = Self::write_int_into(&mut buf, as_int);
        buf[len] = b'.';
        len += 1;
        len += Self::write_frac_into(&mut buf[len..], frac);
        bytes.len() >= len && bytes.windows(len).any(|w| w == &buf[..len])
    }

    /// Ecrit la representation decimale de n dans buf, retourne la longueur ecrite.
    fn write_int_into(buf: &mut [u8], n: i64) -> usize {
        if n == 0 {
            buf[0] = b'0';
            return 1;
        }
        let mut scratch = [0u8; 20];
        let mut len = 0usize;
        let mut m = n;
        let negative = m < 0;
        while m != 0 {
            scratch[len] = b'0' + (m % 10).unsigned_abs() as u8;
            len += 1;
            m /= 10;
        }
        let mut out = 0usize;
        if negative {
            buf[out] = b'-';
            out += 1;
        }
        while len > 0 {
            len -= 1;
            buf[out] = scratch[len];
            out += 1;
        }
        out
    }

    /// Ecrit les decimales significatives d'une fraction (ex : 0.45 -> "45").
    fn write_frac_into(buf: &mut [u8], mut frac: f64) -> usize {
        let mut len = 0usize;
        while frac > 0.0 && len < buf.len() && len < 12 {
            frac *= 10.0;
            let digit = frac as u8;
            buf[len] = b'0' + digit.min(9);
            len += 1;
            frac -= digit as f64;
        }
        while len > 0 && buf[len - 1] == b'0' {
            len -= 1;
        }
        len
    }

    /// Recherche sans allocation d'un entier dans une requete (issue #15).
    fn contains_int(bytes: &[u8], n: i64) -> bool {
        let mut buf = [0u8; 21];
        let len = Self::write_int_into(&mut buf, n);
        bytes.len() >= len && bytes.windows(len).any(|w| w == &buf[..len])
    }

    /// Recherche sans allocation de la representation hexadecimale minuscule de vk.
    fn contains_hex_u16(bytes: &[u8], vk: u16) -> bool {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut buf = [0u8; 4];
        let mut len = 0usize;
        let mut v = vk;
        if v == 0 {
            buf[0] = b'0';
            len = 1;
        } else {
            while v != 0 {
                buf[len] = HEX[(v % 16) as usize];
                len += 1;
                v /= 16;
            }
            buf[..len].reverse();
        }
        bytes.len() >= len && bytes.windows(len).any(|w| w == &buf[..len])
    }

    /// Equivalent sans allocation de raw.contains(format!("btn {b}")).
    fn contains_btn(bytes: &[u8], btn: u8) -> bool {
        const PREFIX: &[u8] = b"btn ";
        let digit = b'0' + btn.min(9);
        for i in 0..bytes.len() {
            if bytes[i] == PREFIX[0]
                && bytes[i..].starts_with(PREFIX)
                && bytes.get(i + PREFIX.len()) == Some(&digit)
            {
                return true;
            }
        }
        false
    }

    /// Matche un texte en minuscules sans re-allouer la requete.
    fn text_matches_query(text: &str, raw: &str) -> bool {
        text.to_lowercase().contains(raw)
    }

    /// Filtrage d'une action, comportement identique a l'ancien matches_filter.
    fn action_matches_filter(action: &MacroAction, hide_moves: bool, q: &SearchQuery) -> bool {
        if hide_moves {
            match &action.action_type {
                ActionType::MouseMove(_, _) | ActionType::MouseMoveRelative(_, _) => return false,
                _ => {}
            }
        }

        if q.raw.is_empty() {
            return true;
        }

        let raw = &q.raw;
        let raw_bytes = raw.as_bytes();
        match &action.action_type {
            ActionType::KeyPress(name, vk, _) => {
                Self::text_matches_query(name, raw)
                    || Self::contains_hex_u16(raw_bytes, *vk)
                    || raw.contains("keypress")
            }
            ActionType::KeyRelease(name, vk, _) => {
                Self::text_matches_query(name, raw)
                    || Self::contains_hex_u16(raw_bytes, *vk)
                    || raw.contains("keyrelease")
            }
            ActionType::MouseMove(x, y) => {
                raw.contains("move")
                    || raw.contains("souris")
                    || Self::number_matches_query(raw, *x)
                    || Self::number_matches_query(raw, *y)
            }
            ActionType::MouseMoveRelative(dx, dy) => {
                raw.contains("rel")
                    || raw.contains("relative")
                    || Self::number_matches_query(raw, f64::from(*dx))
                    || Self::number_matches_query(raw, f64::from(*dy))
            }
            ActionType::MousePress(btn, x, y) => {
                Self::contains_btn(raw_bytes, *btn)
                    || raw.contains("click")
                    || raw.contains("clic")
                    || Self::number_matches_query(raw, *x)
                    || Self::number_matches_query(raw, *y)
            }
            ActionType::MouseRelease(btn, _, _) => {
                Self::contains_btn(raw_bytes, *btn) || raw.contains("release")
            }
            ActionType::Scroll(dx, dy) => {
                raw.contains("scroll")
                    || raw.contains("molette")
                    || Self::number_matches_query(raw, *dx)
                    || Self::number_matches_query(raw, *dy)
            }
            ActionType::Wait(ms) => {
                Self::contains_int(raw_bytes, *ms as i64)
                    || raw.contains("wait")
                    || raw.contains("pause")
            }
            ActionType::WaitImage(path, timeout) => {
                Self::text_matches_query(path, raw)
                    || Self::contains_int(raw_bytes, *timeout as i64)
                    || raw.contains("image")
            }
        }
    }

    fn render_embedded_viewport_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let win_lock_cfg = macro_core::get_window_lock();
        let target_title = macro_core::get_embedded_target_title().unwrap_or_else(|| {
            if !win_lock_cfg.title_filter.trim().is_empty() {
                win_lock_cfg.title_filter.clone()
            } else {
                self.lang.viewport_header_title().to_string()
            }
        });

        // En-tête du Viewport
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("🎮 {}", target_title))
                    .strong()
                    .size(13.5)
                    .color(colors::TEXT_PRIMARY),
            );

            let status_badge = Frame::none()
                .fill(Color32::from_rgba_unmultiplied(16, 185, 129, 30))
                .stroke(Stroke::new(
                    1.0_f32,
                    Color32::from_rgba_unmultiplied(16, 185, 129, 120),
                ))
                .rounding(Rounding::same(4.0))
                .inner_margin(Margin::symmetric(6.0, 2.0));

            status_badge.show(ui, |ui| {
                ui.label(
                    egui::RichText::new(self.lang.viewport_status_docked())
                        .strong()
                        .size(11.0)
                        .color(colors::ACCENT_SUCCESS),
                );
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Bouton Détacher / Rétablir
                let detach_btn = GlassButton::new(self.lang.viewport_detach_btn())
                    .icon("🔓")
                    .compact(true)
                    .variant(ButtonVariant::Secondary);
                if ui
                    .add(detach_btn)
                    .on_hover_text("Détacher la fenêtre et la rétablir sur le bureau Windows")
                    .clicked()
                {
                    let cfg = macro_core::get_window_lock();
                    let _ = macro_core::restore_target_window(&cfg);
                    let mut new_cfg = cfg;
                    new_cfg.embed_in_macroforge = false;
                    macro_core::set_window_lock(new_cfg.clone());
                    let mut settings = crate::ui::i18n::AppSettings::load();
                    settings.window_lock = new_cfg;
                    settings.save();
                    self.status_message = match self.lang {
                        Language::Fr => {
                            "🔓 Fenêtre cible détachée et rétablie sur le bureau.".to_string()
                        }
                        Language::En => {
                            "🔓 Target window detached and restored to desktop.".to_string()
                        }
                    };
                }

                ui.add_space(4.0);

                // Bouton Configurer
                let cfg_btn = GlassButton::new("⚙")
                    .compact(true)
                    .variant(ButtonVariant::Ghost);
                if ui
                    .add(cfg_btn)
                    .on_hover_text("Ouvrir les paramètres de verrouillage")
                    .clicked()
                {
                    self.window_lock_modal.open();
                }
            });
        });

        ui.add_space(4.0);

        // Cadre visuel délimité pour le Viewport
        let viewport_frame = Frame::none()
            .fill(Color32::from_rgba_unmultiplied(10, 15, 26, 220))
            .stroke(Stroke::new(
                1.0_f32,
                Color32::from_rgba_unmultiplied(59, 130, 246, 120),
            ))
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::same(4.0));

        viewport_frame.show(ui, |ui| {
            let avail_size = ui.available_size_before_wrap();
            let (rect, _response) = ui.allocate_exact_size(avail_size, egui::Sense::hover());

            // Synchroniser les coordonnées physiques pour le SetWindowPos Win32
            let ppp = ctx.pixels_per_point();
            let phys_x = (rect.min.x * ppp).round() as i32;
            let phys_y = (rect.min.y * ppp).round() as i32;
            let phys_w = (rect.width() * ppp).round() as i32;
            let phys_h = (rect.height() * ppp).round() as i32;

            if phys_w > 50 && phys_h > 50 {
                macro_core::update_embedded_viewport_bounds(phys_x, phys_y, phys_w, phys_h, true);
            }
        });
    }

    fn render_timeline_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
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
                    .compact(true)
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

        ui.add_space(6.0);

        // Filtrage des actions
        let total_count = self.actions_cache.len();
        // Cache : recalcul seulement si la liste ou le filtre a change (issue #32)
        self.ensure_filtered_indices();
        let filtered_indices = &self.filtered_indices;
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

        ui.add_space(8.0);

        if self.actions_cache.is_empty() {
            // État vide élégant Dark Glassmorphism avec actions rapides
            ui.vertical_centered(|ui| {
                ui.add_space(35.0);

                let empty_card = theme::glass_card_frame();
                empty_card.show(ui, |ui| {
                    ui.set_max_width(520.0);
                    ui.vertical_centered(|ui| {
                        ui.add_space(14.0);
                        ui.label(
                            egui::RichText::new("⚡")
                                .size(44.0)
                                .color(colors::ACCENT_PRIMARY_HOVER),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(self.lang.empty_state_title())
                                .strong()
                                .size(17.0)
                                .color(colors::TEXT_PRIMARY),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new(self.lang.empty_state_desc())
                                .color(colors::TEXT_SECONDARY)
                                .size(13.5),
                        );
                        ui.add_space(16.0);

                        // Boutons d'action rapide pour démarrer immédiatement
                        ui.horizontal(|ui| {
                            let rec_quick_btn = GlassButton::new(if self.lang == Language::Fr {
                                "Enregistrer (F8)"
                            } else {
                                "Record (F8)"
                            })
                            .icon("🔴")
                            .variant(ButtonVariant::Danger);
                            if ui.add(rec_quick_btn).clicked() {
                                macro_core::start_recording();
                            }

                            ui.add_space(6.0);

                            let key_quick_btn = GlassButton::new(self.lang.quick_add_key())
                                .variant(ButtonVariant::Secondary);
                            if ui.add(key_quick_btn).clicked() {
                                self.action_modal.open_for_new(ActionModalTab::Keyboard);
                            }

                            ui.add_space(6.0);

                            let mouse_quick_btn = GlassButton::new(self.lang.quick_add_mouse())
                                .variant(ButtonVariant::Secondary);
                            if ui.add(mouse_quick_btn).clicked() {
                                self.action_modal.open_for_new(ActionModalTab::Mouse);
                            }
                        });

                        ui.add_space(14.0);
                    });
                });
            });
        } else {
            // Liste scrollable des ActionCards avec support Drag & Drop
            let mut card_event_to_process = None;
            let is_unfiltered = !self.hide_mouse_moves && self.search_query.trim().is_empty();

            // Virtualisation de la liste (issue #10) : seules les cartes
            // visibles dans la fenetre sont layoutees/dessinees par frame.
            const ROW_HEIGHT: f32 = 34.0;
            let total_rows = if is_unfiltered {
                total_count
            } else {
                filtered_indices.len()
            };

            let mut timeline_scroll = egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .id_salt("timeline_scroll_area");
            if let Some(target) = self.scroll_target_index {
                let target_row = if is_unfiltered {
                    Some(target)
                } else {
                    filtered_indices.iter().position(|&i| i == target)
                };
                if let Some(row) = target_row {
                    // Lignes a hauteur fixe : offset = row * ROW_HEIGHT.
                    timeline_scroll =
                        timeline_scroll.scroll_offset(egui::vec2(0.0, row as f32 * ROW_HEIGHT));
                }
            }

            timeline_scroll.show_rows(ui, ROW_HEIGHT, total_rows, |ui, row_range| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 4.0);

                for row in row_range {
                    if is_unfiltered {
                        let idx = row;
                        if let Some(action) = self.actions_cache.get(idx) {
                            let card = ActionCard::new(idx, action)
                                .selected(self.selected_action_index == Some(idx))
                                .lang(self.lang)
                                .bounds(idx == 0, idx == total_count - 1);
                            let (_, ev) = card.show(ui);
                            if let Some(e) = ev {
                                card_event_to_process = Some(e);
                            }
                        }
                    } else if let Some(&original_idx) = filtered_indices.get(row) {
                        let is_selected = self.selected_action_index == Some(original_idx);
                        if let Some(action) = self.actions_cache.get(original_idx) {
                            let card = ActionCard::new(original_idx, action)
                                .selected(is_selected)
                                .lang(self.lang)
                                .bounds(original_idx == 0, original_idx == total_count - 1);
                            let (_, ev) = card.show(ui);
                            if let Some(e) = ev {
                                card_event_to_process = Some(e);
                            }
                        }
                    }
                }
            });

            // Nettoyer l'état de glisser-déposer si le pointeur est relâché
            if ctx.input(|i| i.pointer.any_released()) {
                let dnd_payload_id = egui::Id::new("timeline_dnd_dragged_idx");
                ui.data_mut(|d| d.remove_temp::<usize>(dnd_payload_id));
            }

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
                                Language::En => {
                                    format!("🔀 Action #{} moved to #{}.", from + 1, actual_to + 1)
                                }
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
    }
}

impl eframe::App for MacroForgeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Maintien du thème sombre uniquement en cas de réinitialisation externe (issue #20)
        if !ctx.style().visuals.dark_mode {
            theme::apply_visuals(ctx);
        }

        self.update_from_events();

        // Gestion de la fermeture de la fenêtre principale (ROOT viewport)
        if ctx.input(|i| i.viewport().close_requested()) && self.toolbar.is_visible {
            // Si la toolbar flottante est ouverte, on annule la fermeture globale de l'app
            // et on masque uniquement la fenêtre principale.
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            self.main_window_visible = false;
        }

        // Raccourcis clavier locaux in-app (F8: Rec/Stop, F9: Stop Rec, F7: Play/Stop, F4: Stop Playback, F10: Détacher fenêtre)
        if !self.action_modal.is_listening_key {
            if ctx.input(|i| i.key_pressed(egui::Key::F8)) {
                macro_core::toggle_recording();
            }
            if ctx.input(|i| i.key_pressed(egui::Key::F9)) && self.is_recording {
                macro_core::stop_recording();
            }
            if ctx.input(|i| i.key_pressed(egui::Key::F7)) {
                macro_core::toggle_playback();
            }
            if ctx.input(|i| i.key_pressed(egui::Key::F4)) {
                macro_core::emergency_stop();
            }
            if ctx.input(|i| i.key_pressed(egui::Key::F10)) {
                let cfg = macro_core::get_window_lock();
                let _ = macro_core::restore_target_window(&cfg);
                let mut new_cfg = cfg;
                new_cfg.embed_in_macroforge = false;
                macro_core::set_window_lock(new_cfg.clone());
                let mut settings = crate::ui::i18n::AppSettings::load();
                settings.window_lock = new_cfg;
                settings.save();
                self.status_message = match self.lang {
                    Language::Fr => {
                        "🔓 Fenêtre cible détachée et rétablie sur le bureau.".to_string()
                    }
                    Language::En => {
                        "🔓 Target window detached and restored to desktop.".to_string()
                    }
                };
            }
        }

        // Si une boîte de dialogue modale est active, masquer temporairement la fenêtre enfant Win32
        // afin qu'elle ne recouvre jamais la pop-up egui ni n'intercepte les clics de souris
        let any_modal_open = self.action_modal.is_open
            || self.stop_image_modal.is_open
            || self.window_lock_modal.is_open;
        if any_modal_open {
            macro_core::hide_embedded_target_window();
        }

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

        if self.window_lock_modal.show(ctx, self.lang) {
            self.status_message = match self.lang {
                Language::Fr => "✅ Configuration de la fenêtre cible enregistrée.".to_string(),
                Language::En => "✅ Target window lock configuration saved.".to_string(),
            };
        }

        // 2. Toolbar flottante native (Multi-viewport)
        let is_embedded = macro_core::get_window_lock().embed_in_macroforge
            || macro_core::is_target_window_embedded();
        match self.toolbar.show(
            ctx,
            self.is_recording,
            self.is_playing,
            is_embedded,
            self.lang,
        ) {
            crate::ui::ToolbarAction::None => {}
            crate::ui::ToolbarAction::ToggleRecord => {
                macro_core::toggle_recording();
            }
            crate::ui::ToolbarAction::TogglePlay => {
                macro_core::play_macro();
            }
            crate::ui::ToolbarAction::EmergencyStop => {
                let can_stop = if let Some(started) = self.playback_started_at {
                    started.elapsed() >= std::time::Duration::from_millis(300)
                } else {
                    true
                };
                if can_stop {
                    macro_core::emergency_stop();
                }
            }
            crate::ui::ToolbarAction::DetachTargetWindow => {
                let cfg = macro_core::get_window_lock();
                let _ = macro_core::restore_target_window(&cfg);
                let mut new_cfg = cfg;
                new_cfg.embed_in_macroforge = false;
                macro_core::set_window_lock(new_cfg.clone());
                let mut settings = crate::ui::i18n::AppSettings::load();
                settings.window_lock = new_cfg;
                settings.save();
                self.status_message = match self.lang {
                    Language::Fr => {
                        "🔓 Fenêtre cible détachée et rétablie sur le bureau.".to_string()
                    }
                    Language::En => {
                        "🔓 Target window detached and restored to desktop.".to_string()
                    }
                };
            }
            crate::ui::ToolbarAction::OpenMainWindow => {
                self.main_window_visible = true;
                ctx.send_viewport_cmd_to(
                    egui::ViewportId::ROOT,
                    egui::ViewportCommand::Visible(true),
                );
                ctx.send_viewport_cmd_to(
                    egui::ViewportId::ROOT,
                    egui::ViewportCommand::Minimized(false),
                );
                ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Focus);
            }
            crate::ui::ToolbarAction::CloseToolbar => {
                self.toolbar.is_visible = false;
                if !self.main_window_visible {
                    // Si la fenêtre principale était masquée et qu'on ferme la toolbar, quitter l'application
                    ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Close);
                }
            }
        }

        // 3. Overlay transparent click-through (HUD temps réel pendant la lecture)
        self.overlay.show(ctx, self.is_playing);

        // 4. En-tête supérieur (Header & Quick Actions Responsive)
        egui::TopBottomPanel::top("header_panel")
            .frame(theme::header_frame())
            .show(ctx, |ui| {
                let avail_w = ui.available_width();
                let is_compact = avail_w < 780.0;
                let is_very_compact = avail_w < 650.0;

                if !is_very_compact {
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
                            .fill(Color32::from_rgba_unmultiplied(30, 41, 59, 180))
                            .stroke(Stroke::new(
                                1.0_f32,
                                Color32::from_rgba_unmultiplied(96, 165, 250, 120),
                            ))
                            .rounding(Rounding::same(4.0))
                            .inner_margin(Margin::symmetric(6.0, 2.0));

                        version_badge.show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                                    .monospace()
                                    .strong()
                                    .color(colors::TEXT_PRIMARY)
                                    .size(10.5),
                            );
                        });

                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(6.0);

                        // Boutons d'ajout rapide d'action
                        let key_btn = GlassButton::new(self.lang.quick_add_key())
                            .compact(is_compact)
                            .variant(ButtonVariant::Secondary);
                        if ui
                            .add(key_btn)
                            .on_hover_text("Ajouter un événement clavier manuellement")
                            .clicked()
                        {
                            self.action_modal.open_for_new(ActionModalTab::Keyboard);
                        }

                        let mouse_btn = GlassButton::new(self.lang.quick_add_mouse())
                            .compact(is_compact)
                            .variant(ButtonVariant::Secondary);
                        if ui
                            .add(mouse_btn)
                            .on_hover_text("Ajouter un événement souris manuellement")
                            .clicked()
                        {
                            self.action_modal.open_for_new(ActionModalTab::Mouse);
                        }

                        let wait_btn = GlassButton::new(self.lang.quick_add_wait())
                            .compact(is_compact)
                            .variant(ButtonVariant::Secondary);
                        if ui
                            .add(wait_btn)
                            .on_hover_text("Ajouter un délai de pause")
                            .clicked()
                        {
                            self.action_modal.open_for_new(ActionModalTab::Wait);
                        }

                        let img_btn = GlassButton::new(self.lang.quick_add_image())
                            .compact(is_compact)
                            .variant(ButtonVariant::Secondary);
                        if ui
                            .add(img_btn)
                            .on_hover_text("Ajouter une attente de détection d'image")
                            .clicked()
                        {
                            self.action_modal.open_for_new(ActionModalTab::Image);
                        }

                        // Commandes alignées à droite sans débordement
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Badge d'état dynamique
                            let status_badge = if self.is_recording {
                                StatusBadge::new(StatusKind::Recording).compact(is_compact)
                            } else if self.is_playing {
                                StatusBadge::new(StatusKind::Playing).compact(is_compact)
                            } else {
                                StatusBadge::new(StatusKind::Idle).compact(is_compact)
                            };
                            ui.add(status_badge).on_hover_text(
                                "État du moteur (F8: Rec, F9: Stop, F4: Stop Playback)",
                            );

                            ui.add_space(4.0);
                            ui.separator();
                            ui.add_space(4.0);

                            // Sélecteur de langue interactif
                            let lang_btn = GlassButton::new(match self.lang {
                                Language::Fr => "FR",
                                Language::En => "EN",
                            })
                            .icon("🌐")
                            .compact(is_compact)
                            .variant(ButtonVariant::Ghost);
                            if ui
                                .add(lang_btn)
                                .on_hover_text("Changer de langue (FR / EN)")
                                .clicked()
                            {
                                self.lang.toggle();
                                self.save_current_settings();
                            }

                            // Bouton Toolbar flottante
                            let toolbar_btn = GlassButton::new(self.lang.toolbar_window_btn())
                                .icon("🗔")
                                .compact(is_compact)
                                .selected(self.toolbar.is_visible)
                                .variant(if self.toolbar.is_visible {
                                    ButtonVariant::Primary
                                } else {
                                    ButtonVariant::Ghost
                                });
                            if ui
                                .add(toolbar_btn)
                                .on_hover_text("Afficher/Masquer la toolbar flottante")
                                .clicked()
                            {
                                self.toolbar.is_visible = !self.toolbar.is_visible;
                                self.toolbar.total_actions = self.actions_cache.len();
                                self.status_message = if self.toolbar.is_visible {
                                    match self.lang {
                                        Language::Fr => "🗔 Toolbar flottante affichée.".to_string(),
                                        Language::En => "🗔 Floating toolbar shown.".to_string(),
                                    }
                                } else {
                                    match self.lang {
                                        Language::Fr => "🗔 Toolbar flottante masquée.".to_string(),
                                        Language::En => "🗔 Floating toolbar hidden.".to_string(),
                                    }
                                };
                            }
                        });
                    });
                } else {
                    // Disposition 2 rangées pour fenêtres très étroites (< 650px)
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(self.lang.app_title())
                                    .heading()
                                    .color(colors::TEXT_PRIMARY)
                                    .strong(),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let status_badge = if self.is_recording {
                                        StatusBadge::new(StatusKind::Recording).compact(true)
                                    } else if self.is_playing {
                                        StatusBadge::new(StatusKind::Playing).compact(true)
                                    } else {
                                        StatusBadge::new(StatusKind::Idle).compact(true)
                                    };
                                    ui.add(status_badge);

                                    let lang_btn = GlassButton::new(match self.lang {
                                        Language::Fr => "FR",
                                        Language::En => "EN",
                                    })
                                    .icon("🌐")
                                    .compact(true)
                                    .variant(ButtonVariant::Ghost);
                                    if ui.add(lang_btn).clicked() {
                                        self.lang.toggle();
                                        self.save_current_settings();
                                    }

                                    let toolbar_btn = GlassButton::new("Toolbar")
                                        .icon("🗔")
                                        .compact(true)
                                        .selected(self.toolbar.is_visible)
                                        .variant(if self.toolbar.is_visible {
                                            ButtonVariant::Primary
                                        } else {
                                            ButtonVariant::Ghost
                                        });
                                    if ui.add(toolbar_btn).clicked() {
                                        self.toolbar.is_visible = !self.toolbar.is_visible;
                                    }
                                },
                            );
                        });

                        ui.add_space(3.0);

                        ui.horizontal(|ui| {
                            let key_btn = GlassButton::new("+ Clavier")
                                .compact(true)
                                .variant(ButtonVariant::Secondary);
                            if ui.add(key_btn).clicked() {
                                self.action_modal.open_for_new(ActionModalTab::Keyboard);
                            }
                            let mouse_btn = GlassButton::new("+ Souris")
                                .compact(true)
                                .variant(ButtonVariant::Secondary);
                            if ui.add(mouse_btn).clicked() {
                                self.action_modal.open_for_new(ActionModalTab::Mouse);
                            }
                            let wait_btn = GlassButton::new("+ Pause")
                                .compact(true)
                                .variant(ButtonVariant::Secondary);
                            if ui.add(wait_btn).clicked() {
                                self.action_modal.open_for_new(ActionModalTab::Wait);
                            }
                            let img_btn = GlassButton::new("+ Image")
                                .compact(true)
                                .variant(ButtonVariant::Secondary);
                            if ui.add(img_btn).clicked() {
                                self.action_modal.open_for_new(ActionModalTab::Image);
                            }
                        });
                    });
                }
            });

        // 3. Barre inférieure de contrôle global (Footer Responsive)
        egui::TopBottomPanel::bottom("footer_panel")
            .frame(theme::footer_frame())
            .show(ctx, |ui| {
                let avail_w = ui.available_width();
                let is_compact = avail_w < 820.0;

                if !is_compact {
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
                                macro_core::toggle_recording();
                            }
                        } else {
                            let btn = GlassButton::new(self.lang.stop_btn())
                                .icon("⏹")
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
                                .icon("▶")
                                .shortcut("F7")
                                .variant(ButtonVariant::Success);
                            if ui
                                .add(btn)
                                .on_hover_text("Exécuter la séquence de macro enregistrée (F7)")
                                .clicked()
                            {
                                macro_core::play_macro();
                            }
                        } else {
                            let btn = GlassButton::new(self.lang.emergency_stop_btn())
                                .icon("⏹")
                                .shortcut("F4")
                                .variant(ButtonVariant::Warning);
                            if ui
                                .add(btn)
                                .on_hover_text("Arrêter immédiatement la relecture (F4)")
                                .clicked()
                            {
                                let can_stop = if let Some(started) = self.playback_started_at {
                                    started.elapsed() >= std::time::Duration::from_millis(300)
                                } else {
                                    true
                                };
                                if can_stop {
                                    macro_core::emergency_stop();
                                }
                            }
                        }

                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(6.0);

                        // Switch Mode Boucle
                        let toggle = CustomToggleSwitch::new(&mut self.loop_playback)
                            .label(self.lang.loop_mode_label());
                        if ui
                            .add(toggle)
                            .on_hover_text(
                                "Répéter la macro indéfiniment jusqu'à l'arrêt d'urgence F4",
                            )
                            .changed()
                        {
                            macro_core::set_loop_playback(self.loop_playback);
                        }

                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(6.0);

                        // Bouton Configuration Image d'arrêt
                        let (stop_img, _) = macro_core::get_stop_image();
                        let has_stop_img = stop_img.is_some();
                        let stop_img_btn = GlassButton::new(self.lang.stop_image_cfg_btn())
                            .icon("🛑")
                            .variant(if has_stop_img {
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

                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(6.0);

                        // Bouton Configuration Verrouillage Fenêtre Cible
                        let win_lock_cfg = macro_core::get_window_lock();
                        let has_win_lock = win_lock_cfg.enabled;
                        let win_lock_lbl = if has_win_lock {
                            format!(
                                "{} ({}×{})",
                                self.lang.window_lock_btn(),
                                win_lock_cfg.width,
                                win_lock_cfg.height
                            )
                        } else {
                            self.lang.window_lock_btn().to_string()
                        };
                        let win_lock_btn =
                            GlassButton::new(&win_lock_lbl)
                                .icon("🎯")
                                .variant(if has_win_lock {
                                    ButtonVariant::Primary
                                } else {
                                    ButtonVariant::Secondary
                                });
                        if ui
                            .add(win_lock_btn)
                            .on_hover_text(self.lang.window_lock_tooltip())
                            .clicked()
                        {
                            self.window_lock_modal.open();
                        }

                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(6.0);

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
                                        self.status_message =
                                            format!("❌ Erreur sauvegarde: {}", e);
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
                            .icon("🗑")
                            .variant(ButtonVariant::Ghost);
                        if ui
                            .add(clear_btn)
                            .on_hover_text("Effacer toutes les actions enregistrées")
                            .clicked()
                        {
                            macro_core::clear_actions();
                            self.actions_cache.clear();
                            self.invalidate_filtered_cache();
                            self.status_message = match self.lang {
                                Language::Fr => "Toutes les actions ont été effacées.".to_string(),
                                Language::En => "All actions have been cleared.".to_string(),
                            };
                        }
                    });
                } else {
                    // Disposition 2 rangées responsive pour fenêtres compactes
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            if !self.is_recording {
                                let btn = GlassButton::new(self.lang.record_btn())
                                    .icon("🔴")
                                    .shortcut("F8")
                                    .compact(true)
                                    .variant(ButtonVariant::Danger);
                                if ui.add(btn).clicked() {
                                    macro_core::toggle_recording();
                                }
                            } else {
                                let btn = GlassButton::new(self.lang.stop_btn())
                                    .icon("⏹")
                                    .shortcut("F9")
                                    .compact(true)
                                    .variant(ButtonVariant::Secondary);
                                if ui.add(btn).clicked() {
                                    macro_core::stop_recording();
                                }
                            }

                            if !self.is_playing {
                                let btn = GlassButton::new(self.lang.play_btn())
                                    .icon("▶")
                                    .shortcut("F7")
                                    .compact(true)
                                    .variant(ButtonVariant::Success);
                                if ui.add(btn).clicked() {
                                    macro_core::play_macro();
                                }
                            } else {
                                let btn = GlassButton::new(self.lang.emergency_stop_btn())
                                    .icon("⏹")
                                    .shortcut("F4")
                                    .compact(true)
                                    .variant(ButtonVariant::Warning);
                                if ui.add(btn).clicked() {
                                    let can_stop = if let Some(started) = self.playback_started_at {
                                        started.elapsed() >= std::time::Duration::from_millis(300)
                                    } else {
                                        true
                                    };
                                    if can_stop {
                                        macro_core::emergency_stop();
                                    }
                                }
                            }

                            ui.separator();

                            let toggle =
                                CustomToggleSwitch::new(&mut self.loop_playback).label("Boucle");
                            if ui.add(toggle).changed() {
                                macro_core::set_loop_playback(self.loop_playback);
                            }

                            let (stop_img, _) = macro_core::get_stop_image();
                            let has_stop_img = stop_img.is_some();
                            let stop_img_btn = GlassButton::new("Arrêt image")
                                .icon("🛑")
                                .compact(true)
                                .variant(if has_stop_img {
                                    ButtonVariant::Primary
                                } else {
                                    ButtonVariant::Secondary
                                });
                            if ui.add(stop_img_btn).clicked() {
                                self.stop_image_modal.open();
                            }

                            let win_lock_cfg = macro_core::get_window_lock();
                            let has_win_lock = win_lock_cfg.enabled;
                            let win_lock_btn = GlassButton::new("Fenêtre")
                                .icon("🎯")
                                .compact(true)
                                .variant(if has_win_lock {
                                    ButtonVariant::Primary
                                } else {
                                    ButtonVariant::Secondary
                                });
                            if ui
                                .add(win_lock_btn)
                                .on_hover_text(self.lang.window_lock_tooltip())
                                .clicked()
                            {
                                self.window_lock_modal.open();
                            }
                        });

                        ui.add_space(3.0);

                        ui.horizontal(|ui| {
                            let save_btn = GlassButton::new(self.lang.save_profile())
                                .icon("💾")
                                .compact(true)
                                .variant(ButtonVariant::Secondary);
                            if ui.add(save_btn).clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("MacroForge Profile", &["mforge", "json"])
                                    .save_file()
                                {
                                    if let Some(path_str) = path.to_str() {
                                        if let Err(e) = macro_core::save_macro_to_file(path_str) {
                                            self.status_message =
                                                format!("❌ Erreur sauvegarde: {}", e);
                                        } else {
                                            self.status_message =
                                                "✅ Profil sauvegardé avec succès!".to_string();
                                        }
                                    }
                                }
                            }

                            let open_btn = GlassButton::new(self.lang.open_profile())
                                .icon("📂")
                                .compact(true)
                                .variant(ButtonVariant::Secondary);
                            if ui.add(open_btn).clicked() {
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

                            let clear_btn = GlassButton::new(self.lang.clear_actions())
                                .icon("🗑")
                                .compact(true)
                                .variant(ButtonVariant::Ghost);
                            if ui.add(clear_btn).clicked() {
                                macro_core::clear_actions();
                                self.actions_cache.clear();
                                self.invalidate_filtered_cache();
                                self.status_message = match self.lang {
                                    Language::Fr => {
                                        "Toutes les actions ont été effacées.".to_string()
                                    }
                                    Language::En => "All actions have been cleared.".to_string(),
                                };
                            }
                        });
                    });
                }

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(2.0);

                // Ligne de statut informative avec puce lumineuse
                ui.horizontal(|ui| {
                    let dot_color = if self.is_recording {
                        colors::ACCENT_DANGER
                    } else if self.is_playing {
                        colors::ACCENT_SUCCESS
                    } else {
                        colors::ACCENT_PRIMARY
                    };
                    ui.label(egui::RichText::new("●").color(dot_color).size(10.0));
                    ui.label(
                        egui::RichText::new(&self.status_message)
                            .color(colors::TEXT_SECONDARY)
                            .size(12.0),
                    );
                });
            });

        // 4. Panneau central (Timeline, Mode Studio Split ou Viewport Dédié)
        egui::CentralPanel::default()
            .frame(theme::central_panel_frame())
            .show(ctx, |ui| {
                let is_embedded = macro_core::get_window_lock().embed_in_macroforge
                    || macro_core::is_target_window_embedded();

                if is_embedded {
                    // Sélecteur de mode de vue Studio
                    ui.horizontal(|ui| {
                        let modes = [
                            (StudioViewMode::Split, self.lang.studio_mode_split()),
                            (StudioViewMode::Timeline, self.lang.studio_mode_timeline()),
                            (StudioViewMode::Game, self.lang.studio_mode_game()),
                        ];

                        for (mode, lbl) in modes {
                            let is_active = self.studio_view_mode == mode;
                            let btn = GlassButton::new(lbl)
                                .compact(true)
                                .selected(is_active)
                                .variant(if is_active {
                                    ButtonVariant::Primary
                                } else {
                                    ButtonVariant::Ghost
                                });
                            if ui.add(btn).clicked() {
                                self.studio_view_mode = mode;
                            }
                        }
                    });

                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);
                }

                if is_embedded && self.studio_view_mode == StudioViewMode::Split {
                    // Mode Studio Split : Timeline à gauche, Viewport à droite
                    let total_width = ui.available_width();
                    let timeline_width = (total_width * 0.44).clamp(380.0, 520.0);

                    ui.horizontal_top(|ui| {
                        // Volet gauche : Timeline
                        ui.allocate_ui_with_layout(
                            egui::vec2(timeline_width, ui.available_height()),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                self.render_timeline_ui(ui, ctx);
                            },
                        );

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // Volet droit : Viewport
                        ui.allocate_ui_with_layout(
                            egui::vec2(ui.available_width(), ui.available_height()),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                self.render_embedded_viewport_ui(ui, ctx);
                            },
                        );
                    });
                } else if is_embedded && self.studio_view_mode == StudioViewMode::Game {
                    // Mode Jeu seul (Viewport plein panneau central)
                    self.render_embedded_viewport_ui(ui, ctx);
                } else {
                    // Mode Timeline standard (ou fenêtre non intégrée)
                    if is_embedded {
                        // Masquer temporairement la fenêtre enfant pour ne pas recouvrir egui
                        macro_core::update_embedded_viewport_bounds(0, 0, 0, 0, false);
                    }
                    self.render_timeline_ui(ui, ctx);
                }
            });

        // Demander un repaint régulier si en enregistrement ou lecture
        if self.is_recording || self.is_playing {
            ctx.request_repaint_after(std::time::Duration::from_millis(33));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn test_app_main_window_and_toolbar_visibility_defaults() {
        let (_tx, rx) = mpsc::channel();
        let app = MacroForgeApp {
            rx_events: rx,
            is_recording: false,
            is_playing: false,
            loop_playback: false,
            actions_cache: Vec::new(),
            status_message: "Ready".to_string(),
            lang: Language::Fr,
            studio_view_mode: StudioViewMode::Split,
            hide_mouse_moves: false,
            search_query: String::new(),
            filtered_indices: Vec::new(),
            filtered_cache_valid: false,
            last_filter_query: None,
            last_filter_hide_moves: None,
            actions_version: 0,
            last_filter_actions_version: 0,
            jump_index: 1,
            scroll_target_index: None,
            selected_action_index: None,
            action_modal: ActionEditorModal::new(),
            stop_image_modal: StopImageConfigModal::new(),
            window_lock_modal: WindowLockModal::new(),
            toolbar: crate::ui::FloatingToolbar {
                is_visible: false,
                current_action_idx: 0,
                total_actions: 0,
                action_detail: String::new(),
            },
            main_window_visible: true,
            overlay: crate::ui::TransparentOverlay {
                is_visible: false,
                current_action_idx: 0,
                total_actions: 0,
                action_type_label: String::new(),
                action_detail: String::new(),
                target_x: None,
                target_y: None,
                win32_configured: false,
            },
            playback_started_at: None,
        };

        assert!(app.main_window_visible);
        assert!(!app.toolbar.is_visible);
    }

    fn make_test_app(actions: Vec<MacroAction>) -> MacroForgeApp {
        let (_tx, rx) = mpsc::channel();
        MacroForgeApp {
            rx_events: rx,
            is_recording: false,
            is_playing: false,
            loop_playback: false,
            actions_cache: actions,
            status_message: "Ready".to_string(),
            lang: Language::Fr,
            studio_view_mode: StudioViewMode::Split,
            hide_mouse_moves: false,
            search_query: String::new(),
            filtered_indices: Vec::new(),
            filtered_cache_valid: false,
            last_filter_query: None,
            last_filter_hide_moves: None,
            actions_version: 0,
            last_filter_actions_version: 0,
            jump_index: 1,
            scroll_target_index: None,
            selected_action_index: None,
            action_modal: ActionEditorModal::new(),
            stop_image_modal: StopImageConfigModal::new(),
            window_lock_modal: WindowLockModal::new(),
            toolbar: crate::ui::FloatingToolbar {
                is_visible: false,
                current_action_idx: 0,
                total_actions: 0,
                action_detail: String::new(),
            },
            main_window_visible: true,
            overlay: crate::ui::TransparentOverlay {
                is_visible: false,
                current_action_idx: 0,
                total_actions: 0,
                action_type_label: String::new(),
                action_detail: String::new(),
                target_x: None,
                target_y: None,
                win32_configured: false,
            },
            playback_started_at: None,
        }
    }

    fn sample_action(action_type: ActionType) -> MacroAction {
        MacroAction {
            action_type,
            delay_ms: 0,
        }
    }

    #[test]
    fn test_filtered_cache_recomputed_only_on_invalidation() {
        let mut app = make_test_app(vec![
            sample_action(ActionType::KeyPress("A".to_string(), 0x41, false)),
            sample_action(ActionType::MouseMove(120.0, 240.0)),
        ]);

        app.ensure_filtered_indices();
        assert_eq!(app.filtered_indices, vec![0, 1]);

        // Aucun changement : le cache reste valide, aucun recalcul necessaire
        app.ensure_filtered_indices();
        assert!(app.filtered_cache_valid);

        // Mutation de la liste : invalidation puis recalcul complet
        app.actions_cache.push(sample_action(ActionType::Wait(500)));
        app.invalidate_filtered_cache();
        app.ensure_filtered_indices();
        assert_eq!(app.filtered_indices, vec![0, 1, 2]);
    }

    #[test]
    fn test_search_matches_previous_behavior() {
        let mut app = make_test_app(vec![
            sample_action(ActionType::KeyPress("Escape".to_string(), 0x1B, false)),
            sample_action(ActionType::MouseMove(10.0, 20.0)),
            sample_action(ActionType::MousePress(0, 10.5, 20.25)),
            sample_action(ActionType::Scroll(0.0, -3.0)),
            sample_action(ActionType::Wait(750)),
        ]);

        app.search_query = "ESC".to_string();
        app.ensure_filtered_indices();
        assert_eq!(app.filtered_indices, vec![0]);

        app.search_query = "move".to_string();
        app.ensure_filtered_indices();
        assert_eq!(app.filtered_indices, vec![1]);

        app.search_query = "10".to_string();
        app.ensure_filtered_indices();
        assert!(app.filtered_indices.contains(&1) && app.filtered_indices.contains(&2));

        app.search_query = "pause".to_string();
        app.ensure_filtered_indices();
        assert_eq!(app.filtered_indices, vec![4]);
    }

    #[test]
    fn test_hide_mouse_moves_filters_absolute_and_relative() {
        let mut app = make_test_app(vec![
            sample_action(ActionType::KeyPress("A".to_string(), 0x41, false)),
            sample_action(ActionType::MouseMove(1.0, 2.0)),
            sample_action(ActionType::MouseMoveRelative(3, 4)),
        ]);

        app.hide_mouse_moves = true;
        app.ensure_filtered_indices();
        assert_eq!(app.filtered_indices, vec![0]);
    }

    #[test]
    fn test_filter_cache_invalidated_on_actions_version_bump() {
        let mut app = make_test_app(vec![sample_action(ActionType::Wait(100))]);
        app.ensure_filtered_indices();
        assert!(app.filtered_cache_valid);

        // Mutation sans changer la requete : la version d'actions force le recalcul.
        app.actions_cache.push(sample_action(ActionType::Wait(200)));
        app.invalidate_filtered_cache();
        assert_ne!(app.last_filter_actions_version, app.actions_version);
        app.ensure_filtered_indices();
        assert_eq!(app.filtered_indices, vec![0, 1]);
        assert_eq!(app.last_filter_actions_version, app.actions_version);
    }

    #[test]
    fn test_number_matching_no_alloc_equivalence() {
        // Entier : "10" doit matcher 10 et 10.5 (comportement historique).
        assert!(MacroForgeApp::number_matches_query("10", 10.0));
        assert!(MacroForgeApp::number_matches_query("10", 10.5));
        assert!(!MacroForgeApp::number_matches_query("11", 10.0));
        // Semantique historique substring : "10.6".contains("10") == vrai.
        assert!(MacroForgeApp::number_matches_query("10.6", 10.5));
        // Decimal exact.
        assert!(MacroForgeApp::number_matches_query("10.5", 10.5));
        assert!(!MacroForgeApp::number_matches_query("12.6", 10.5));
        // Negatifs.
        assert!(MacroForgeApp::number_matches_query("-3", -3.0));
        assert!(!MacroForgeApp::number_matches_query("-4", -3.0));
        // Zero.
        assert!(MacroForgeApp::number_matches_query("0", 0.0));
    }

    #[test]
    fn test_btn_and_hex_matching_no_alloc() {
        assert!(MacroForgeApp::contains_btn(b"btn 2", 2));
        // Semantique historique substring : "btn 22" contient "btn 2".
        assert!(MacroForgeApp::contains_btn(b"btn 22", 2));
        assert!(!MacroForgeApp::contains_btn(b"btn 3", 2));
        assert!(MacroForgeApp::contains_hex_u16(b"1b", 0x1B));
        assert!(MacroForgeApp::contains_hex_u16(b"vk: ff", 0xFF));
        assert!(!MacroForgeApp::contains_hex_u16(b"1c", 0x1B));
    }
}
