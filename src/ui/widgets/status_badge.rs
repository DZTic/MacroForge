use crate::ui::theme::colors;
use eframe::egui::{Color32, Pos2, Response, Rounding, Sense, Stroke, TextStyle, Ui, Vec2, Widget};

/// États de fonctionnement représentés par le StatusBadge
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    Recording, // Rouge vibrant
    Playing,   // Vert émeraude
    Paused,    // Ambre
    Idle,      // Gris / Slate neutre
}

/// Badge d'état avec puce lumineuse et capsule translucide (Glassmorphism)
pub struct StatusBadge<'a> {
    kind: StatusKind,
    label: Option<&'a str>,
    compact: bool,
}

impl<'a> StatusBadge<'a> {
    pub fn new(kind: StatusKind) -> Self {
        Self {
            kind,
            label: None,
            compact: false,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }
}

impl<'a> Widget for StatusBadge<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (dot_color, glow_color, bg_fill, border_stroke, default_text) = match self.kind {
            StatusKind::Recording => (
                colors::ACCENT_DANGER,
                Color32::from_rgba_premultiplied(239, 68, 68, 90),
                Color32::from_rgba_premultiplied(239, 68, 68, 35),
                Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(239, 68, 68, 140)),
                if self.compact {
                    "REC"
                } else {
                    "ENREGISTREMENT"
                },
            ),
            StatusKind::Playing => (
                colors::ACCENT_SUCCESS,
                Color32::from_rgba_premultiplied(16, 185, 129, 90),
                Color32::from_rgba_premultiplied(16, 185, 129, 35),
                Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(16, 185, 129, 140)),
                if self.compact {
                    "PLAY"
                } else {
                    "LECTURE ACTIVE"
                },
            ),
            StatusKind::Paused => (
                colors::ACCENT_WARNING,
                Color32::from_rgba_premultiplied(245, 158, 11, 90),
                Color32::from_rgba_premultiplied(245, 158, 11, 35),
                Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(245, 158, 11, 140)),
                if self.compact { "PAUSE" } else { "EN PAUSE" },
            ),
            StatusKind::Idle => (
                colors::ACCENT_PRIMARY_HOVER,
                Color32::from_rgba_premultiplied(59, 130, 246, 60),
                colors::BG_CARD,
                Stroke::new(1.0_f32, colors::BORDER_CARD),
                if self.compact {
                    "PRÊT"
                } else {
                    "PRÊT / INACTIF"
                },
            ),
        };

        let display_text = self.label.unwrap_or(default_text);
        let font_id = TextStyle::Small.resolve(ui.style());
        let text_color = match self.kind {
            StatusKind::Recording => colors::ACCENT_DANGER_HOVER,
            StatusKind::Playing => colors::ACCENT_SUCCESS_HOVER,
            StatusKind::Paused => colors::ACCENT_WARNING_HOVER,
            StatusKind::Idle => colors::TEXT_PRIMARY,
        };

        let text_galley =
            ui.painter()
                .layout_no_wrap(display_text.to_string(), font_id, text_color);

        let dot_radius = 3.5;
        let glow_radius = 6.5;
        let padding = if self.compact {
            Vec2::new(7.0, 3.0)
        } else {
            Vec2::new(9.0, 3.5)
        };
        let spacing_dot_text = 6.0;

        let desired_size = Vec2::new(
            padding.x * 2.0 + glow_radius * 2.0 + spacing_dot_text + text_galley.size().x,
            (text_galley.size().y + padding.y * 2.0).max(22.0),
        );

        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());

        if ui.is_rect_visible(rect) {
            let rounding = Rounding::same(rect.height() * 0.5);

            // Fond capsule
            ui.painter().rect(rect, rounding, bg_fill, border_stroke);

            // Halo lumineux (glow)
            let dot_center = Pos2::new(rect.min.x + padding.x + glow_radius, rect.center().y);
            ui.painter()
                .circle_filled(dot_center, glow_radius, glow_color);

            // Pastille centrale solide
            ui.painter()
                .circle_filled(dot_center, dot_radius, dot_color);

            // Texte d'état
            let text_pos = Pos2::new(
                rect.min.x + padding.x + glow_radius * 2.0 + spacing_dot_text,
                rect.center().y - text_galley.size().y * 0.5,
            );
            ui.painter().galley(text_pos, text_galley, text_color);
        }

        response
    }
}
