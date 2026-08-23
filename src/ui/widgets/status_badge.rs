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
}

impl<'a> StatusBadge<'a> {
    pub fn new(kind: StatusKind) -> Self {
        Self { kind, label: None }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }
}

impl<'a> Widget for StatusBadge<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (dot_color, glow_color, bg_fill, border_stroke, default_text) = match self.kind {
            StatusKind::Recording => (
                colors::ACCENT_DANGER,
                Color32::from_rgba_premultiplied(239, 68, 68, 80),
                Color32::from_rgba_premultiplied(239, 68, 68, 30),
                Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(239, 68, 68, 120)),
                "ENREGISTREMENT EN COURS",
            ),
            StatusKind::Playing => (
                colors::ACCENT_SUCCESS,
                Color32::from_rgba_premultiplied(16, 185, 129, 80),
                Color32::from_rgba_premultiplied(16, 185, 129, 30),
                Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(16, 185, 129, 120)),
                "LECTURE ACTIVE",
            ),
            StatusKind::Paused => (
                colors::ACCENT_WARNING,
                Color32::from_rgba_premultiplied(245, 158, 11, 80),
                Color32::from_rgba_premultiplied(245, 158, 11, 30),
                Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(245, 158, 11, 120)),
                "EN PAUSE",
            ),
            StatusKind::Idle => (
                colors::TEXT_MUTED,
                Color32::from_rgba_premultiplied(110, 118, 129, 40),
                Color32::from_rgba_premultiplied(30, 41, 59, 120),
                Stroke::new(1.0_f32, colors::BORDER_SUBTLE),
                "PRÊT / INACTIF",
            ),
        };

        let display_text = self.label.unwrap_or(default_text);
        let font_id = TextStyle::Small.resolve(ui.style());
        let text_galley = ui.painter().layout_no_wrap(
            display_text.to_string(),
            font_id,
            match self.kind {
                StatusKind::Recording => colors::ACCENT_DANGER_HOVER,
                StatusKind::Playing => colors::ACCENT_SUCCESS_HOVER,
                StatusKind::Paused => colors::ACCENT_WARNING_HOVER,
                StatusKind::Idle => colors::TEXT_SECONDARY,
            },
        );

        let dot_radius = 4.0;
        let glow_radius = 7.0;
        let padding = Vec2::new(10.0, 4.0);
        let spacing_dot_text = 8.0;

        let desired_size = Vec2::new(
            padding.x * 2.0 + glow_radius * 2.0 + spacing_dot_text + text_galley.size().x,
            (text_galley.size().y + padding.y * 2.0).max(24.0),
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
            ui.painter().galley(text_pos, text_galley, Color32::WHITE);
        }

        response
    }
}
