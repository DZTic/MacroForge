use crate::macro_core::{ActionType, MacroAction};
use crate::ui::i18n::Language;
use crate::ui::theme::colors;
use eframe::egui::{
    self, Color32, DragValue, Frame, Margin, Response, Rounding, Stroke, Ui, Widget,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionCardEvent {
    Edit(usize),
    Duplicate(usize),
    Delete(usize),
    MoveUp(usize),
    MoveDown(usize),
    Reorder { from: usize, to: usize },
    DelayChanged(usize, u64),
}

/// Carte d'action moderne dans la timeline de la macro (Glassmorphism Action Card)
pub struct ActionCard<'a> {
    index: usize,
    action: &'a MacroAction,
    selected: bool,
    lang: Language,
    is_first: bool,
    is_last: bool,
}

impl<'a> ActionCard<'a> {
    pub fn new(index: usize, action: &'a MacroAction) -> Self {
        Self {
            index,
            action,
            selected: false,
            lang: Language::Fr,
            is_first: false,
            is_last: false,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn lang(mut self, lang: Language) -> Self {
        self.lang = lang;
        self
    }

    pub fn bounds(mut self, is_first: bool, is_last: bool) -> Self {
        self.is_first = is_first;
        self.is_last = is_last;
        self
    }

    pub fn show(self, ui: &mut Ui) -> (Response, Option<ActionCardEvent>) {
        let mut event = None;

        let (icon, type_label, type_color, detail_str) = match &self.action.action_type {
            ActionType::KeyPress(name, vk, _) => (
                "⌨",
                self.lang.action_key_press(),
                colors::ACCENT_PRIMARY,
                format!("{} (VK: {:#04X})", name, vk),
            ),
            ActionType::KeyRelease(name, vk, _) => (
                "⌨",
                self.lang.action_key_release(),
                colors::TEXT_SECONDARY,
                format!("{} (VK: {:#04X})", name, vk),
            ),
            ActionType::MouseMove(x, y) => (
                "🖱",
                self.lang.action_mouse_pos(),
                colors::ACCENT_CYAN,
                format!("X: {:.0}, Y: {:.0}", x, y),
            ),
            ActionType::MouseMoveRelative(dx, dy) => (
                "🖱",
                self.lang.action_mouse_relative(),
                colors::ACCENT_WARNING,
                format!("ΔX: {}, ΔY: {}", dx, dy),
            ),
            ActionType::MousePress(btn, x, y) => (
                "🖱",
                self.lang.action_mouse_press(),
                colors::ACCENT_SUCCESS,
                format!("Btn {} à ({:.0}, {:.0})", btn, x, y),
            ),
            ActionType::MouseRelease(btn, x, y) => (
                "🖱",
                self.lang.action_mouse_release(),
                colors::TEXT_SECONDARY,
                format!("Btn {} à ({:.0}, {:.0})", btn, x, y),
            ),
            ActionType::Scroll(dx, dy) => (
                "📜",
                self.lang.action_scroll(),
                colors::ACCENT_WARNING_HOVER,
                format!("ΔX: {:.1}, ΔY: {:.1}", dx, dy),
            ),
            ActionType::Wait(ms) => (
                "⏱",
                self.lang.action_wait(),
                colors::ACCENT_WARNING,
                format!("Attente de {} ms", ms),
            ),
            ActionType::WaitImage(path, timeout) => (
                "🖼",
                self.lang.action_wait_image(),
                colors::ACCENT_PURPLE,
                format!("{} ({}ms)", path, timeout),
            ),
        };

        let dnd_payload_id = egui::Id::new("timeline_dnd_dragged_idx");
        let is_being_dragged = ui.data(|d| d.get_temp::<usize>(dnd_payload_id)) == Some(self.index);
        let maybe_dragged_idx = ui.data(|d| d.get_temp::<usize>(dnd_payload_id));

        let card_frame = Frame::none()
            .fill(if is_being_dragged {
                Color32::from_rgba_premultiplied(40, 60, 95, 160)
            } else if self.selected {
                colors::BG_CARD_SELECTED
            } else {
                colors::BG_CARD
            })
            .stroke(if is_being_dragged {
                Stroke::new(1.5_f32, colors::ACCENT_PRIMARY_HOVER)
            } else if self.selected {
                Stroke::new(1.5_f32, colors::ACCENT_PRIMARY)
            } else {
                Stroke::new(1.0_f32, colors::BORDER_CARD)
            })
            .rounding(Rounding::same(7.0))
            .inner_margin(Margin::symmetric(10.0, 6.0));

        let card_response = card_frame
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Poignée de Drag & Drop (⠿) avec curseur interactif
                    let handle_resp = ui.add(
                        egui::Label::new(
                            egui::RichText::new("⠿")
                                .color(if is_being_dragged {
                                    colors::ACCENT_PRIMARY_HOVER
                                } else {
                                    colors::TEXT_MUTED
                                })
                                .size(16.0),
                        )
                        .sense(egui::Sense::drag()),
                    );

                    if handle_resp.drag_started() {
                        ui.data_mut(|d| d.insert_temp(dnd_payload_id, self.index));
                    }

                    if handle_resp.hovered() {
                        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grab);
                    } else if handle_resp.dragged() || is_being_dragged {
                        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grabbing);
                    }

                    ui.add_space(2.0);

                    // Badge numéro d'index (#001) avec fond sombre
                    let index_text = format!("#{:03}", self.index + 1);
                    let index_badge = Frame::none()
                        .fill(colors::BG_INPUT)
                        .stroke(Stroke::new(1.0_f32, colors::BORDER_SUBTLE))
                        .rounding(Rounding::same(4.0))
                        .inner_margin(Margin::symmetric(5.0, 2.0));
                    index_badge.show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(index_text)
                                .monospace()
                                .color(colors::TEXT_MUTED)
                                .size(11.0),
                        );
                    });

                    ui.add_space(4.0);

                    // Badge de catégorie d'action avec pastille / fond teinté haute visibilité
                    let type_badge_frame = Frame::none()
                        .fill(Color32::from_rgba_premultiplied(
                            type_color.r(),
                            type_color.g(),
                            type_color.b(),
                            45,
                        ))
                        .stroke(Stroke::new(
                            1.0_f32,
                            Color32::from_rgba_premultiplied(
                                type_color.r(),
                                type_color.g(),
                                type_color.b(),
                                120,
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
                                    .size(11.5)
                                    .strong(),
                            );
                        });
                    });

                    ui.add_space(6.0);

                    // Détails techniques clairs et lisibles
                    ui.label(
                        egui::RichText::new(detail_str)
                            .monospace()
                            .color(colors::TEXT_PRIMARY)
                            .size(12.0),
                    );

                    // Boutons d'actions et délai alignés à droite
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // 1. Bouton Supprimer
                        if ui
                            .small_button("🗑")
                            .on_hover_text(self.lang.delete_tooltip())
                            .clicked()
                        {
                            event = Some(ActionCardEvent::Delete(self.index));
                        }

                        // 2. Bouton Dupliquer
                        if ui
                            .small_button("📋")
                            .on_hover_text(self.lang.duplicate_tooltip())
                            .clicked()
                        {
                            event = Some(ActionCardEvent::Duplicate(self.index));
                        }

                        // 3. Bouton Éditer
                        if ui
                            .small_button("✏")
                            .on_hover_text(self.lang.edit_tooltip())
                            .clicked()
                        {
                            event = Some(ActionCardEvent::Edit(self.index));
                        }

                        // 4. Boutons Déplacement ▲ / ▼
                        if !self.is_last
                            && ui
                                .small_button("▼")
                                .on_hover_text(self.lang.move_down_tooltip())
                                .clicked()
                        {
                            event = Some(ActionCardEvent::MoveDown(self.index));
                        }
                        if !self.is_first
                            && ui
                                .small_button("▲")
                                .on_hover_text(self.lang.move_up_tooltip())
                                .clicked()
                        {
                            event = Some(ActionCardEvent::MoveUp(self.index));
                        }

                        ui.add_space(6.0);

                        // 5. Ajustement direct du délai
                        let mut cur_delay = self.action.delay_ms;
                        let delay_resp = ui.add(
                            DragValue::new(&mut cur_delay)
                                .range(0..=60000)
                                .speed(5.0)
                                .prefix("+")
                                .suffix("ms"),
                        );
                        if delay_resp.changed() {
                            event = Some(ActionCardEvent::DelayChanged(self.index, cur_delay));
                        }
                    });
                });
            })
            .response;

        // Gestion du Drag & Drop : Détection géométrique précise et retour visuel
        if let Some(dragged_idx) = maybe_dragged_idx {
            ui.ctx().request_repaint();

            let pointer_pos = ui.input(|i| i.pointer.hover_pos().or(i.pointer.interact_pos()));
            let is_over = pointer_pos.map_or(false, |pos| card_response.rect.contains(pos));

            if is_over && dragged_idx != self.index {
                let pos = pointer_pos.unwrap();
                let is_top = pos.y < card_response.rect.center().y;
                let line_y = if is_top {
                    card_response.rect.top() - 2.0
                } else {
                    card_response.rect.bottom() + 2.0
                };

                // Ligne visuelle lumineuse d'insertion
                ui.painter().hline(
                    (card_response.rect.min.x - 2.0)..=(card_response.rect.max.x + 2.0),
                    line_y,
                    Stroke::new(3.0_f32, colors::ACCENT_PRIMARY_HOVER),
                );

                // Pastille d'accroche visuelle gauche
                ui.painter().circle_filled(
                    egui::pos2(card_response.rect.min.x, line_y),
                    4.0,
                    colors::ACCENT_PRIMARY,
                );

                // Pastille d'accroche visuelle droite
                ui.painter().circle_filled(
                    egui::pos2(card_response.rect.max.x, line_y),
                    4.0,
                    colors::ACCENT_PRIMARY,
                );

                if ui.input(|i| i.pointer.any_released()) {
                    let target_idx = if is_top { self.index } else { self.index + 1 };
                    event = Some(ActionCardEvent::Reorder {
                        from: dragged_idx,
                        to: target_idx,
                    });
                    ui.data_mut(|d| d.remove_temp::<usize>(dnd_payload_id));
                }
            }

            // Info-bulle flottante guidant le déplacement
            if is_being_dragged && pointer_pos.is_some() {
                egui::show_tooltip_at_pointer(
                    ui.ctx(),
                    ui.layer_id(),
                    egui::Id::new("dnd_tooltip"),
                    |ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "🔀 Déplacement Action #{:03} ({})",
                                self.index + 1,
                                type_label
                            ))
                            .color(colors::TEXT_PRIMARY)
                            .size(12.0),
                        );
                    },
                );
            }
        }

        (card_response, event)
    }
}

impl<'a> Widget for ActionCard<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (response, _) = self.show(ui);
        response
    }
}
