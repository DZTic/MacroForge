use crate::ui::i18n::Language;
use crate::ui::theme::colors;
use crate::ui::widgets::{ButtonVariant, CustomToggleSwitch, GlassButton};
use eframe::egui::{
    self, Color32, DragValue, Frame, Margin, Response, Rounding, Stroke, Ui, Widget,
};

pub struct FilterBar<'a> {
    hide_mouse_moves: &'a mut bool,
    search_query: &'a mut String,
    jump_index: &'a mut usize,
    total_count: usize,
    visible_count: usize,
    lang: Language,
    on_jump: &'a mut bool,
}

impl<'a> FilterBar<'a> {
    pub fn new(
        hide_mouse_moves: &'a mut bool,
        search_query: &'a mut String,
        jump_index: &'a mut usize,
        total_count: usize,
        visible_count: usize,
        lang: Language,
        on_jump: &'a mut bool,
    ) -> Self {
        Self {
            hide_mouse_moves,
            search_query,
            jump_index,
            total_count,
            visible_count,
            lang,
            on_jump,
        }
    }
}

impl<'a> Widget for FilterBar<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let avail_w = ui.available_width();
        let is_compact = avail_w < 720.0;

        let frame = Frame::none()
            .fill(colors::BG_PANEL)
            .stroke(Stroke::new(1.0_f32, colors::BORDER_SUBTLE))
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::symmetric(8.0, 5.0));

        frame
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // 1. Switch Masquer les déplacements souris
                    let toggle_label = if is_compact {
                        "Masquer souris"
                    } else {
                        self.lang.filter_hide_mouse_moves()
                    };
                    let toggle = CustomToggleSwitch::new(self.hide_mouse_moves).label(toggle_label);
                    ui.add(toggle).on_hover_text(
                        "Masquer les événements de déplacement continu de la souris pour simplifier la vue.",
                    );

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);

                    // 2. Recherche textuelle
                    let search_w = if is_compact { 110.0 } else { 160.0 };
                    ui.add(
                        egui::TextEdit::singleline(self.search_query)
                            .hint_text(self.lang.filter_search_placeholder())
                            .desired_width(search_w),
                    );

                    if !self.search_query.is_empty() && ui.small_button("✕").clicked() {
                        self.search_query.clear();
                    }

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);

                    // 3. Saut direct "Aller à n°"
                    ui.label(self.lang.jump_to_action_label());
                    let mut jump_val = *self.jump_index;
                    let drag_resp = ui.add(
                        DragValue::new(&mut jump_val)
                            .range(1..=self.total_count.max(1))
                            .speed(1.0),
                    );
                    if drag_resp.changed() {
                        *self.jump_index = jump_val;
                    }

                    let jump_btn = GlassButton::new(self.lang.jump_btn())
                        .compact(true)
                        .variant(ButtonVariant::Secondary);
                    if ui.add(jump_btn).clicked() {
                        *self.on_jump = true;
                    }

                    // 4. Badge Compteur d'actions aligné à droite
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let count_badge = Frame::none()
                            .fill(Color32::from_rgba_premultiplied(15, 23, 42, 200))
                            .stroke(Stroke::new(1.0_f32, colors::BORDER_SUBTLE))
                            .rounding(Rounding::same(10.0))
                            .inner_margin(Margin::symmetric(8.0, 2.5));

                        count_badge.show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(
                                    self.lang
                                        .action_count_badge(self.visible_count, self.total_count),
                                )
                                .monospace()
                                .size(11.0)
                                .color(if self.visible_count < self.total_count {
                                    colors::ACCENT_WARNING_HOVER
                                } else {
                                    colors::TEXT_SECONDARY
                                }),
                            );
                        });
                    });
                });
            })
            .response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_bar_construction() {
        let mut hide_mouse = false;
        let mut query = String::new();
        let mut jump_idx = 1;
        let mut jump_triggered = false;

        let _filter_bar = FilterBar::new(
            &mut hide_mouse,
            &mut query,
            &mut jump_idx,
            100,
            25,
            Language::Fr,
            &mut jump_triggered,
        );
    }
}

