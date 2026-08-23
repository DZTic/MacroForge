use crate::ui::theme::colors;
use crate::ui::widgets::{ButtonVariant, GlassButton};
use eframe::egui::{self, DragValue, Response, Ui, Widget};

/// Champ de saisie numérique de précision avec curseur et boutons d'ajustement
pub struct NumericInputWithSlider<'a, T> {
    value: &'a mut T,
    label: &'a str,
    range: std::ops::RangeInclusive<T>,
    step: Option<f64>,
    suffix: Option<&'a str>,
}

impl<'a> NumericInputWithSlider<'a, u64> {
    pub fn new_u64(
        value: &'a mut u64,
        label: &'a str,
        range: std::ops::RangeInclusive<u64>,
    ) -> Self {
        Self {
            value,
            label,
            range,
            step: Some(10.0),
            suffix: None,
        }
    }

    pub fn suffix(mut self, suffix: &'a str) -> Self {
        self.suffix = Some(suffix);
        self
    }

    pub fn step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }
}

impl<'a> Widget for NumericInputWithSlider<'a, u64> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal_centered(|ui| {
            ui.label(
                egui::RichText::new(self.label)
                    .color(colors::TEXT_PRIMARY)
                    .size(13.0),
            );

            let min = *self.range.start();
            let max = *self.range.end();

            // Bouton de décrémentation rapide
            let dec_btn = GlassButton::new("➖")
                .compact(true)
                .variant(ButtonVariant::Secondary);
            if ui.add(dec_btn).clicked() {
                let step_val = self.step.unwrap_or(10.0) as u64;
                *self.value = self.value.saturating_sub(step_val).max(min);
            }

            // DragValue avec style sombre
            let mut drag = DragValue::new(self.value)
                .range(self.range)
                .speed(self.step.unwrap_or(1.0));

            if let Some(suf) = self.suffix {
                drag = drag.suffix(format!(" {}", suf));
            }

            let response = ui.add(drag);

            // Bouton d'incrémentation rapide
            let inc_btn = GlassButton::new("➕")
                .compact(true)
                .variant(ButtonVariant::Secondary);
            if ui.add(inc_btn).clicked() {
                let step_val = self.step.unwrap_or(10.0) as u64;
                *self.value = (*self.value + step_val).min(max);
            }

            response
        })
        .inner
    }
}
