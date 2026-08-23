use crate::ui::theme::colors;
use eframe::egui::{self, Color32, Frame, Margin, Rounding, Stroke, ViewportBuilder, ViewportId};

pub struct TransparentOverlay {
    pub is_visible: bool,
    pub current_action_idx: usize,
    pub total_actions: usize,
    pub action_type_label: String,
    pub action_detail: String,
    pub target_x: Option<f64>,
    pub target_y: Option<f64>,
    pub win32_configured: bool,
}

impl Default for TransparentOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl TransparentOverlay {
    pub fn new() -> Self {
        Self {
            is_visible: false,
            current_action_idx: 0,
            total_actions: 0,
            action_type_label: String::new(),
            action_detail: String::new(),
            target_x: None,
            target_y: None,
            win32_configured: false,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, is_playing: bool) {
        if !self.is_visible && !is_playing {
            self.win32_configured = false;
            return;
        }

        let viewport_id = ViewportId::from_hash_of("macroforge_overlay");

        ctx.show_viewport_immediate(
            viewport_id,
            ViewportBuilder::default()
                .with_title("MacroForge Overlay")
                .with_inner_size([380.0, 68.0])
                .with_min_inner_size([320.0, 56.0])
                .with_decorations(false)
                .with_transparent(true)
                .with_always_on_top()
                .with_mouse_passthrough(true)
                .with_resizable(false),
            |ctx, _class| {
                #[cfg(windows)]
                if !self.win32_configured {
                    apply_win32_overlay_styles();
                    self.win32_configured = true;
                }

                let hud_frame = Frame::none()
                    .fill(Color32::from_rgba_premultiplied(12, 16, 24, 230))
                    .stroke(Stroke::new(
                        1.5,
                        Color32::from_rgba_premultiplied(59, 130, 246, 220),
                    ))
                    .rounding(Rounding::same(10.0))
                    .inner_margin(Margin::symmetric(14.0, 8.0));

                egui::CentralPanel::default()
                    .frame(hud_frame)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("⚡")
                                    .color(Color32::from_rgb(59, 130, 246))
                                    .size(22.0),
                            );

                            ui.add_space(4.0);

                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "Action #{}/{}",
                                            self.current_action_idx, self.total_actions
                                        ))
                                        .color(Color32::from_rgb(147, 197, 253))
                                        .size(11.5)
                                        .strong(),
                                    );

                                    if !self.action_type_label.is_empty() {
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "• {}",
                                                self.action_type_label
                                            ))
                                            .color(Color32::WHITE)
                                            .size(12.0)
                                            .strong(),
                                        );
                                    }
                                });

                                if !self.action_detail.is_empty() {
                                    ui.label(
                                        egui::RichText::new(&self.action_detail)
                                            .color(colors::TEXT_MUTED)
                                            .size(10.5),
                                    );
                                }
                            });
                        });
                    });
            },
        );
    }
}

#[cfg(windows)]
pub fn apply_win32_overlay_styles() {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::winuser::*;

    let title: Vec<u16> = OsStr::new("MacroForge Overlay\0").encode_wide().collect();
    unsafe {
        let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
        if !hwnd.is_null() {
            // WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_TOPMOST
            let ex_style = WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_TOPMOST;
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style as isize);

            // WDA_EXCLUDEFROMCAPTURE = 0x00000011 (exclusion des captures d'écran GDI)
            const WDA_EXCLUDEFROMCAPTURE: u32 = 0x00000011;
            SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlay_initial_state() {
        let overlay = TransparentOverlay::new();
        assert!(!overlay.is_visible);
        assert_eq!(overlay.current_action_idx, 0);
        assert_eq!(overlay.total_actions, 0);
        assert!(!overlay.win32_configured);
    }
}
