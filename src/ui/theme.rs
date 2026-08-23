use eframe::egui::{
    self, epaint::Shadow, Color32, Context, FontFamily, FontId, Frame, Margin, Rounding, Stroke,
    TextStyle, Vec2, Visuals,
};

/// Palette de couleurs du Design System MacroForge (Dark UI / Glassmorphism)
pub mod colors {
    use eframe::egui::Color32;

    // Arrière-plans
    pub const BG_APP: Color32 = Color32::from_rgb(13, 17, 23); // #0d1117 (Dark Slate)
    pub const BG_PANEL: Color32 = Color32::from_rgb(22, 27, 34); // #161b22
    pub const BG_CARD: Color32 = Color32::from_rgba_premultiplied(30, 41, 59, 180); // rgba(30, 41, 59, 0.7)
    pub const BG_CARD_HOVER: Color32 = Color32::from_rgba_premultiplied(40, 53, 75, 210);
    pub const BG_CARD_ACTIVE: Color32 = Color32::from_rgba_premultiplied(50, 65, 90, 230);
    pub const BG_INPUT: Color32 = Color32::from_rgba_premultiplied(15, 23, 42, 200);

    // Bordures
    pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(48, 54, 61); // #30363d
    pub const BORDER_HOVER: Color32 = Color32::from_rgb(88, 166, 255); // #58a6ff
    pub const BORDER_ACTIVE: Color32 = Color32::from_rgb(59, 130, 246); // #3b82f6

    // Accents
    pub const ACCENT_PRIMARY: Color32 = Color32::from_rgb(59, 130, 246); // #3b82f6 (Bleu lumineux)
    pub const ACCENT_PRIMARY_HOVER: Color32 = Color32::from_rgb(96, 165, 250); // #60a5fa
    pub const ACCENT_SUCCESS: Color32 = Color32::from_rgb(16, 185, 129); // #10b981 (Vert émeraude)
    pub const ACCENT_SUCCESS_HOVER: Color32 = Color32::from_rgb(52, 211, 153); // #34d399
    pub const ACCENT_DANGER: Color32 = Color32::from_rgb(239, 68, 68); // #ef4444 (Rouge vibrant)
    pub const ACCENT_DANGER_HOVER: Color32 = Color32::from_rgb(248, 113, 113); // #f87171
    pub const ACCENT_WARNING: Color32 = Color32::from_rgb(245, 158, 11); // #f59e0b (Ambre)
    pub const ACCENT_WARNING_HOVER: Color32 = Color32::from_rgb(251, 191, 36); // #fbbf24
    pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(217, 70, 239); // #d946ef (Fuchsia/Violet)
    pub const ACCENT_PURPLE_HOVER: Color32 = Color32::from_rgb(232, 121, 249); // #e879f9
    pub const ACCENT_CYAN: Color32 = Color32::from_rgb(6, 182, 212); // #06b6d4 (Cyan)

    // Textes
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(240, 246, 252); // #f0f6fc
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(139, 148, 158); // #8b949e
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(110, 118, 129); // #6e7681
    pub const TEXT_WHITE: Color32 = Color32::from_rgb(255, 255, 255);
}

/// Applique le thème sombre personnalisé et la typographie au contexte egui
pub fn apply_theme(ctx: &Context) {
    let mut visuals = Visuals::dark();

    // Couleurs de base de l'application
    visuals.panel_fill = colors::BG_PANEL;
    visuals.window_fill = colors::BG_APP;
    visuals.faint_bg_color = colors::BG_CARD;
    visuals.extreme_bg_color = colors::BG_INPUT;

    // Bordures et séparateurs
    visuals.window_stroke = Stroke::new(1.0_f32, colors::BORDER_SUBTLE);
    visuals.widgets.noninteractive.bg_fill = colors::BG_CARD;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, colors::BORDER_SUBTLE);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, colors::TEXT_SECONDARY);
    visuals.widgets.noninteractive.rounding = Rounding::same(8.0);

    // Widgets inactifs
    visuals.widgets.inactive.bg_fill = Color32::from_rgba_premultiplied(30, 41, 59, 140);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, colors::BORDER_SUBTLE);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, colors::TEXT_PRIMARY);
    visuals.widgets.inactive.rounding = Rounding::same(8.0);

    // Widgets survolés (hovered)
    visuals.widgets.hovered.bg_fill = colors::BG_CARD_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, colors::BORDER_HOVER);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, colors::TEXT_WHITE);
    visuals.widgets.hovered.rounding = Rounding::same(8.0);

    // Widgets actifs/pressés
    visuals.widgets.active.bg_fill = colors::BG_CARD_ACTIVE;
    visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, colors::ACCENT_PRIMARY);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, colors::TEXT_WHITE);
    visuals.widgets.active.rounding = Rounding::same(8.0);

    // Widgets ouverts / déroulants
    visuals.widgets.open.bg_fill = colors::BG_CARD_HOVER;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0_f32, colors::ACCENT_PRIMARY);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0_f32, colors::TEXT_WHITE);
    visuals.widgets.open.rounding = Rounding::same(8.0);

    // Sélection
    visuals.selection.bg_fill = Color32::from_rgba_premultiplied(59, 130, 246, 120);
    visuals.selection.stroke = Stroke::new(1.0_f32, colors::ACCENT_PRIMARY);

    // Ombres fenêtres et modales
    visuals.window_shadow = Shadow {
        offset: Vec2::new(0.0, 4.0),
        blur: 12.0,
        spread: 0.0,
        color: Color32::from_black_alpha(100),
    };
    visuals.popup_shadow = Shadow {
        offset: Vec2::new(0.0, 2.0),
        blur: 8.0,
        spread: 0.0,
        color: Color32::from_black_alpha(80),
    };

    // Configuration des styles de texte
    let mut style = (*ctx.style()).clone();
    style.visuals = visuals;

    style.text_styles = [
        (
            TextStyle::Heading,
            FontId::new(18.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Name("Subheading".into()),
            FontId::new(15.0, FontFamily::Proportional),
        ),
        (TextStyle::Body, FontId::new(13.5, FontFamily::Proportional)),
        (
            TextStyle::Button,
            FontId::new(13.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Monospace,
            FontId::new(12.5, FontFamily::Monospace),
        ),
        (
            TextStyle::Small,
            FontId::new(11.0, FontFamily::Proportional),
        ),
    ]
    .into();

    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    style.spacing.window_margin = Margin::same(12.0);

    ctx.set_style(style);
}

/// Frame stylisée pour les cartes d'action et widgets (effet Glassmorphism)
pub fn glass_card_frame() -> Frame {
    Frame::none()
        .fill(colors::BG_CARD)
        .stroke(Stroke::new(1.0_f32, colors::BORDER_SUBTLE))
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin::symmetric(12.0, 8.0))
        .shadow(Shadow {
            offset: Vec2::new(0.0, 2.0),
            blur: 6.0,
            spread: 0.0,
            color: Color32::from_black_alpha(40),
        })
}

/// Frame pour les conteneurs ou panneaux intérieurs
pub fn glass_panel_frame() -> Frame {
    Frame::none()
        .fill(colors::BG_PANEL)
        .stroke(Stroke::new(1.0_f32, colors::BORDER_SUBTLE))
        .rounding(Rounding::same(10.0))
        .inner_margin(Margin::same(10.0))
}

/// Frame pour le header supérieur
pub fn header_frame() -> Frame {
    Frame::none()
        .fill(colors::BG_PANEL)
        .stroke(Stroke::new(1.0_f32, colors::BORDER_SUBTLE))
        .inner_margin(Margin::symmetric(14.0, 10.0))
}

/// Frame pour le footer d'actions inférieur
pub fn footer_frame() -> Frame {
    Frame::none()
        .fill(colors::BG_PANEL)
        .stroke(Stroke::new(1.0_f32, colors::BORDER_SUBTLE))
        .inner_margin(Margin::symmetric(14.0, 10.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_colors_integrity() {
        assert_eq!(colors::BG_APP, Color32::from_rgb(13, 17, 23));
        assert_eq!(colors::ACCENT_PRIMARY, Color32::from_rgb(59, 130, 246));
        assert_eq!(colors::ACCENT_SUCCESS, Color32::from_rgb(16, 185, 129));
        assert_eq!(colors::ACCENT_DANGER, Color32::from_rgb(239, 68, 68));
    }

    #[test]
    fn test_apply_theme() {
        let ctx = Context::default();
        apply_theme(&ctx);
        assert_eq!(ctx.style().visuals.panel_fill, colors::BG_PANEL);
    }
}
