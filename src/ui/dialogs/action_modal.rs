use crate::macro_core::{ActionType, MacroAction};
use crate::ui::i18n::Language;
use crate::ui::theme::{self, colors};
use crate::ui::widgets::{ButtonVariant, GlassButton};
use eframe::egui::{self, DragValue, Frame, Key, Margin, Rounding, Stroke, Ui, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionModalTab {
    Keyboard,
    Mouse,
    Wait,
    Image,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionModalTarget {
    New,
    Edit(usize),
}

pub struct ActionEditorModal {
    pub is_open: bool,
    pub target: ActionModalTarget,
    pub current_tab: ActionModalTab,
    pub delay_ms: u64,

    // Keyboard fields
    pub key_name: String,
    pub vk_code: u16,
    pub is_extended: bool,
    pub is_key_press: bool,
    pub is_listening_key: bool,

    // Mouse fields
    pub mouse_sub_type: usize, // 0: Press, 1: Release, 2: Move, 3: MoveRel, 4: Scroll
    pub mouse_button: u8,      // 1: Left, 2: Right, 3: Middle, 4: Other
    pub mouse_x: f64,
    pub mouse_y: f64,
    pub mouse_dx: i32,
    pub mouse_dy: i32,
    pub scroll_dx: f64,
    pub scroll_dy: f64,

    // Wait fields
    pub wait_duration_ms: u64,

    // Image fields
    pub image_path: String,
    pub image_timeout_ms: u64,
}

impl Default for ActionEditorModal {
    fn default() -> Self {
        Self::new()
    }
}

impl ActionEditorModal {
    pub fn new() -> Self {
        Self {
            is_open: false,
            target: ActionModalTarget::New,
            current_tab: ActionModalTab::Keyboard,
            delay_ms: 10,

            key_name: "A".to_string(),
            vk_code: 0x41,
            is_extended: false,
            is_key_press: true,
            is_listening_key: false,

            mouse_sub_type: 0,
            mouse_button: 1,
            mouse_x: 500.0,
            mouse_y: 500.0,
            mouse_dx: 0,
            mouse_dy: 0,
            scroll_dx: 0.0,
            scroll_dy: 120.0,

            wait_duration_ms: 100,

            image_path: String::new(),
            image_timeout_ms: 5000,
        }
    }

    pub fn open_for_new(&mut self, default_tab: ActionModalTab) {
        self.is_open = true;
        self.target = ActionModalTarget::New;
        self.current_tab = default_tab;
        self.is_listening_key = false;
    }

    pub fn open_for_edit(&mut self, index: usize, action: &MacroAction) {
        self.is_open = true;
        self.target = ActionModalTarget::Edit(index);
        self.delay_ms = action.delay_ms;
        self.is_listening_key = false;

        match &action.action_type {
            ActionType::KeyPress(name, vk, ext) => {
                self.current_tab = ActionModalTab::Keyboard;
                self.is_key_press = true;
                self.key_name = name.clone();
                self.vk_code = *vk;
                self.is_extended = *ext;
            }
            ActionType::KeyRelease(name, vk, ext) => {
                self.current_tab = ActionModalTab::Keyboard;
                self.is_key_press = false;
                self.key_name = name.clone();
                self.vk_code = *vk;
                self.is_extended = *ext;
            }
            ActionType::MousePress(btn, x, y) => {
                self.current_tab = ActionModalTab::Mouse;
                self.mouse_sub_type = 0;
                self.mouse_button = *btn;
                self.mouse_x = *x;
                self.mouse_y = *y;
            }
            ActionType::MouseRelease(btn, x, y) => {
                self.current_tab = ActionModalTab::Mouse;
                self.mouse_sub_type = 1;
                self.mouse_button = *btn;
                self.mouse_x = *x;
                self.mouse_y = *y;
            }
            ActionType::MouseMove(x, y) => {
                self.current_tab = ActionModalTab::Mouse;
                self.mouse_sub_type = 2;
                self.mouse_x = *x;
                self.mouse_y = *y;
            }
            ActionType::MouseMoveRelative(dx, dy) => {
                self.current_tab = ActionModalTab::Mouse;
                self.mouse_sub_type = 3;
                self.mouse_dx = *dx;
                self.mouse_dy = *dy;
            }
            ActionType::Scroll(dx, dy) => {
                self.current_tab = ActionModalTab::Mouse;
                self.mouse_sub_type = 4;
                self.scroll_dx = *dx;
                self.scroll_dy = *dy;
            }
            ActionType::Wait(ms) => {
                self.current_tab = ActionModalTab::Wait;
                self.wait_duration_ms = *ms;
            }
            ActionType::WaitImage(path, timeout) => {
                self.current_tab = ActionModalTab::Image;
                self.image_path = path.clone();
                self.image_timeout_ms = *timeout;
            }
        }
    }

    pub fn build_action(&self) -> MacroAction {
        let action_type = match self.current_tab {
            ActionModalTab::Keyboard => {
                if self.is_key_press {
                    ActionType::KeyPress(self.key_name.clone(), self.vk_code, self.is_extended)
                } else {
                    ActionType::KeyRelease(self.key_name.clone(), self.vk_code, self.is_extended)
                }
            }
            ActionModalTab::Mouse => match self.mouse_sub_type {
                0 => ActionType::MousePress(self.mouse_button, self.mouse_x, self.mouse_y),
                1 => ActionType::MouseRelease(self.mouse_button, self.mouse_x, self.mouse_y),
                2 => ActionType::MouseMove(self.mouse_x, self.mouse_y),
                3 => ActionType::MouseMoveRelative(self.mouse_dx, self.mouse_dy),
                _ => ActionType::Scroll(self.scroll_dx, self.scroll_dy),
            },
            ActionModalTab::Wait => ActionType::Wait(self.wait_duration_ms),
            ActionModalTab::Image => {
                ActionType::WaitImage(self.image_path.clone(), self.image_timeout_ms)
            }
        };

        MacroAction {
            action_type,
            delay_ms: self.delay_ms,
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        lang: Language,
    ) -> Option<(ActionModalTarget, MacroAction)> {
        if !self.is_open {
            return None;
        }

        // Gestion de l'écoute interactive de touche clavier
        if self.is_listening_key {
            ctx.input(|i| {
                for event in &i.events {
                    if let egui::Event::Key {
                        key,
                        pressed: true,
                        repeat: false,
                        ..
                    } = event
                    {
                        if *key == Key::Escape {
                            self.is_listening_key = false;
                        } else if let Some((name, vk, ext)) = egui_key_to_vk(*key) {
                            self.key_name = name.to_string();
                            self.vk_code = vk;
                            self.is_extended = ext;
                            self.is_listening_key = false;
                        }
                    }
                }
            });
        }

        let mut confirmed_action = None;
        let mut should_close = false;

        let title = match self.target {
            ActionModalTarget::New => lang.modal_add_action_title(),
            ActionModalTarget::Edit(_) => lang.modal_edit_action_title(),
        };

        egui::Window::new(title)
            .frame(theme::modal_frame())
            .collapsible(false)
            .resizable(false)
            .default_size(Vec2::new(490.0, 430.0))
            .anchor(egui::Align2::CENTER_CENTER, Vec2::new(0.0, 0.0))
            .show(ctx, |ui| {
                ui.add_space(2.0);

                // Onglets de catégorie
                ui.horizontal(|ui| {
                    let tabs = [
                        (ActionModalTab::Keyboard, lang.tab_keyboard()),
                        (ActionModalTab::Mouse, lang.tab_mouse()),
                        (ActionModalTab::Wait, lang.tab_wait()),
                        (ActionModalTab::Image, lang.tab_image()),
                    ];

                    for (tab, label) in tabs {
                        let is_active = self.current_tab == tab;
                        let btn =
                            GlassButton::new(label)
                                .selected(is_active)
                                .variant(if is_active {
                                    ButtonVariant::Primary
                                } else {
                                    ButtonVariant::Secondary
                                });
                        if ui.add(btn).clicked() {
                            self.current_tab = tab;
                            self.is_listening_key = false;
                        }
                    }
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // Contenu spécifique selon l'onglet
                Frame::none()
                    .fill(colors::BG_CARD)
                    .stroke(Stroke::new(1.0_f32, colors::BORDER_CARD))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(Margin::same(12.0))
                    .show(ui, |ui| {
                        match self.current_tab {
                            ActionModalTab::Keyboard => self.render_keyboard_fields(ui, lang),
                            ActionModalTab::Mouse => self.render_mouse_fields(ui, lang),
                            ActionModalTab::Wait => self.render_wait_fields(ui, lang),
                            ActionModalTab::Image => self.render_image_fields(ui, lang),
                        }

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(6.0);

                        // Champ délai universel avec présélections rapides
                        ui.horizontal(|ui| {
                            ui.label(lang.delay_label());
                            ui.add(
                                DragValue::new(&mut self.delay_ms)
                                    .range(0..=60000)
                                    .speed(5.0)
                                    .suffix(" ms"),
                            );

                            let quick_delays = [0, 5, 10, 25, 50, 100];
                            for d in quick_delays {
                                if ui.small_button(format!("{}ms", d)).clicked() {
                                    self.delay_ms = d;
                                }
                            }
                        });
                    });

                ui.add_space(10.0);

                // Boutons d'action Valider / Annuler
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let ok_btn = GlassButton::new(lang.modal_save())
                            .icon("💾")
                            .variant(ButtonVariant::Success);
                        if ui.add(ok_btn).clicked() {
                            confirmed_action = Some((self.target, self.build_action()));
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
            self.is_listening_key = false;
        }

        confirmed_action
    }

    fn render_keyboard_fields(&mut self, ui: &mut Ui, lang: Language) {
        ui.horizontal(|ui| {
            ui.label(lang.event_type_label());
            ui.radio_value(&mut self.is_key_press, true, lang.action_key_press());
            ui.radio_value(&mut self.is_key_press, false, lang.action_key_release());
        });

        ui.add_space(6.0);

        // Bouton de capture interactive
        let capture_text = if self.is_listening_key {
            lang.key_listening_prompt()
        } else {
            lang.capture_key_btn()
        };
        let capture_btn = GlassButton::new(capture_text)
            .icon(if self.is_listening_key {
                "🔴"
            } else {
                "🎯"
            })
            .variant(if self.is_listening_key {
                ButtonVariant::Danger
            } else {
                ButtonVariant::Primary
            });

        if ui.add(capture_btn).clicked() {
            self.is_listening_key = !self.is_listening_key;
        }

        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label(lang.key_label());
            ui.text_edit_singleline(&mut self.key_name);
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label(lang.vk_code_label());
            let mut vk_u32 = self.vk_code as u32;
            if ui
                .add(
                    DragValue::new(&mut vk_u32)
                        .range(0..=255)
                        .speed(1.0)
                        .hexadecimal(2, true, true),
                )
                .changed()
            {
                self.vk_code = vk_u32 as u16;
            }
            ui.label(format!("(Hex: {:#04X})", self.vk_code));
        });

        ui.add_space(4.0);

        ui.checkbox(&mut self.is_extended, lang.extended_key_label());
    }

    fn render_mouse_fields(&mut self, ui: &mut Ui, lang: Language) {
        ui.horizontal(|ui| {
            ui.label(lang.mouse_action_type());
            egui::ComboBox::from_id_salt("mouse_sub_type_combo")
                .selected_text(match self.mouse_sub_type {
                    0 => lang.action_mouse_press(),
                    1 => lang.action_mouse_release(),
                    2 => lang.action_mouse_pos(),
                    3 => lang.action_mouse_relative(),
                    _ => lang.action_scroll(),
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.mouse_sub_type, 0, lang.action_mouse_press());
                    ui.selectable_value(&mut self.mouse_sub_type, 1, lang.action_mouse_release());
                    ui.selectable_value(&mut self.mouse_sub_type, 2, lang.action_mouse_pos());
                    ui.selectable_value(&mut self.mouse_sub_type, 3, lang.action_mouse_relative());
                    ui.selectable_value(&mut self.mouse_sub_type, 4, lang.action_scroll());
                });
        });

        ui.add_space(4.0);

        if self.mouse_sub_type == 0 || self.mouse_sub_type == 1 {
            ui.horizontal(|ui| {
                ui.label(lang.mouse_btn_label());
                egui::ComboBox::from_id_salt("mouse_btn_combo")
                    .selected_text(match self.mouse_button {
                        1 => lang.mouse_btn_left(),
                        2 => lang.mouse_btn_right(),
                        3 => lang.mouse_btn_middle(),
                        _ => "Autre (4+)",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.mouse_button, 1, lang.mouse_btn_left());
                        ui.selectable_value(&mut self.mouse_button, 2, lang.mouse_btn_right());
                        ui.selectable_value(&mut self.mouse_button, 3, lang.mouse_btn_middle());
                    });
            });
            ui.add_space(4.0);
        }

        if self.mouse_sub_type <= 2 {
            // Bouton pour capturer la position actuelle du curseur avec GetCursorPos
            let cap_pos_btn = GlassButton::new(lang.capture_cursor_pos_btn())
                .icon("🎯")
                .variant(ButtonVariant::Secondary);
            if ui.add(cap_pos_btn).clicked() {
                self.capture_current_cursor();
            }

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("X :");
                ui.add(
                    DragValue::new(&mut self.mouse_x)
                        .range(0.0..=10000.0)
                        .speed(1.0),
                );
                ui.label("Y :");
                ui.add(
                    DragValue::new(&mut self.mouse_y)
                        .range(0.0..=10000.0)
                        .speed(1.0),
                );
            });
        } else if self.mouse_sub_type == 3 {
            ui.horizontal(|ui| {
                ui.label("ΔX :");
                ui.add(
                    DragValue::new(&mut self.mouse_dx)
                        .range(-10000..=10000)
                        .speed(1.0),
                );
                ui.label("ΔY :");
                ui.add(
                    DragValue::new(&mut self.mouse_dy)
                        .range(-10000..=10000)
                        .speed(1.0),
                );
            });
        } else {
            ui.horizontal(|ui| {
                ui.label("ΔX :");
                ui.add(
                    DragValue::new(&mut self.scroll_dx)
                        .range(-1000.0..=1000.0)
                        .speed(1.0),
                );
                ui.label("ΔY :");
                ui.add(
                    DragValue::new(&mut self.scroll_dy)
                        .range(-1000.0..=1000.0)
                        .speed(10.0),
                );
            });
        }
    }

    pub fn capture_current_cursor(&mut self) {
        #[cfg(windows)]
        {
            use winapi::shared::windef::POINT;
            use winapi::um::winuser::GetCursorPos;
            let mut pt = POINT { x: 0, y: 0 };
            unsafe {
                if GetCursorPos(&mut pt) != 0 {
                    self.mouse_x = pt.x as f64;
                    self.mouse_y = pt.y as f64;
                }
            }
        }
    }

    fn render_wait_fields(&mut self, ui: &mut Ui, lang: Language) {
        ui.horizontal(|ui| {
            ui.label(lang.wait_duration_label());
            ui.add(
                DragValue::new(&mut self.wait_duration_ms)
                    .range(1..=600000)
                    .speed(50.0)
                    .suffix(" ms"),
            );
        });

        ui.add_space(6.0);

        ui.horizontal(|ui| {
            let presets = [50, 100, 250, 500, 1000, 2000, 5000];
            ui.label(lang.presets_label());
            for p in presets {
                if ui.small_button(format!("{}ms", p)).clicked() {
                    self.wait_duration_ms = p;
                }
            }
        });
    }

    fn render_image_fields(&mut self, ui: &mut Ui, lang: Language) {
        ui.horizontal(|ui| {
            ui.label(lang.stop_image_path_label());
            ui.text_edit_singleline(&mut self.image_path);
            if ui.button(lang.browse_file_btn()).clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter(
                        "Images (*.png, *.jpg, *.bmp)",
                        &["png", "jpg", "jpeg", "bmp"],
                    )
                    .pick_file()
                {
                    if let Some(s) = path.to_str() {
                        self.image_path = s.to_string();
                    }
                }
            }
        });

        ui.add_space(6.0);

        // Présélections d'images intégrées
        ui.horizontal(|ui| {
            ui.label(lang.embedded_images_label());
            if ui.small_button("🎯 extreme.png").clicked() {
                self.image_path = "embedded://extreme.png".to_string();
            }
            if ui.small_button("❌ failed.PNG").clicked() {
                self.image_path = "embedded://failed.PNG".to_string();
            }
        });

        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label(lang.timeout_label());
            ui.add(
                DragValue::new(&mut self.image_timeout_ms)
                    .range(100..=120000)
                    .speed(100.0)
                    .suffix(" ms"),
            );

            let timeout_presets = [1000, 2000, 5000, 10000];
            for t in timeout_presets {
                if ui.small_button(format!("{}s", t / 1000)).clicked() {
                    self.image_timeout_ms = t;
                }
            }
        });
    }
}

/// Convertit un egui::Key en (Nom de la touche, Code VK Windows, Touche étendue)
pub fn egui_key_to_vk(key: Key) -> Option<(&'static str, u16, bool)> {
    match key {
        Key::A => Some(("A", 0x41, false)),
        Key::B => Some(("B", 0x42, false)),
        Key::C => Some(("C", 0x43, false)),
        Key::D => Some(("D", 0x44, false)),
        Key::E => Some(("E", 0x45, false)),
        Key::F => Some(("F", 0x46, false)),
        Key::G => Some(("G", 0x47, false)),
        Key::H => Some(("H", 0x48, false)),
        Key::I => Some(("I", 0x49, false)),
        Key::J => Some(("J", 0x4A, false)),
        Key::K => Some(("K", 0x4B, false)),
        Key::L => Some(("L", 0x4C, false)),
        Key::M => Some(("M", 0x4D, false)),
        Key::N => Some(("N", 0x4E, false)),
        Key::O => Some(("O", 0x4F, false)),
        Key::P => Some(("P", 0x50, false)),
        Key::Q => Some(("Q", 0x51, false)),
        Key::R => Some(("R", 0x52, false)),
        Key::S => Some(("S", 0x53, false)),
        Key::T => Some(("T", 0x54, false)),
        Key::U => Some(("U", 0x55, false)),
        Key::V => Some(("V", 0x56, false)),
        Key::W => Some(("W", 0x57, false)),
        Key::X => Some(("X", 0x58, false)),
        Key::Y => Some(("Y", 0x59, false)),
        Key::Z => Some(("Z", 0x5A, false)),

        Key::Num0 => Some(("0", 0x30, false)),
        Key::Num1 => Some(("1", 0x31, false)),
        Key::Num2 => Some(("2", 0x32, false)),
        Key::Num3 => Some(("3", 0x33, false)),
        Key::Num4 => Some(("4", 0x34, false)),
        Key::Num5 => Some(("5", 0x35, false)),
        Key::Num6 => Some(("6", 0x36, false)),
        Key::Num7 => Some(("7", 0x37, false)),
        Key::Num8 => Some(("8", 0x38, false)),
        Key::Num9 => Some(("9", 0x39, false)),

        Key::F1 => Some(("F1", 0x70, false)),
        Key::F2 => Some(("F2", 0x71, false)),
        Key::F3 => Some(("F3", 0x72, false)),
        Key::F4 => Some(("F4", 0x73, false)),
        Key::F5 => Some(("F5", 0x74, false)),
        Key::F6 => Some(("F6", 0x75, false)),
        Key::F7 => Some(("F7", 0x76, false)),
        Key::F8 => Some(("F8", 0x77, false)),
        Key::F9 => Some(("F9", 0x78, false)),
        Key::F10 => Some(("F10", 0x79, false)),
        Key::F11 => Some(("F11", 0x7A, false)),
        Key::F12 => Some(("F12", 0x7B, false)),

        Key::Space => Some(("Space", 0x20, false)),
        Key::Enter => Some(("Enter", 0x0D, false)),
        Key::Backspace => Some(("Backspace", 0x08, false)),
        Key::Tab => Some(("Tab", 0x09, false)),
        Key::Escape => Some(("Escape", 0x1B, false)),
        Key::Insert => Some(("Insert", 0x2D, true)),
        Key::Delete => Some(("Delete", 0x2E, true)),
        Key::Home => Some(("Home", 0x24, true)),
        Key::End => Some(("End", 0x23, true)),
        Key::PageUp => Some(("PageUp", 0x21, true)),
        Key::PageDown => Some(("PageDown", 0x22, true)),
        Key::ArrowLeft => Some(("ArrowLeft", 0x25, true)),
        Key::ArrowUp => Some(("ArrowUp", 0x26, true)),
        Key::ArrowRight => Some(("ArrowRight", 0x27, true)),
        Key::ArrowDown => Some(("ArrowDown", 0x28, true)),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_modal_initial_state() {
        let modal = ActionEditorModal::new();
        assert!(!modal.is_open);
        assert_eq!(modal.target, ActionModalTarget::New);
        assert_eq!(modal.current_tab, ActionModalTab::Keyboard);
        assert!(!modal.is_listening_key);
    }

    #[test]
    fn test_action_modal_build_keyboard() {
        let mut modal = ActionEditorModal::new();
        modal.open_for_new(ActionModalTab::Keyboard);
        modal.key_name = "Enter".to_string();
        modal.vk_code = 0x0D;
        modal.is_key_press = true;
        modal.delay_ms = 45;

        let action = modal.build_action();
        assert_eq!(action.delay_ms, 45);
        assert_eq!(
            action.action_type,
            ActionType::KeyPress("Enter".to_string(), 0x0D, false)
        );
    }

    #[test]
    fn test_action_modal_edit_existing() {
        let original = MacroAction {
            action_type: ActionType::Wait(500),
            delay_ms: 25,
        };

        let mut modal = ActionEditorModal::new();
        modal.open_for_edit(3, &original);

        assert!(modal.is_open);
        assert_eq!(modal.target, ActionModalTarget::Edit(3));
        assert_eq!(modal.current_tab, ActionModalTab::Wait);
        assert_eq!(modal.wait_duration_ms, 500);
        assert_eq!(modal.delay_ms, 25);
    }

    #[test]
    fn test_egui_key_mapping() {
        let (name, vk, ext) = egui_key_to_vk(Key::F5).unwrap();
        assert_eq!(name, "F5");
        assert_eq!(vk, 0x74);
        assert!(!ext);

        let (name, vk, ext) = egui_key_to_vk(Key::ArrowDown).unwrap();
        assert_eq!(name, "ArrowDown");
        assert_eq!(vk, 0x28);
        assert!(ext);
    }
}
