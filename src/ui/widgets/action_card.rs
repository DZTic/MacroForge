use crate::macro_core::{ActionType, MacroAction};
use crate::ui::theme::colors;
use eframe::egui::{self, Color32, Frame, Margin, Response, Rounding, Stroke, Ui, Widget};

/// Carte d'action moderne dans la timeline de la macro (Glassmorphism Action Card)
pub struct ActionCard<'a> {
    index: usize,
    action: &'a MacroAction,
    selected: bool,
}

impl<'a> ActionCard<'a> {
    pub fn new(index: usize, action: &'a MacroAction) -> Self {
        Self {
            index,
            action,
            selected: false,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

impl<'a> Widget for ActionCard<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (icon, type_label, type_color, detail_str) = match &self.action.action_type {
            ActionType::KeyPress(name, vk, _) => (
                "⌨️",
                "Touche Pressée",
                colors::ACCENT_PRIMARY,
                format!("{} (VK: {:#04X})", name, vk),
            ),
            ActionType::KeyRelease(name, vk, _) => (
                "⌨️",
                "Touche Relâchée",
                colors::TEXT_SECONDARY,
                format!("{} (VK: {:#04X})", name, vk),
            ),
            ActionType::MouseMove(x, y) => (
                "🖱️",
                "Position Souris",
                colors::ACCENT_CYAN,
                format!("X: {:.0}, Y: {:.0}", x, y),
            ),
            ActionType::MouseMoveRelative(dx, dy) => (
                "🖱️",
                "Mouvement Relatif",
                colors::ACCENT_WARNING,
                format!("ΔX: {}, ΔY: {}", dx, dy),
            ),
            ActionType::MousePress(btn, x, y) => (
                "🖱️",
                "Clic Pressé",
                colors::ACCENT_SUCCESS,
                format!("Bouton {} à ({:.0}, {:.0})", btn, x, y),
            ),
            ActionType::MouseRelease(btn, x, y) => (
                "🖱️",
                "Clic Relâché",
                colors::TEXT_SECONDARY,
                format!("Bouton {} à ({:.0}, {:.0})", btn, x, y),
            ),
            ActionType::Scroll(dx, dy) => (
                "📜",
                "Molette Défilement",
                colors::ACCENT_WARNING_HOVER,
                format!("ΔX: {:.1}, ΔY: {:.1}", dx, dy),
            ),
            ActionType::Wait(ms) => (
                "⏱️",
                "Pause",
                colors::ACCENT_WARNING,
                format!("Attente de {} ms", ms),
            ),
            ActionType::WaitImage(path, timeout) => (
                "🖼️",
                "Détection Image",
                colors::ACCENT_PURPLE,
                format!("Fichier: {} (timeout: {}ms)", path, timeout),
            ),
        };

        let card_frame = Frame::none()
            .fill(if self.selected {
                colors::BG_CARD_ACTIVE
            } else {
                colors::BG_CARD
            })
            .stroke(if self.selected {
                Stroke::new(1.5_f32, colors::ACCENT_PRIMARY)
            } else {
                Stroke::new(1.0_f32, colors::BORDER_SUBTLE)
            })
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::symmetric(10.0, 7.0));

        card_frame
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Badge numéro d'index (#001)
                    let index_text = format!("#{:03}", self.index + 1);
                    ui.label(
                        egui::RichText::new(index_text)
                            .monospace()
                            .color(colors::TEXT_MUTED)
                            .size(11.0),
                    );

                    ui.add_space(4.0);

                    // Badge de catégorie d'action avec pastille / fond teinté
                    let type_badge_frame = Frame::none()
                        .fill(Color32::from_rgba_premultiplied(
                            type_color.r(),
                            type_color.g(),
                            type_color.b(),
                            35,
                        ))
                        .stroke(Stroke::new(
                            1.0_f32,
                            Color32::from_rgba_premultiplied(
                                type_color.r(),
                                type_color.g(),
                                type_color.b(),
                                90,
                            ),
                        ))
                        .rounding(Rounding::same(5.0))
                        .inner_margin(Margin::symmetric(6.0, 2.5));

                    type_badge_frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(icon);
                            ui.label(
                                egui::RichText::new(type_label)
                                    .color(type_color)
                                    .size(12.0)
                                    .strong(),
                            );
                        });
                    });

                    ui.add_space(6.0);

                    // Détails techniques
                    ui.label(
                        egui::RichText::new(detail_str)
                            .monospace()
                            .color(colors::TEXT_PRIMARY)
                            .size(12.5),
                    );

                    // Badge délai (+X ms) aligné à droite
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let delay_badge_frame = Frame::none()
                            .fill(Color32::from_rgba_premultiplied(15, 23, 42, 160))
                            .stroke(Stroke::new(1.0_f32, colors::BORDER_SUBTLE))
                            .rounding(Rounding::same(4.0))
                            .inner_margin(Margin::symmetric(6.0, 2.0));

                        delay_badge_frame.show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(format!("+{} ms", self.action.delay_ms))
                                    .monospace()
                                    .color(colors::TEXT_SECONDARY)
                                    .size(11.5),
                            );
                        });
                    });
                });
            })
            .response
    }
}
