use crate::ui::i18n::Language;
use crate::ui::theme::{colors, toolbar_frame};
use eframe::egui::{
    self, Color32, Margin, Pos2, Rect, Rounding, Stroke, Vec2, ViewportBuilder, ViewportId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarAction {
    None,
    ToggleRecord,
    TogglePlay,
    EmergencyStop,
    OpenMainWindow,
    CloseToolbar,
    DetachTargetWindow,
}

pub struct FloatingToolbar {
    pub is_visible: bool,
    pub current_action_idx: usize,
    pub total_actions: usize,
    pub action_detail: String,
}

impl Default for FloatingToolbar {
    fn default() -> Self {
        Self::new()
    }
}

impl FloatingToolbar {
    pub fn new() -> Self {
        Self {
            is_visible: false,
            current_action_idx: 0,
            total_actions: 0,
            action_detail: String::new(),
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        is_recording: bool,
        is_playing: bool,
        is_embedded: bool,
        lang: Language,
    ) -> ToolbarAction {
        if !self.is_visible {
            return ToolbarAction::None;
        }

        let mut triggered_action = ToolbarAction::None;
        let viewport_id = ViewportId::from_hash_of("macroforge_floating_toolbar");
        let tb_width = if is_embedded { 336.0 } else { 300.0 };

        ctx.show_viewport_immediate(
            viewport_id,
            ViewportBuilder::default()
                .with_title("MacroForge Toolbar")
                .with_inner_size([tb_width, 44.0])
                .with_min_inner_size([260.0, 40.0])
                .with_decorations(false)
                .with_transparent(true)
                .with_always_on_top()
                .with_resizable(false),
            |ctx, _class| {
                egui::CentralPanel::default()
                    .frame(toolbar_frame())
                    .show(ctx, |ui| {
                        // Permettre le déplacement sur toute la surface de fond non occupée
                        let panel_rect = ui.max_rect();
                        let bg_id = ui.id().with("tb_bg_drag");
                        let bg_resp = ui.interact(panel_rect, bg_id, egui::Sense::click_and_drag());
                        if bg_resp.drag_started() || bg_resp.dragged() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                        }

                        ui.horizontal_centered(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(6.0, 0.0);

                            // 1. Poignée de déplacement tactile (Drag Handle)
                            let handle_size = egui::vec2(16.0, 26.0);
                            let (handle_rect, handle_resp) =
                                ui.allocate_exact_size(handle_size, egui::Sense::click_and_drag());

                            if handle_resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                            }
                            if handle_resp.drag_started() || handle_resp.dragged() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                            }

                            // Dessin des 6 points matriciels anti-aliasés
                            let dot_color = if handle_resp.hovered() || handle_resp.dragged() {
                                colors::ACCENT_PRIMARY_HOVER
                            } else {
                                Color32::from_rgba_premultiplied(148, 163, 184, 150)
                            };
                            let center = handle_rect.center();
                            for dx in [-3.0_f32, 3.0_f32] {
                                for dy in [-6.0_f32, 0.0_f32, 6.0_f32] {
                                    ui.painter().circle_filled(
                                        Pos2::new(center.x + dx, center.y + dy),
                                        1.4_f32,
                                        dot_color,
                                    );
                                }
                            }
                            handle_resp.on_hover_text(lang.toolbar_drag_tip());

                            ui.add_space(2.0);

                            // 2. Bouton Enregistrer (Record / Stop Record)
                            let btn_size = egui::vec2(28.0, 28.0);
                            if !is_recording {
                                let resp = render_record_btn(
                                    ui,
                                    false,
                                    lang.toolbar_rec_start_tip(),
                                    btn_size,
                                );
                                if resp.clicked() {
                                    triggered_action = ToolbarAction::ToggleRecord;
                                }
                            } else {
                                let resp = render_record_btn(
                                    ui,
                                    true,
                                    lang.toolbar_rec_stop_tip(),
                                    btn_size,
                                );
                                if resp.clicked() {
                                    triggered_action = ToolbarAction::ToggleRecord;
                                }
                            }

                            // 3. Bouton Jouer / Arrêt d'urgence (Play / Stop Playback)
                            if !is_playing {
                                let resp =
                                    render_play_btn(ui, false, lang.toolbar_play_tip(), btn_size);
                                if resp.clicked() {
                                    triggered_action = ToolbarAction::TogglePlay;
                                }
                            } else {
                                let resp = render_play_btn(
                                    ui,
                                    true,
                                    lang.toolbar_stop_playback_tip(),
                                    btn_size,
                                );
                                if resp.clicked() {
                                    triggered_action = ToolbarAction::EmergencyStop;
                                }
                            }

                            ui.add_space(3.0);

                            // 4. Capsule d'état centrale (Status Pill)
                            let status_frame = egui::Frame::none()
                                .fill(Color32::from_rgba_premultiplied(10, 16, 28, 200))
                                .stroke(Stroke::new(
                                    1.0_f32,
                                    Color32::from_rgba_premultiplied(255, 255, 255, 18),
                                ))
                                .rounding(Rounding::same(6.0_f32))
                                .inner_margin(Margin::symmetric(8.0, 3.0));

                            status_frame.show(ui, |ui| {
                                if is_playing {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "▶ {}/{}",
                                            self.current_action_idx, self.total_actions
                                        ))
                                        .color(colors::ACCENT_CYAN_HOVER)
                                        .size(11.0)
                                        .strong(),
                                    );
                                } else if is_recording {
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing = egui::vec2(3.0, 0.0);
                                        ui.painter().circle_filled(
                                            Pos2::new(
                                                ui.cursor().min.x + 3.0_f32,
                                                ui.cursor().center().y + 1.0_f32,
                                            ),
                                            3.0_f32,
                                            Color32::from_rgb(239, 68, 68),
                                        );
                                        ui.add_space(8.0);
                                        ui.label(
                                            egui::RichText::new("REC")
                                                .color(Color32::from_rgb(239, 68, 68))
                                                .size(11.0)
                                                .strong(),
                                        );
                                    });
                                } else {
                                    ui.label(
                                        egui::RichText::new(
                                            lang.toolbar_actions_count(self.total_actions),
                                        )
                                        .color(colors::TEXT_SECONDARY)
                                        .size(11.0),
                                    );
                                }
                            });

                            // 5. Boutons de contrôle de la fenêtre à droite (Ouvrir éditeur & Fermer & Détacher)
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let win_btn_size = egui::vec2(24.0, 24.0);

                                    // Bouton Fermer
                                    let close_resp = render_window_btn(
                                        ui,
                                        WindowBtnType::Close,
                                        lang.toolbar_close_tip(),
                                        win_btn_size,
                                    );
                                    if close_resp.clicked() {
                                        triggered_action = ToolbarAction::CloseToolbar;
                                    }

                                    ui.add_space(2.0);

                                    // Bouton Ouvrir Éditeur
                                    let edit_resp = render_window_btn(
                                        ui,
                                        WindowBtnType::OpenEditor,
                                        lang.toolbar_open_editor_tip(),
                                        win_btn_size,
                                    );
                                    if edit_resp.clicked() {
                                        triggered_action = ToolbarAction::OpenMainWindow;
                                    }

                                    if is_embedded {
                                        ui.add_space(2.0);
                                        let detach_resp = render_window_btn(
                                            ui,
                                            WindowBtnType::Detach,
                                            lang.toolbar_detach_tip(),
                                            win_btn_size,
                                        );
                                        if detach_resp.clicked() {
                                            triggered_action = ToolbarAction::DetachTargetWindow;
                                        }
                                    }
                                },
                            );
                        });
                    });
            },
        );

        triggered_action
    }
}

/// Rendu vectoriel du bouton Enregistrer avec centrage au pixel
fn render_record_btn(
    ui: &mut egui::Ui,
    is_recording: bool,
    tooltip: &str,
    size: Vec2,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let is_hovered = resp.hovered();
    let is_clicked = resp.is_pointer_button_down_on();

    let (bg_fill, border_stroke) = if is_recording {
        // Enregistrement actif : fond rubis pulsant et lueur vive
        if is_clicked {
            (
                Color32::from_rgb(185, 28, 28),
                Stroke::new(1.5_f32, colors::ACCENT_DANGER_HOVER),
            )
        } else if is_hovered {
            (
                Color32::from_rgb(220, 38, 38),
                Stroke::new(1.5_f32, colors::ACCENT_DANGER_HOVER),
            )
        } else {
            (
                Color32::from_rgba_premultiplied(185, 28, 28, 230),
                Stroke::new(1.5_f32, colors::ACCENT_DANGER),
            )
        }
    } else {
        // En veille : fond sombre rubis et contour discret
        if is_clicked {
            (
                Color32::from_rgb(153, 27, 27),
                Stroke::new(1.5_f32, colors::ACCENT_DANGER),
            )
        } else if is_hovered {
            (
                Color32::from_rgba_premultiplied(185, 28, 28, 160),
                Stroke::new(1.5_f32, colors::ACCENT_DANGER_HOVER),
            )
        } else {
            (
                Color32::from_rgba_premultiplied(127, 29, 29, 140),
                Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(239, 68, 68, 180)),
            )
        }
    };

    let rounding = Rounding::same(7.0_f32);
    ui.painter().rect(rect, rounding, bg_fill, border_stroke);

    let center = rect.center();
    if is_recording {
        // Carré d'arrêt blanc (Stop record)
        let sq_size = Vec2::splat(9.0_f32);
        let sq_rect = Rect::from_center_size(center, sq_size);
        ui.painter()
            .rect_filled(sq_rect, Rounding::same(1.5_f32), Color32::WHITE);
    } else {
        // Point rouge lumineux
        let dot_color = if is_hovered {
            Color32::from_rgb(255, 255, 255)
        } else {
            Color32::from_rgb(239, 68, 68)
        };
        ui.painter().circle_filled(center, 4.5_f32, dot_color);
    }

    resp.on_hover_text(tooltip)
}

/// Rendu vectoriel du bouton Rejouer / Arrêt Urgence avec centrage géométrique parfait
fn render_play_btn(
    ui: &mut egui::Ui,
    is_playing: bool,
    tooltip: &str,
    size: Vec2,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let is_hovered = resp.hovered();
    let is_clicked = resp.is_pointer_button_down_on();

    let (bg_fill, border_stroke) = if is_playing {
        // Lecture en cours : bouton ambre/warning pour arrêt d'urgence F4
        if is_clicked {
            (
                Color32::from_rgb(180, 83, 9),
                Stroke::new(1.5_f32, colors::ACCENT_WARNING_HOVER),
            )
        } else if is_hovered {
            (
                Color32::from_rgb(217, 119, 6),
                Stroke::new(1.5_f32, colors::ACCENT_WARNING_HOVER),
            )
        } else {
            (
                Color32::from_rgba_premultiplied(180, 83, 9, 230),
                Stroke::new(1.5_f32, colors::ACCENT_WARNING),
            )
        }
    } else {
        // En veille : vert émeraude glass
        if is_clicked {
            (
                Color32::from_rgb(4, 120, 87),
                Stroke::new(1.5_f32, colors::ACCENT_SUCCESS_HOVER),
            )
        } else if is_hovered {
            (
                Color32::from_rgba_premultiplied(5, 150, 105, 180),
                Stroke::new(1.5_f32, colors::ACCENT_SUCCESS_HOVER),
            )
        } else {
            (
                Color32::from_rgba_premultiplied(6, 95, 70, 160),
                Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(16, 185, 129, 180)),
            )
        }
    };

    let rounding = Rounding::same(7.0_f32);
    ui.painter().rect(rect, rounding, bg_fill, border_stroke);

    let center = rect.center();
    if is_playing {
        // Carré d'arrêt blanc (Stop playback)
        let sq_size = Vec2::splat(9.0_f32);
        let sq_rect = Rect::from_center_size(center, sq_size);
        ui.painter()
            .rect_filled(sq_rect, Rounding::same(1.5_f32), Color32::WHITE);
    } else {
        // Polygone triangle blanc centré
        let tri_points = vec![
            Pos2::new(center.x - 3.5_f32, center.y - 5.0_f32),
            Pos2::new(center.x + 5.0_f32, center.y),
            Pos2::new(center.x - 3.5_f32, center.y + 5.0_f32),
        ];
        ui.painter().add(egui::Shape::convex_polygon(
            tri_points,
            Color32::WHITE,
            Stroke::NONE,
        ));
    }

    resp.on_hover_text(tooltip)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowBtnType {
    OpenEditor,
    Close,
    Detach,
}

/// Rendu des boutons de gestion de fenêtre (Ouvrir éditeur / Fermer toolbar / Détacher fenêtre cible)
fn render_window_btn(
    ui: &mut egui::Ui,
    btn_type: WindowBtnType,
    tooltip: &str,
    size: Vec2,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let is_hovered = resp.hovered();
    let is_clicked = resp.is_pointer_button_down_on();

    let (bg_fill, border_stroke, icon_color) = match btn_type {
        WindowBtnType::OpenEditor => {
            if is_clicked {
                (
                    Color32::from_rgba_unmultiplied(59, 130, 246, 75),
                    Stroke::new(1.0_f32, colors::ACCENT_PRIMARY),
                    Color32::WHITE,
                )
            } else if is_hovered {
                (
                    Color32::from_rgba_unmultiplied(59, 130, 246, 40),
                    Stroke::new(1.0_f32, colors::BORDER_HOVER),
                    Color32::WHITE,
                )
            } else {
                (
                    Color32::from_rgba_unmultiplied(255, 255, 255, 12),
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 25)),
                    colors::TEXT_SECONDARY,
                )
            }
        }
        WindowBtnType::Close => {
            if is_clicked {
                (
                    Color32::from_rgba_unmultiplied(185, 28, 28, 220),
                    Stroke::new(1.0_f32, colors::ACCENT_DANGER_HOVER),
                    Color32::WHITE,
                )
            } else if is_hovered {
                (
                    Color32::from_rgba_unmultiplied(220, 38, 38, 160),
                    Stroke::new(1.0_f32, colors::ACCENT_DANGER),
                    Color32::WHITE,
                )
            } else {
                (
                    Color32::from_rgba_unmultiplied(255, 255, 255, 12),
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 25)),
                    colors::TEXT_MUTED,
                )
            }
        }
        WindowBtnType::Detach => {
            if is_clicked {
                (
                    Color32::from_rgba_unmultiplied(245, 158, 11, 200),
                    Stroke::new(1.0_f32, colors::ACCENT_WARNING),
                    Color32::WHITE,
                )
            } else if is_hovered {
                (
                    Color32::from_rgba_unmultiplied(245, 158, 11, 90),
                    Stroke::new(1.0_f32, colors::ACCENT_WARNING),
                    Color32::WHITE,
                )
            } else {
                (
                    Color32::from_rgba_unmultiplied(255, 255, 255, 12),
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 25)),
                    colors::ACCENT_WARNING,
                )
            }
        }
    };

    let rounding = Rounding::same(6.0_f32);
    ui.painter().rect(rect, rounding, bg_fill, border_stroke);

    let center = rect.center();
    match btn_type {
        WindowBtnType::OpenEditor => {
            // Icône fenêtre vectorielle nette (deux rectangles imbriqués / fenêtre épurée)
            let win_rect = Rect::from_center_size(center, Vec2::new(10.0_f32, 9.0_f32));
            ui.painter().rect_stroke(
                win_rect,
                Rounding::same(1.5_f32),
                Stroke::new(1.2_f32, icon_color),
            );
            ui.painter().line_segment(
                [
                    Pos2::new(win_rect.min.x, win_rect.min.y + 2.5_f32),
                    Pos2::new(win_rect.max.x, win_rect.min.y + 2.5_f32),
                ],
                Stroke::new(1.2_f32, icon_color),
            );
        }
        WindowBtnType::Close => {
            // Croix vectorielle ✕ parfaitement centrée
            let s = 3.5_f32;
            ui.painter().line_segment(
                [
                    Pos2::new(center.x - s, center.y - s),
                    Pos2::new(center.x + s, center.y + s),
                ],
                Stroke::new(1.4_f32, icon_color),
            );
            ui.painter().line_segment(
                [
                    Pos2::new(center.x + s, center.y - s),
                    Pos2::new(center.x - s, center.y + s),
                ],
                Stroke::new(1.4_f32, icon_color),
            );
        }
        WindowBtnType::Detach => {
            // Icône détachement / cadenas ouvert épuré
            ui.painter().text(
                center,
                egui::Align2::CENTER_CENTER,
                "🔓",
                egui::FontId::proportional(11.0),
                icon_color,
            );
        }
    }

    resp.on_hover_text(tooltip)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolbar_initial_state() {
        let mut toolbar = FloatingToolbar::new();
        assert!(!toolbar.is_visible);
        assert_eq!(toolbar.current_action_idx, 0);
        assert_eq!(toolbar.total_actions, 0);
        assert!(toolbar.action_detail.is_empty());

        toolbar.is_visible = true;
        toolbar.total_actions = 15;
        toolbar.current_action_idx = 3;
        toolbar.action_detail = "Test Action".to_string();

        assert!(toolbar.is_visible);
        assert_eq!(toolbar.total_actions, 15);
        assert_eq!(toolbar.current_action_idx, 3);
        assert_eq!(toolbar.action_detail, "Test Action");
    }

    #[test]
    fn test_toolbar_actions_variants() {
        assert_eq!(ToolbarAction::None, ToolbarAction::None);
        assert_ne!(ToolbarAction::ToggleRecord, ToolbarAction::TogglePlay);
        assert_ne!(ToolbarAction::EmergencyStop, ToolbarAction::CloseToolbar);
        assert_ne!(ToolbarAction::OpenMainWindow, ToolbarAction::None);
        assert_ne!(ToolbarAction::DetachTargetWindow, ToolbarAction::None);
    }
}
