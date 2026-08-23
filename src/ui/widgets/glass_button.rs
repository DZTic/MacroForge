use crate::ui::theme::colors;
use eframe::egui::{
    Color32, Pos2, Rect, Response, Rounding, Sense, Stroke, TextStyle, Ui, Vec2, Widget,
};

/// Variantes de style pour les boutons Glassmorphism
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,   // Accent bleu électrique
    Success,   // Accent vert émeraude (Play)
    Danger,    // Accent rouge rubis (Record / Arrêt)
    Warning,   // Accent ambre (Pause / Attention)
    Secondary, // Slate neutre (Ouvrir, Sauvegarder, Paramètres)
    Ghost,     // Discret sans fond fixe
}

/// Bouton stylisé avec effet de verre dépoli, dégradé subtil et retour visuel au survol
pub struct GlassButton<'a> {
    text: &'a str,
    icon: Option<&'a str>,
    shortcut: Option<&'a str>,
    variant: ButtonVariant,
    min_size: Vec2,
    selected: bool,
    enabled: bool,
    compact: bool,
}

impl<'a> GlassButton<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            icon: None,
            shortcut: None,
            variant: ButtonVariant::Secondary,
            min_size: Vec2::new(0.0, 32.0),
            selected: false,
            enabled: true,
            compact: false,
        }
    }

    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn shortcut(mut self, shortcut: &'a str) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn min_size(mut self, min_size: Vec2) -> Self {
        self.min_size = min_size;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        if compact {
            self.min_size = Vec2::new(0.0, 26.0);
        }
        self
    }
}

impl<'a> Widget for GlassButton<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let padding = if self.compact {
            Vec2::new(9.0, 4.0)
        } else {
            Vec2::new(13.0, 6.0)
        };

        let font_id = if self.compact {
            TextStyle::Small.resolve(ui.style())
        } else {
            TextStyle::Button.resolve(ui.style())
        };
        let shortcut_font_id = TextStyle::Small.resolve(ui.style());

        // Construction du libellé
        let mut full_text = String::new();
        if let Some(icon) = self.icon {
            full_text.push_str(icon);
            if !self.text.is_empty() {
                full_text.push(' ');
            }
        }
        full_text.push_str(self.text);

        let galley =
            ui.painter()
                .layout_no_wrap(full_text.clone(), font_id.clone(), colors::TEXT_PRIMARY);

        let shortcut_galley = self.shortcut.map(|sc| {
            ui.painter().layout_no_wrap(
                sc.to_string(),
                shortcut_font_id.clone(),
                colors::TEXT_PRIMARY,
            )
        });

        let mut total_content_width = galley.size().x;
        let mut shortcut_box_width = 0.0;

        if let Some(ref sc_g) = shortcut_galley {
            shortcut_box_width = sc_g.size().x + 10.0;
            total_content_width += shortcut_box_width + 10.0; // 10px de séparation nette
        }

        let desired_size = Vec2::new(
            (total_content_width + padding.x * 2.0).max(self.min_size.x),
            (galley.size().y + padding.y * 2.0).max(self.min_size.y),
        );

        let (rect, response) = ui.allocate_exact_size(
            desired_size,
            if self.enabled {
                Sense::click()
            } else {
                Sense::hover()
            },
        );

        if ui.is_rect_visible(rect) {
            let is_hovered = response.hovered() && self.enabled;
            let is_clicked = response.is_pointer_button_down_on() && self.enabled;

            // Couleurs de fond, de bordure et de texte selon la variante avec contraste maximal
            let (bg_fill, border_stroke, text_color) = match self.variant {
                ButtonVariant::Primary => {
                    if is_clicked || self.selected {
                        (
                            Color32::from_rgb(29, 78, 216),
                            Stroke::new(1.5_f32, colors::ACCENT_PRIMARY_HOVER),
                            colors::TEXT_WHITE,
                        )
                    } else if is_hovered {
                        (
                            Color32::from_rgb(37, 99, 235),
                            Stroke::new(1.5_f32, colors::ACCENT_PRIMARY_HOVER),
                            colors::TEXT_WHITE,
                        )
                    } else {
                        (
                            Color32::from_rgb(30, 64, 175),
                            Stroke::new(1.0_f32, colors::ACCENT_PRIMARY),
                            colors::TEXT_WHITE,
                        )
                    }
                }
                ButtonVariant::Success => {
                    if is_clicked || self.selected {
                        (
                            Color32::from_rgb(4, 120, 87),
                            Stroke::new(1.5_f32, colors::ACCENT_SUCCESS_HOVER),
                            colors::TEXT_WHITE,
                        )
                    } else if is_hovered {
                        (
                            Color32::from_rgb(5, 150, 105),
                            Stroke::new(1.5_f32, colors::ACCENT_SUCCESS_HOVER),
                            colors::TEXT_WHITE,
                        )
                    } else {
                        (
                            Color32::from_rgb(6, 95, 70),
                            Stroke::new(1.0_f32, colors::ACCENT_SUCCESS),
                            colors::TEXT_WHITE,
                        )
                    }
                }
                ButtonVariant::Danger => {
                    if is_clicked || self.selected {
                        (
                            Color32::from_rgb(185, 28, 28),
                            Stroke::new(1.5_f32, colors::ACCENT_DANGER_HOVER),
                            colors::TEXT_WHITE,
                        )
                    } else if is_hovered {
                        (
                            Color32::from_rgb(220, 38, 38),
                            Stroke::new(1.5_f32, colors::ACCENT_DANGER_HOVER),
                            colors::TEXT_WHITE,
                        )
                    } else {
                        (
                            Color32::from_rgb(153, 27, 27),
                            Stroke::new(1.0_f32, colors::ACCENT_DANGER),
                            colors::TEXT_WHITE,
                        )
                    }
                }
                ButtonVariant::Warning => {
                    if is_clicked || self.selected {
                        (
                            Color32::from_rgb(180, 83, 9),
                            Stroke::new(1.5_f32, colors::ACCENT_WARNING_HOVER),
                            colors::TEXT_WHITE,
                        )
                    } else if is_hovered {
                        (
                            Color32::from_rgb(217, 119, 6),
                            Stroke::new(1.5_f32, colors::ACCENT_WARNING_HOVER),
                            colors::TEXT_WHITE,
                        )
                    } else {
                        (
                            Color32::from_rgb(146, 64, 14),
                            Stroke::new(1.0_f32, colors::ACCENT_WARNING),
                            colors::TEXT_WHITE,
                        )
                    }
                }
                ButtonVariant::Secondary => {
                    if is_clicked || self.selected {
                        (
                            colors::BG_CARD_ACTIVE,
                            Stroke::new(1.5_f32, colors::ACCENT_PRIMARY),
                            colors::TEXT_WHITE,
                        )
                    } else if is_hovered {
                        (
                            colors::BG_CARD_HOVER,
                            Stroke::new(1.0_f32, colors::BORDER_HOVER),
                            colors::TEXT_WHITE,
                        )
                    } else {
                        (
                            colors::BG_CARD,
                            Stroke::new(1.0_f32, colors::BORDER_CARD),
                            colors::TEXT_PRIMARY,
                        )
                    }
                }
                ButtonVariant::Ghost => {
                    if is_clicked || self.selected {
                        (
                            Color32::from_rgba_premultiplied(59, 130, 246, 70),
                            Stroke::new(1.0_f32, colors::ACCENT_PRIMARY),
                            colors::TEXT_WHITE,
                        )
                    } else if is_hovered {
                        (
                            Color32::from_rgba_premultiplied(255, 255, 255, 24),
                            Stroke::new(1.0_f32, colors::BORDER_HOVER),
                            colors::TEXT_WHITE,
                        )
                    } else {
                        (Color32::TRANSPARENT, Stroke::NONE, colors::TEXT_SECONDARY)
                    }
                }
            };

            let rounding = Rounding::same(7.0);

            // Rendu du fond du bouton
            ui.painter().rect(rect, rounding, bg_fill, border_stroke);

            if let Some(sc_g) = shortcut_galley {
                // Centrer harmonieusement l'ensemble [Texte + Espace + Badge] dans le bouton
                let spacing = 8.0;
                let sc_rect_height = (sc_g.size().y + 4.0).max(18.0);
                let content_w = galley.size().x + spacing + shortcut_box_width;
                let start_x = rect.center().x - content_w * 0.5;

                // Rendu du badge de raccourci clavier
                let sc_rect = Rect::from_min_size(
                    Pos2::new(
                        start_x + galley.size().x + spacing,
                        rect.center().y - sc_rect_height * 0.5,
                    ),
                    Vec2::new(shortcut_box_width, sc_rect_height),
                );

                // Position du texte principal
                let text_pos = Pos2::new(start_x, rect.center().y - galley.size().y * 0.5);
                ui.painter().galley(text_pos, galley, text_color);

                // Fond capsule du raccourci
                ui.painter().rect(
                    sc_rect,
                    Rounding::same(4.0),
                    Color32::from_rgba_premultiplied(0, 0, 0, 110),
                    Stroke::new(1.0_f32, Color32::from_white_alpha(45)),
                );

                let sc_pos = Pos2::new(
                    sc_rect.center().x - sc_g.size().x * 0.5,
                    sc_rect.center().y - sc_g.size().y * 0.5,
                );
                ui.painter().galley(sc_pos, sc_g, colors::TEXT_WHITE);
            } else {
                // Bouton standard : centrage parfait horizontal et vertical
                let text_pos = Pos2::new(
                    rect.center().x - galley.size().x * 0.5,
                    rect.center().y - galley.size().y * 0.5,
                );
                ui.painter().galley(text_pos, galley, text_color);
            }
        }

        response
    }
}
