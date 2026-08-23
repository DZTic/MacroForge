use crate::ui::theme::colors;
use eframe::egui::{
    Color32, Pos2, Rect, Response, Rounding, Sense, Stroke, TextStyle, Ui, Vec2, Widget,
};

/// Interrupteur à bascule fluide et moderne (Glassmorphism Toggle Switch)
pub struct CustomToggleSwitch<'a> {
    value: &'a mut bool,
    label: Option<&'a str>,
}

impl<'a> CustomToggleSwitch<'a> {
    pub fn new(value: &'a mut bool) -> Self {
        Self { value, label: None }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }
}

impl<'a> Widget for CustomToggleSwitch<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let switch_width: f32 = 38.0;
        let switch_height: f32 = 20.0;
        let thumb_radius: f32 = 7.5;

        let font_id = TextStyle::Body.resolve(ui.style());
        let label_galley = self.label.map(|lbl| {
            ui.painter()
                .layout_no_wrap(lbl.to_string(), font_id, colors::TEXT_PRIMARY)
        });

        let mut desired_width = switch_width;
        if let Some(ref lg) = label_galley {
            desired_width += lg.size().x + 8.0;
        }

        let desired_size = Vec2::new(desired_width, switch_height.max(22.0));
        let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());

        if response.clicked() {
            *self.value = !*self.value;
            response.mark_changed();
        }

        if ui.is_rect_visible(rect) {
            let is_on = *self.value;
            let is_hovered = response.hovered();

            let switch_rect = Rect::from_min_size(
                Pos2::new(rect.min.x, rect.center().y - switch_height * 0.5),
                Vec2::new(switch_width, switch_height),
            );

            // Couleur de fond du commutateur
            let (bg_color, border_stroke) = if is_on {
                if is_hovered {
                    (
                        colors::ACCENT_PRIMARY_HOVER,
                        Stroke::new(1.0, colors::ACCENT_PRIMARY),
                    )
                } else {
                    (
                        colors::ACCENT_PRIMARY,
                        Stroke::new(1.0, Color32::from_rgb(29, 78, 216)),
                    )
                }
            } else if is_hovered {
                (
                    colors::BG_CARD_HOVER,
                    Stroke::new(1.0, colors::BORDER_HOVER),
                )
            } else {
                (colors::BG_INPUT, Stroke::new(1.0, colors::BORDER_SUBTLE))
            };

            // Dessin du corps en pilule
            ui.painter().rect(
                switch_rect,
                Rounding::same(switch_height * 0.5),
                bg_color,
                border_stroke,
            );

            // Position du curseur (thumb)
            let thumb_x = if is_on {
                switch_rect.max.x - thumb_radius - 2.5
            } else {
                switch_rect.min.x + thumb_radius + 2.5
            };
            let thumb_center = Pos2::new(thumb_x, switch_rect.center().y);

            // Ombre portée subtile du thumb
            ui.painter().circle_filled(
                Pos2::new(thumb_center.x, thumb_center.y + 1.0),
                thumb_radius,
                Color32::from_black_alpha(50),
            );

            // Curseur blanc
            ui.painter()
                .circle_filled(thumb_center, thumb_radius, colors::TEXT_WHITE);

            // Libellé optionnel
            if let Some(lg) = label_galley {
                let text_pos =
                    Pos2::new(switch_rect.max.x + 8.0, rect.center().y - lg.size().y * 0.5);
                ui.painter().galley(text_pos, lg, colors::TEXT_PRIMARY);
            }
        }

        response
    }
}
