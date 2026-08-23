use crate::macro_core;
use crate::ui::i18n::Language;
use crate::ui::theme::{self, colors};
use crate::ui::widgets::{ButtonVariant, GlassButton};
use eframe::egui::{self, DragValue, Frame, Margin, Rounding, Stroke, Vec2};

pub struct StopImageConfigModal {
    pub is_open: bool,
    pub enabled: bool,
    pub path: String,
    pub timeout_ms: u64,
}

impl Default for StopImageConfigModal {
    fn default() -> Self {
        Self::new()
    }
}

impl StopImageConfigModal {
    pub fn new() -> Self {
        let (current_path, current_timeout) = macro_core::get_stop_image();
        Self {
            is_open: false,
            enabled: current_path.is_some(),
            path: current_path.unwrap_or_default(),
            timeout_ms: current_timeout,
        }
    }

    pub fn open(&mut self) {
        let (current_path, current_timeout) = macro_core::get_stop_image();
        self.enabled = current_path.is_some();
        self.path = current_path.unwrap_or_default();
        self.timeout_ms = current_timeout;
        self.is_open = true;
    }

    pub fn show(&mut self, ctx: &egui::Context, lang: Language) -> bool {
        if !self.is_open {
            return false;
        }

        let mut should_close = false;
        let mut changed = false;

        egui::Window::new(lang.stop_image_modal_title())
            .frame(theme::modal_frame())
            .collapsible(false)
            .resizable(false)
            .default_size(Vec2::new(490.0, 270.0))
            .anchor(egui::Align2::CENTER_CENTER, Vec2::new(0.0, 0.0))
            .show(ctx, |ui| {
                ui.add_space(2.0);

                Frame::none()
                    .fill(colors::BG_CARD)
                    .stroke(Stroke::new(1.0_f32, colors::BORDER_CARD))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.checkbox(&mut self.enabled, lang.stop_image_enable());

                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            ui.label(lang.stop_image_path_label());
                            ui.text_edit_singleline(&mut self.path);
                            if ui.button(lang.browse_file_btn()).clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter(
                                        "Images (*.png, *.jpg, *.bmp)",
                                        &["png", "jpg", "jpeg", "bmp"],
                                    )
                                    .pick_file()
                                {
                                    if let Some(s) = path.to_str() {
                                        self.path = s.to_string();
                                    }
                                }
                            }
                        });

                        ui.add_space(6.0);

                        // Presets images intégrées
                        ui.horizontal(|ui| {
                            ui.label(lang.embedded_images_label());
                            if ui.small_button("🎯 extreme.png").clicked() {
                                self.path = "embedded://extreme.png".to_string();
                            }
                            if ui.small_button("❌ failed.PNG").clicked() {
                                self.path = "embedded://failed.PNG".to_string();
                            }
                        });

                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            ui.label(lang.timeout_label());
                            ui.add(
                                DragValue::new(&mut self.timeout_ms)
                                    .range(100..=60000)
                                    .speed(100.0)
                                    .suffix(" ms"),
                            );

                            let timeout_presets = [1000, 2000, 5000, 10000];
                            for t in timeout_presets {
                                if ui.small_button(format!("{}s", t / 1000)).clicked() {
                                    self.timeout_ms = t;
                                }
                            }
                        });
                    });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let save_btn = GlassButton::new(lang.modal_save())
                            .icon("💾")
                            .variant(ButtonVariant::Success);
                        if ui.add(save_btn).clicked() {
                            let path_opt = if self.enabled && !self.path.trim().is_empty() {
                                Some(self.path.trim().to_string())
                            } else {
                                None
                            };
                            macro_core::set_stop_image(path_opt, self.timeout_ms);
                            changed = true;
                            should_close = true;
                        }

                        ui.add_space(8.0);

                        let cancel_btn =
                            GlassButton::new(lang.modal_cancel()).variant(ButtonVariant::Ghost);
                        if ui.add(cancel_btn).clicked() {
                            should_close = true;
                        }
                    });
                });
            });

        if should_close {
            self.is_open = false;
        }

        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stop_image_modal_state() {
        let mut modal = StopImageConfigModal::new();
        assert!(!modal.is_open);
        modal.open();
        assert!(modal.is_open);
    }
}
