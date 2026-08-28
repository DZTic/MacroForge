use crate::macro_core::{self, WindowInfo, WindowLockConfig};
use crate::ui::i18n::Language;
use crate::ui::theme::{self, colors};
use crate::ui::widgets::{ButtonVariant, GlassButton};
use eframe::egui::{self, DragValue, Frame, Margin, Rounding, Stroke, Vec2};

pub struct WindowLockModal {
    pub is_open: bool,
    pub enabled: bool,
    pub title_filter: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub force_foreground: bool,
    pub restore_if_maximized: bool,

    // Cache des fenêtres détectées pour le sélecteur
    pub detected_windows: Vec<WindowInfo>,
    pub selected_window_idx: Option<usize>,

    // Message de résultat du test en direct
    pub test_status: Option<(bool, String)>,
}

impl Default for WindowLockModal {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowLockModal {
    pub fn new() -> Self {
        let current_cfg = macro_core::get_window_lock();
        Self {
            is_open: false,
            enabled: current_cfg.enabled,
            title_filter: current_cfg.title_filter,
            x: current_cfg.x,
            y: current_cfg.y,
            width: current_cfg.width,
            height: current_cfg.height,
            force_foreground: current_cfg.force_foreground,
            restore_if_maximized: current_cfg.restore_if_maximized,
            detected_windows: Vec::new(),
            selected_window_idx: None,
            test_status: None,
        }
    }

    pub fn open(&mut self) {
        let current_cfg = macro_core::get_window_lock();
        self.enabled = current_cfg.enabled;
        self.title_filter = current_cfg.title_filter;
        self.x = current_cfg.x;
        self.y = current_cfg.y;
        self.width = current_cfg.width;
        self.height = current_cfg.height;
        self.force_foreground = current_cfg.force_foreground;
        self.restore_if_maximized = current_cfg.restore_if_maximized;
        self.test_status = None;
        self.refresh_windows();
        self.is_open = true;
    }

    pub fn refresh_windows(&mut self) {
        self.detected_windows = macro_core::list_open_windows();
        self.selected_window_idx = None;

        // Si un titre de filtre existe déjà, essayer de présélectionner la fenêtre correspondante
        if !self.title_filter.trim().is_empty() {
            let filter_lower = self.title_filter.trim().to_lowercase();
            for (idx, w) in self.detected_windows.iter().enumerate() {
                if w.title.to_lowercase().contains(&filter_lower) {
                    self.selected_window_idx = Some(idx);
                    break;
                }
            }
        }
    }

    pub fn center_on_screen(&mut self) {
        let (screen_w, screen_h) = macro_core::get_primary_screen_dimensions();
        if screen_w > 0 && screen_h > 0 {
            self.x = ((screen_w - self.width) / 2).max(0);
            self.y = ((screen_h - self.height) / 2).max(0);
        }
    }

    pub fn capture_active_window(&mut self) -> bool {
        if let Some(info) = macro_core::capture_active_window_info() {
            self.title_filter = info.title.clone();
            self.x = info.x;
            self.y = info.y;
            self.width = info.width;
            self.height = info.height;
            self.refresh_windows();
            true
        } else {
            false
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, lang: Language) -> bool {
        if !self.is_open {
            return false;
        }

        let mut should_close = false;
        let mut changed = false;

        egui::Window::new(lang.window_lock_modal_title())
            .frame(theme::modal_frame())
            .collapsible(false)
            .resizable(false)
            .default_size(Vec2::new(560.0, 480.0))
            .anchor(egui::Align2::CENTER_CENTER, Vec2::new(0.0, 0.0))
            .show(ctx, |ui| {
                ui.add_space(2.0);

                // --- 1. Activation globale ---
                Frame::none()
                    .fill(colors::BG_CARD)
                    .stroke(Stroke::new(1.0_f32, colors::BORDER_CARD))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.checkbox(&mut self.enabled, lang.window_lock_enable());
                    });

                ui.add_space(8.0);

                // --- 2. Sélection de la Fenêtre Cible ---
                Frame::none()
                    .fill(colors::BG_CARD)
                    .stroke(Stroke::new(1.0_f32, colors::BORDER_CARD))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(lang.target_window_section())
                                .color(colors::TEXT_PRIMARY)
                                .strong()
                                .size(13.5),
                        );
                        ui.add_space(6.0);

                        // Sélecteur de fenêtres ouvertes
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(lang.open_windows_list_label())
                                    .color(colors::TEXT_SECONDARY)
                                    .size(12.5),
                            );

                            let current_label = if let Some(idx) = self.selected_window_idx {
                                if let Some(w) = self.detected_windows.get(idx) {
                                    let preview = if w.title.len() > 30 {
                                        format!("{}...", &w.title[..30])
                                    } else {
                                        w.title.clone()
                                    };
                                    format!("{} ({}×{})", preview, w.width, w.height)
                                } else {
                                    "---".to_string()
                                }
                            } else if self.title_filter.trim().is_empty() {
                                "Active (Automatique)".to_string()
                            } else {
                                format!("Filtre: {}", self.title_filter)
                            };

                            egui::ComboBox::from_id_salt("target_window_combo")
                                .selected_text(current_label)
                                .width(240.0)
                                .show_ui(ui, |ui| {
                                    if ui
                                        .selectable_label(
                                            self.selected_window_idx.is_none()
                                                && self.title_filter.is_empty(),
                                            "🎯 Dernière fenêtre active (Auto)",
                                        )
                                        .clicked()
                                    {
                                        self.selected_window_idx = None;
                                        self.title_filter.clear();
                                    }

                                    ui.separator();

                                    for (idx, w) in self.detected_windows.iter().enumerate() {
                                        let display = if w.title.len() > 40 {
                                            format!("{}...", &w.title[..40])
                                        } else {
                                            w.title.clone()
                                        };
                                        let is_selected = self.selected_window_idx == Some(idx);
                                        if ui
                                            .selectable_label(
                                                is_selected,
                                                format!("{} ({}×{})", display, w.width, w.height),
                                            )
                                            .clicked()
                                        {
                                            self.selected_window_idx = Some(idx);
                                            self.title_filter = w.title.clone();
                                            // Proposer d'adapter directement les dimensions
                                            self.x = w.x;
                                            self.y = w.y;
                                            self.width = w.width;
                                            self.height = w.height;
                                        }
                                    }
                                });

                            let refresh_btn = GlassButton::new(lang.refresh_windows_list_btn())
                                .icon("🔄")
                                .compact(true)
                                .variant(ButtonVariant::Secondary);
                            if ui.add(refresh_btn).clicked() {
                                self.refresh_windows();
                            }
                        });

                        ui.add_space(6.0);

                        // Filtre de titre et capture
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(lang.target_window_title_filter())
                                    .color(colors::TEXT_SECONDARY)
                                    .size(12.5),
                            );
                            ui.text_edit_singleline(&mut self.title_filter);

                            let capture_btn = GlassButton::new(lang.detect_active_window_btn())
                                .icon("🎯")
                                .compact(true)
                                .variant(ButtonVariant::Primary);
                            if ui.add(capture_btn).clicked() {
                                self.capture_active_window();
                            }
                        });
                    });

                ui.add_space(8.0);

                // --- 3. Dimensions & Position Cibles ---
                Frame::none()
                    .fill(colors::BG_CARD)
                    .stroke(Stroke::new(1.0_f32, colors::BORDER_CARD))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(lang.dimensions_section())
                                .color(colors::TEXT_PRIMARY)
                                .strong()
                                .size(13.5),
                        );
                        ui.add_space(6.0);

                        // Grille Largeur / Hauteur / X / Y
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(lang.width_label())
                                    .color(colors::TEXT_PRIMARY)
                                    .size(13.0),
                            );
                            ui.add(
                                DragValue::new(&mut self.width)
                                    .range(100..=7680)
                                    .speed(5.0)
                                    .suffix(" px"),
                            );

                            ui.add_space(8.0);

                            ui.label(
                                egui::RichText::new(lang.height_label())
                                    .color(colors::TEXT_PRIMARY)
                                    .size(13.0),
                            );
                            ui.add(
                                DragValue::new(&mut self.height)
                                    .range(100..=4320)
                                    .speed(5.0)
                                    .suffix(" px"),
                            );

                            ui.add_space(12.0);

                            ui.label(
                                egui::RichText::new(lang.pos_x_label())
                                    .color(colors::TEXT_PRIMARY)
                                    .size(13.0),
                            );
                            ui.add(
                                DragValue::new(&mut self.x)
                                    .range(-3840..=7680)
                                    .speed(2.0)
                                    .suffix(" px"),
                            );

                            ui.add_space(8.0);

                            ui.label(
                                egui::RichText::new(lang.pos_y_label())
                                    .color(colors::TEXT_PRIMARY)
                                    .size(13.0),
                            );
                            ui.add(
                                DragValue::new(&mut self.y)
                                    .range(-2160..=4320)
                                    .speed(2.0)
                                    .suffix(" px"),
                            );
                        });

                        ui.add_space(8.0);

                        // Préréglages de résolution
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(lang.presets_label())
                                    .color(colors::TEXT_SECONDARY)
                                    .size(12.5),
                            );

                            let presets = [
                                (1280, 720, "1280×720 (720p)"),
                                (1920, 1080, "1920×1080 (1080p)"),
                                (1600, 900, "1600×900"),
                                (1024, 768, "1024×768"),
                                (800, 600, "800×600"),
                            ];

                            for &(pw, ph, plbl) in &presets {
                                let pbtn = GlassButton::new(plbl)
                                    .compact(true)
                                    .variant(ButtonVariant::Secondary);
                                if ui.add(pbtn).clicked() {
                                    self.width = pw;
                                    self.height = ph;
                                }
                            }
                        });

                        ui.add_space(6.0);

                        // Boutons d'aide au positionnement
                        ui.horizontal(|ui| {
                            let center_btn = GlassButton::new(lang.center_screen_btn())
                                .icon("🖥")
                                .compact(true)
                                .variant(ButtonVariant::Secondary);
                            if ui.add(center_btn).clicked() {
                                self.center_on_screen();
                            }

                            let cap_size_btn = GlassButton::new(lang.capture_current_size_btn())
                                .icon("📐")
                                .compact(true)
                                .variant(ButtonVariant::Secondary);
                            if ui.add(cap_size_btn).clicked() {
                                if let Some(info) = macro_core::capture_active_window_info() {
                                    self.x = info.x;
                                    self.y = info.y;
                                    self.width = info.width;
                                    self.height = info.height;
                                }
                            }
                        });
                    });

                ui.add_space(8.0);

                // --- 4. Options avancées ---
                Frame::none()
                    .fill(colors::BG_CARD)
                    .stroke(Stroke::new(1.0_f32, colors::BORDER_CARD))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.checkbox(&mut self.force_foreground, lang.force_foreground_label());
                        ui.add_space(4.0);
                        ui.checkbox(
                            &mut self.restore_if_maximized,
                            lang.restore_maximized_label(),
                        );
                    });

                ui.add_space(10.0);

                // --- 5. Barre de test et boutons d'actions ---
                ui.horizontal(|ui| {
                    let test_btn = GlassButton::new(lang.test_window_lock_btn())
                        .icon("🧪")
                        .variant(ButtonVariant::Secondary);
                    if ui.add(test_btn).clicked() {
                        let test_cfg = WindowLockConfig {
                            enabled: true,
                            title_filter: self.title_filter.clone(),
                            x: self.x,
                            y: self.y,
                            width: self.width,
                            height: self.height,
                            force_foreground: self.force_foreground,
                            restore_if_maximized: self.restore_if_maximized,
                        };
                        match macro_core::apply_window_lock(&test_cfg) {
                            Ok(()) => {
                                self.test_status =
                                    Some((true, lang.window_lock_success_test().to_string()));
                            }
                            Err(err) => {
                                self.test_status = Some((
                                    false,
                                    format!("{} ({})", lang.window_lock_error_test(), err),
                                ));
                            }
                        }
                    }

                    if let Some((success, ref msg)) = self.test_status {
                        let color = if success {
                            colors::ACCENT_SUCCESS
                        } else {
                            colors::ACCENT_DANGER
                        };
                        ui.label(egui::RichText::new(msg).color(color).size(12.0));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let save_btn = GlassButton::new(lang.modal_save())
                            .icon("💾")
                            .variant(ButtonVariant::Success);
                        if ui.add(save_btn).clicked() {
                            let new_cfg = WindowLockConfig {
                                enabled: self.enabled,
                                title_filter: self.title_filter.trim().to_string(),
                                x: self.x,
                                y: self.y,
                                width: self.width,
                                height: self.height,
                                force_foreground: self.force_foreground,
                                restore_if_maximized: self.restore_if_maximized,
                            };
                            macro_core::set_window_lock(new_cfg.clone());

                            // Sauvegarder dans les paramètres globaux
                            let mut settings = crate::ui::i18n::AppSettings::load();
                            settings.window_lock = new_cfg;
                            settings.save();

                            changed = true;
                            should_close = true;
                        }

                        ui.add_space(8.0);

                        let cancel_btn =
                            GlassButton::new(lang.modal_cancel()).variant(ButtonVariant::Ghost);
                        if ui.add(cancel_btn).clicked() {
                            should_close = true;
                        }
                    });
                });
            });

        if should_close {
            self.is_open = false;
        }

        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_lock_modal_open_close() {
        let mut modal = WindowLockModal::new();
        assert!(!modal.is_open);
        modal.open();
        assert!(modal.is_open);
    }
}
