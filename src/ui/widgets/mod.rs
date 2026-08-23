pub mod action_card;
pub mod glass_button;
pub mod numeric_input;
pub mod status_badge;
pub mod toggle_switch;

pub use action_card::ActionCard;
pub use glass_button::{ButtonVariant, GlassButton};
pub use numeric_input::NumericInputWithSlider;
pub use status_badge::{StatusBadge, StatusKind};
pub use toggle_switch::CustomToggleSwitch;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macro_core::{ActionType, MacroAction};
    use eframe::egui::Vec2;

    #[test]
    fn test_glass_button_builder() {
        let _btn = GlassButton::new("Enregistrer")
            .icon("🔴")
            .shortcut("F8")
            .variant(ButtonVariant::Danger)
            .min_size(Vec2::new(100.0, 35.0))
            .selected(true)
            .enabled(true);
    }

    #[test]
    fn test_status_badge_variants() {
        let _badge_rec = StatusBadge::new(StatusKind::Recording);
        let _badge_play = StatusBadge::new(StatusKind::Playing).label("TEST PLAY");
        let _badge_idle = StatusBadge::new(StatusKind::Idle);
        let _badge_paused = StatusBadge::new(StatusKind::Paused);
    }

    #[test]
    fn test_toggle_switch() {
        let mut val = false;
        let _switch = CustomToggleSwitch::new(&mut val).label("Test Toggle");
    }

    #[test]
    fn test_numeric_input() {
        let mut delay: u64 = 100;
        let _num_input = NumericInputWithSlider::new_u64(&mut delay, "Délai", 0..=5000)
            .suffix("ms")
            .step(50.0);
    }

    #[test]
    fn test_action_cards_for_all_types() {
        let actions = vec![
            MacroAction {
                action_type: ActionType::KeyPress("A".into(), 0x41, false),
                delay_ms: 15,
            },
            MacroAction {
                action_type: ActionType::KeyRelease("A".into(), 0x41, false),
                delay_ms: 50,
            },
            MacroAction {
                action_type: ActionType::MouseMove(500.0, 300.0),
                delay_ms: 10,
            },
            MacroAction {
                action_type: ActionType::MouseMoveRelative(10, -5),
                delay_ms: 10,
            },
            MacroAction {
                action_type: ActionType::MousePress(1, 500.0, 300.0),
                delay_ms: 20,
            },
            MacroAction {
                action_type: ActionType::MouseRelease(1, 500.0, 300.0),
                delay_ms: 30,
            },
            MacroAction {
                action_type: ActionType::Scroll(0.0, 120.0),
                delay_ms: 10,
            },
            MacroAction {
                action_type: ActionType::Wait(250),
                delay_ms: 0,
            },
            MacroAction {
                action_type: ActionType::WaitImage("test.png".into(), 1000),
                delay_ms: 100,
            },
        ];

        for (idx, action) in actions.iter().enumerate() {
            let _card = ActionCard::new(idx, action).selected(idx == 0);
        }
    }
}
