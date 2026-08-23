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
                "⌨️",
                self.lang.action_key_press(),
                colors::ACCENT_PRIMARY,
                format!("{} (VK: {:#04X})", name, vk),
            ),
            ActionType::KeyRelease(name, vk, _) => (
                "⌨️",
                self.lang.action_key_release(),
                colors::TEXT_SECONDARY,
                format!("{} (VK: {:#04X})", name, vk),
            ),
            ActionType::MouseMove(x, y) => (
                "🖱️",
                self.lang.action_mouse_pos(),
                colors::ACCENT_CYAN,
                format!("X: {:.0}, Y: {:.0}", x, y),
            ),
            ActionType::MouseMoveRelative(dx, dy) => (
                "🖱️",
                self.lang.action_mouse_relative(),
                colors::ACCENT_WARNING,
                format!("ΔX: {}, ΔY: {}", dx, dy),
            ),
            ActionType::MousePress(btn, x, y) => (
                "🖱️",
                self.lang.action_mouse_press(),
                colors::ACCENT_SUCCESS,
                format!("Btn {} à ({:.0}, {:.0})", btn, x, y),
            ),
            ActionType::MouseRelease(btn, x, y) => (
                "🖱️",
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
                "⏱️",
                self.lang.action_wait(),
                colors::ACCENT_WARNING,
                format!("Attente de {} ms", ms),
            ),
            ActionType::WaitImage(path, timeout) => (
                "🖼️",
                self.lang.action_wait_image(),
                colors::ACCENT_PURPLE,
                format!("Fichier: {} ({}ms)", path, timeout),
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
            .inner_margin(Margin::symmetric(8.0, 6.0));

        let card_response = card_frame
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Poignée de Drag & Drop (⠿)
                    let handle_resp = ui.add(
                        egui::Label::new(
                            egui::RichText::new("⠿")
                                .color(colors::TEXT_MUTED)
                                .size(15.0),
                        )
                        .sense(egui::Sense::drag()),
                    );

                    let dnd_payload_id = egui::Id::new("timeline_dnd_dragged_idx");
                    if handle_resp.drag_started() {
                        ui.data_mut(|d| d.insert_temp(dnd_payload_id, self.index));
                    }

                    if handle_resp.hovered() || handle_resp.dragged() {
                        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grab);
                    }

                    ui.add_space(2.0);

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
                        .inner_margin(Margin::symmetric(6.0, 2.0));

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
                            .size(12.0),
                    );

                    // Boutons d'actions et délai alignés à droite
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // 1. Bouton Supprimer
                        if ui
                            .small_button("🗑️")
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
                            .small_button("✏️")
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

        // Gestion du Drag & Drop : Ligne d'insertion visuelle et détection du drop
        let dnd_payload_id = egui::Id::new("timeline_dnd_dragged_idx");
        if let Some(dragged_idx) = ui.data(|d| d.get_temp::<usize>(dnd_payload_id)) {
            if card_response.hovered() {
                let is_top = ui
                    .input(|i| i.pointer.hover_pos())
                    .is_none_or(|pos| pos.y < card_response.rect.center().y);
                let line_y = if is_top {
                    card_response.rect.top() - 1.5
                } else {
                    card_response.rect.bottom() + 1.5
                };

                // Ligne visuelle lumineuse
                ui.painter().hline(
                    card_response.rect.x_range(),
                    line_y,
                    Stroke::new(2.5_f32, colors::ACCENT_PRIMARY_HOVER),
                );

                // Pastille d'accroche visuelle
                ui.painter().circle_filled(
                    egui::pos2(card_response.rect.min.x + 4.0, line_y),
                    3.5,
                    colors::ACCENT_PRIMARY,
                );

                if ui.input(|i| i.pointer.any_released()) {
                    let target_idx = if is_top { self.index } else { self.index + 1 };
                    if dragged_idx != self.index && dragged_idx != target_idx {
                        event = Some(ActionCardEvent::Reorder {
                            from: dragged_idx,
                            to: target_idx,
                        });
                    }
                    ui.data_mut(|d| d.remove_temp::<usize>(dnd_payload_id));
                }
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
