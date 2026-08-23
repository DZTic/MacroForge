use crate::ui::i18n::Language;
use crate::ui::theme::colors;
use crate::ui::widgets::{ButtonVariant, GlassButton};
use eframe::egui::{self, Color32, Frame, Margin, Rounding, Stroke, ViewportBuilder, ViewportId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarAction {
    None,
    ToggleRecord,
    TogglePlay,
    EmergencyStop,
    OpenMainWindow,
    CloseToolbar,
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
        _lang: Language,
    ) -> ToolbarAction {
        if !self.is_visible {
            return ToolbarAction::None;
        }

        let mut triggered_action = ToolbarAction::None;
        let viewport_id = ViewportId::from_hash_of("macroforge_floating_toolbar");

        ctx.show_viewport_immediate(
            viewport_id,
            ViewportBuilder::default()
                .with_title("MacroForge Toolbar")
                .with_inner_size([310.0, 54.0])
                .with_min_inner_size([280.0, 48.0])
                .with_decorations(false)
                .with_transparent(true)
                .with_always_on_top()
                .with_resizable(false),
            |ctx, _class| {
                let dark_glass_frame = Frame::none()
                    .fill(colors::BG_PANEL)
                    .stroke(Stroke::new(1.5_f32, colors::ACCENT_PRIMARY))
                    .rounding(Rounding::same(10.0))
                    .inner_margin(Margin::symmetric(10.0, 6.0));

                egui::CentralPanel::default()
                    .frame(dark_glass_frame)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            // 1. Poignée de déplacement (Drag Handle)
                            let drag_resp = ui
                                .scope(|ui| {
                                    ui.label(
                                        egui::RichText::new("⣿")
                                            .color(colors::TEXT_MUTED)
                                            .size(16.0),
                                    );
                                })
                                .response
                                .interact(egui::Sense::drag());

                            if drag_resp.dragged() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                            }

                            ui.add_space(2.0);

                            // 2. Bouton Enregistrer / Arrêter enregistrement
                            if !is_recording {
                                let rec_btn = GlassButton::new("")
                                    .icon("🔴")
                                    .compact(true)
                                    .variant(ButtonVariant::Danger);
                                if ui.add(rec_btn).on_hover_text("Enregistrer (F8)").clicked() {
                                    triggered_action = ToolbarAction::ToggleRecord;
                                }
                            } else {
                                let stop_btn = GlassButton::new("")
                                    .icon("⏹")
                                    .compact(true)
                                    .variant(ButtonVariant::Secondary);
                                if ui
                                    .add(stop_btn)
                                    .on_hover_text("Arrêter Enregistrement (F9)")
                                    .clicked()
                                {
                                    triggered_action = ToolbarAction::ToggleRecord;
                                }
                            }

                            // 3. Bouton Jouer / Arrêt d'urgence
                            if !is_playing {
                                let play_btn = GlassButton::new("")
                                    .icon("▶")
                                    .compact(true)
                                    .variant(ButtonVariant::Success);
                                if ui.add(play_btn).on_hover_text("Rejouer (F4)").clicked() {
                                    triggered_action = ToolbarAction::TogglePlay;
                                }
                            } else {
                                let stop_btn = GlassButton::new("")
                                    .icon("🛑")
                                    .compact(true)
                                    .variant(ButtonVariant::Warning);
                                if ui
                                    .add(stop_btn)
                                    .on_hover_text("Arrêt Urgence (F4)")
                                    .clicked()
                                {
                                    triggered_action = ToolbarAction::EmergencyStop;
                                }
                            }

                            ui.add_space(4.0);

                            // 4. Indicateur de progression / état
                            if is_playing {
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "▶ {}/{}",
                                            self.current_action_idx, self.total_actions
                                        ))
                                        .color(colors::ACCENT_PRIMARY_HOVER)
                                        .size(11.0)
                                        .strong(),
                                    );
                                });
                            } else if is_recording {
                                ui.label(
                                    egui::RichText::new("REC 🔴")
                                        .color(Color32::from_rgb(239, 68, 68))
                                        .size(11.0)
                                        .strong(),
                                );
                            } else {
                                ui.label(
                                    egui::RichText::new(format!("{} act.", self.total_actions))
                                        .color(colors::TEXT_MUTED)
                                        .size(11.0),
                                );
                            }

                            // 5. Boutons de navigation à droite
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .small_button("✕")
                                        .on_hover_text("Fermer la toolbar")
                                        .clicked()
                                    {
                                        triggered_action = ToolbarAction::CloseToolbar;
                                    }

                                    let edit_btn = GlassButton::new("")
                                        .icon("🗖")
                                        .variant(ButtonVariant::Ghost);
                                    if ui
                                        .add(edit_btn)
                                        .on_hover_text("Ouvrir la fenêtre principale")
                                        .clicked()
                                    {
                                        triggered_action = ToolbarAction::OpenMainWindow;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolbar_initial_state() {
        let toolbar = FloatingToolbar::new();
        assert!(!toolbar.is_visible);
        assert_eq!(toolbar.current_action_idx, 0);
        assert_eq!(toolbar.total_actions, 0);
    }
}
