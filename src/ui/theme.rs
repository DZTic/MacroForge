use eframe::egui::{
    self, epaint::Shadow, Color32, Context, FontFamily, FontId, Frame, Margin, Rounding, Stroke,
    TextStyle, Vec2, Visuals,
};

/// Palette de couleurs du Design System MacroForge (Dark Glassmorphism / Modern Tech)
pub mod colors {
    use eframe::egui::Color32;

    // Arrière-plans sombres profonds & surfaces
    pub const BG_APP: Color32 = Color32::from_rgb(10, 14, 23); // #0a0e17
    pub const BG_PANEL: Color32 = Color32::from_rgb(15, 23, 42); // #0f172a (Slate 900)
    pub const BG_CARD: Color32 = Color32::from_rgba_premultiplied(30, 41, 59, 180); // #1e293b
    pub const BG_CARD_HOVER: Color32 = Color32::from_rgba_premultiplied(51, 65, 85, 210);
    pub const BG_CARD_ACTIVE: Color32 = Color32::from_rgba_premultiplied(71, 85, 105, 230);
    pub const BG_INPUT: Color32 = Color32::from_rgba_premultiplied(15, 23, 42, 220);

    // Bordures lumineuses et subtiles
    pub const BORDER_SUBTLE: Color32 = Color32::from_rgba_premultiplied(148, 163, 184, 45);
    pub const BORDER_HOVER: Color32 = Color32::from_rgb(96, 165, 250); // #60a5fa
    pub const BORDER_ACTIVE: Color32 = Color32::from_rgb(59, 130, 246); // #3b82f6

    // Accents vibrants
    pub const ACCENT_PRIMARY: Color32 = Color32::from_rgb(59, 130, 246); // #3b82f6 (Bleu électrique)
    pub const ACCENT_PRIMARY_HOVER: Color32 = Color32::from_rgb(96, 165, 250); // #60a5fa
    pub const ACCENT_SUCCESS: Color32 = Color32::from_rgb(16, 185, 129); // #10b981 (Vert émeraude)
    pub const ACCENT_SUCCESS_HOVER: Color32 = Color32::from_rgb(52, 211, 153); // #34d399
    pub const ACCENT_DANGER: Color32 = Color32::from_rgb(239, 68, 68); // #ef4444 (Rouge vif)
    pub const ACCENT_DANGER_HOVER: Color32 = Color32::from_rgb(248, 113, 113); // #f87171
    pub const ACCENT_WARNING: Color32 = Color32::from_rgb(245, 158, 11); // #f59e0b (Ambre)
    pub const ACCENT_WARNING_HOVER: Color32 = Color32::from_rgb(251, 191, 36); // #fbbf24
    pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(168, 85, 247); // #a855f7 (Violet moderne)
    pub const ACCENT_PURPLE_HOVER: Color32 = Color32::from_rgb(192, 132, 252); // #c084fc
    pub const ACCENT_CYAN: Color32 = Color32::from_rgb(6, 182, 212); // #06b6d4 (Cyan)

    // Typographie & Hiérarchie de contraste
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(241, 245, 249); // #f1f5f9
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(148, 163, 184); // #94a3b8
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(100, 116, 139); // #64748b
    pub const TEXT_WHITE: Color32 = Color32::from_rgb(255, 255, 255);
}

/// Configure les polices système Windows (Segoe UI, Segoe UI Emoji, Consolas) pour un rendu net
pub fn configure_fonts(ctx: &Context) {
    let mut fonts = egui::FontDefinitions::default();

    #[cfg(windows)]
    {
        // 1. Segoe UI (Police système principale pour interface propre)
        if let Ok(font_data) = std::fs::read("C:\\Windows\\Fonts\\segoeui.ttf") {
            fonts.font_data.insert(
                "segoe_ui".to_owned(),
                egui::FontData::from_owned(font_data),
            );
            if let Some(family) = fonts.families.get_mut(&FontFamily::Proportional) {
                family.insert(0, "segoe_ui".to_owned());
            }
        }

        // 2. Segoe UI Emoji (Support complet des glyphes et symboles modernes)
        if let Ok(emoji_data) = std::fs::read("C:\\Windows\\Fonts\\seguiemj.ttf") {
            fonts.font_data.insert(
                "segoe_emoji".to_owned(),
                egui::FontData::from_owned(emoji_data),
            );
            if let Some(family) = fonts.families.get_mut(&FontFamily::Proportional) {
                family.push("segoe_emoji".to_owned());
            }
            if let Some(family) = fonts.families.get_mut(&FontFamily::Monospace) {
                family.push("segoe_emoji".to_owned());
            }
        }

        // 3. Segoe UI Symbol (Symboles universels)
        if let Ok(symbol_data) = std::fs::read("C:\\Windows\\Fonts\\seguisym.ttf") {
            fonts.font_data.insert(
                "segoe_symbol".to_owned(),
                egui::FontData::from_owned(symbol_data),
            );
            if let Some(family) = fonts.families.get_mut(&FontFamily::Proportional) {
                family.push("segoe_symbol".to_owned());
            }
        }

        // 4. Consolas pour la typographie Monospace (Coordonnées, délais, VK codes)
        if let Ok(mono_data) = std::fs::read("C:\\Windows\\Fonts\\consola.ttf") {
            fonts.font_data.insert(
                "consolas".to_owned(),
                egui::FontData::from_owned(mono_data),
            );
            if let Some(family) = fonts.families.get_mut(&FontFamily::Monospace) {
                family.insert(0, "consolas".to_owned());
            }
        }
    }

    ctx.set_fonts(fonts);
}

/// Applique le thème sombre personnalisé et la typographie au contexte egui
pub fn apply_theme(ctx: &Context) {
    // 1. Initialiser et appliquer les polices nettes
    configure_fonts(ctx);

    // 2. Configurer les styles visuels Dark Glassmorphism
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
    visuals.widgets.inactive.rounding = Rounding::same(6.0);

    // Widgets survolés (hovered)
    visuals.widgets.hovered.bg_fill = colors::BG_CARD_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, colors::BORDER_HOVER);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, colors::TEXT_WHITE);
    visuals.widgets.hovered.rounding = Rounding::same(6.0);

    // Widgets actifs/pressés
    visuals.widgets.active.bg_fill = colors::BG_CARD_ACTIVE;
    visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, colors::ACCENT_PRIMARY);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, colors::TEXT_WHITE);
    visuals.widgets.active.rounding = Rounding::same(6.0);

    // Widgets ouverts / déroulants
    visuals.widgets.open.bg_fill = colors::BG_CARD_HOVER;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0_f32, colors::ACCENT_PRIMARY);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0_f32, colors::TEXT_WHITE);
    visuals.widgets.open.rounding = Rounding::same(6.0);

    // Sélection
    visuals.selection.bg_fill = Color32::from_rgba_premultiplied(59, 130, 246, 120);
    visuals.selection.stroke = Stroke::new(1.0_f32, colors::ACCENT_PRIMARY);

    // Ombres fenêtres et modales
    visuals.window_shadow = Shadow {
        offset: Vec2::new(0.0, 4.0),
        blur: 16.0,
        spread: 0.0,
        color: Color32::from_black_alpha(120),
    };
    visuals.popup_shadow = Shadow {
        offset: Vec2::new(0.0, 2.0),
        blur: 10.0,
        spread: 0.0,
        color: Color32::from_black_alpha(90),
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
        assert_eq!(colors::BG_APP, Color32::from_rgb(10, 14, 23));
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

