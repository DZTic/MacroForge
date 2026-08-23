//! Module d'internationalisation (i18n) pour MacroForge
//! Supporte le Français (FR) et l'Anglais (EN).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    Fr,
    En,
}

impl Language {
    pub fn toggle(&mut self) {
        *self = match self {
            Language::Fr => Language::En,
            Language::En => Language::Fr,
        };
    }

    pub fn code(&self) -> &'static str {
        match self {
            Language::Fr => "FR",
            Language::En => "EN",
        }
    }

    // --- Header & General ---
    pub fn app_title(&self) -> &'static str {
        match self {
            Language::Fr => "⚡ MacroForge",
            Language::En => "⚡ MacroForge",
        }
    }

    pub fn quick_add_key(&self) -> &'static str {
        match self {
            Language::Fr => "+ ⌨️ Clavier",
            Language::En => "+ ⌨️ Keyboard",
        }
    }

    pub fn quick_add_mouse(&self) -> &'static str {
        match self {
            Language::Fr => "+ 🖱️ Souris",
            Language::En => "+ 🖱️ Mouse",
        }
    }

    pub fn quick_add_wait(&self) -> &'static str {
        match self {
            Language::Fr => "+ ⏱️ Pause",
            Language::En => "+ ⏱️ Wait",
        }
    }

    pub fn quick_add_image(&self) -> &'static str {
        match self {
            Language::Fr => "+ 🖼️ Image",
            Language::En => "+ 🖼️ Image",
        }
    }

    pub fn toolbar_window_btn(&self) -> &'static str {
        match self {
            Language::Fr => "🗔 Toolbar",
            Language::En => "🗔 Toolbar",
        }
    }

    pub fn save_profile(&self) -> &'static str {
        match self {
            Language::Fr => "Sauvegarder",
            Language::En => "Save",
        }
    }

    pub fn open_profile(&self) -> &'static str {
        match self {
            Language::Fr => "Ouvrir",
            Language::En => "Open",
        }
    }

    pub fn clear_actions(&self) -> &'static str {
        match self {
            Language::Fr => "Vider",
            Language::En => "Clear",
        }
    }

    pub fn refresh_btn(&self) -> &'static str {
        match self {
            Language::Fr => "Rafraîchir",
            Language::En => "Refresh",
        }
    }

    // --- Timeline & Filters ---
    pub fn timeline_heading(&self) -> &'static str {
        match self {
            Language::Fr => "📋 Séquence d'Actions",
            Language::En => "📋 Action Sequence",
        }
    }

    pub fn filter_hide_mouse_moves(&self) -> &'static str {
        match self {
            Language::Fr => "Masquer déplacements souris",
            Language::En => "Hide mouse movements",
        }
    }

    pub fn filter_search_placeholder(&self) -> &'static str {
        match self {
            Language::Fr => "🔍 Filtrer les actions...",
            Language::En => "🔍 Filter actions...",
        }
    }

    pub fn jump_to_action_label(&self) -> &'static str {
        match self {
            Language::Fr => "Aller à n°",
            Language::En => "Jump to #",
        }
    }

    pub fn jump_btn(&self) -> &'static str {
        match self {
            Language::Fr => "Aller",
            Language::En => "Go",
        }
    }

    pub fn action_count_badge(&self, visible: usize, total: usize) -> String {
        match self {
            Language::Fr => format!("{} visible(s) / {} total", visible, total),
            Language::En => format!("{} visible / {} total", visible, total),
        }
    }

    pub fn empty_state_title(&self) -> &'static str {
        match self {
            Language::Fr => "Aucune action dans la macro",
            Language::En => "No actions in macro",
        }
    }

    pub fn empty_state_desc(&self) -> &'static str {
        match self {
            Language::Fr => "Appuyez sur la touche F8 pour enregistrer ou utilisez les boutons d'ajout ci-dessus pour insérer des actions manuellement.",
            Language::En => "Press F8 to record or use the quick add buttons above to manually insert actions.",
        }
    }

    // --- Action Card ---
    pub fn action_key_press(&self) -> &'static str {
        match self {
            Language::Fr => "Touche Pressée",
            Language::En => "Key Press",
        }
    }

    pub fn action_key_release(&self) -> &'static str {
        match self {
            Language::Fr => "Touche Relâchée",
            Language::En => "Key Release",
        }
    }

    pub fn action_mouse_pos(&self) -> &'static str {
        match self {
            Language::Fr => "Position Souris",
            Language::En => "Mouse Position",
        }
    }

    pub fn action_mouse_relative(&self) -> &'static str {
        match self {
            Language::Fr => "Mouvement Relatif",
            Language::En => "Relative Move",
        }
    }

    pub fn action_mouse_press(&self) -> &'static str {
        match self {
            Language::Fr => "Clic Pressé",
            Language::En => "Mouse Down",
        }
    }

    pub fn action_mouse_release(&self) -> &'static str {
        match self {
            Language::Fr => "Clic Relâché",
            Language::En => "Mouse Up",
        }
    }

    pub fn action_scroll(&self) -> &'static str {
        match self {
            Language::Fr => "Molette Défilement",
            Language::En => "Mouse Scroll",
        }
    }

    pub fn action_wait(&self) -> &'static str {
        match self {
            Language::Fr => "Pause",
            Language::En => "Wait",
        }
    }

    pub fn action_wait_image(&self) -> &'static str {
        match self {
            Language::Fr => "Détection Image",
            Language::En => "Image Detection",
        }
    }

    pub fn edit_tooltip(&self) -> &'static str {
        match self {
            Language::Fr => "Modifier cette action",
            Language::En => "Edit this action",
        }
    }

    pub fn duplicate_tooltip(&self) -> &'static str {
        match self {
            Language::Fr => "Dupliquer cette action",
            Language::En => "Duplicate this action",
        }
    }

    pub fn delete_tooltip(&self) -> &'static str {
        match self {
            Language::Fr => "Supprimer cette action",
            Language::En => "Delete this action",
        }
    }

    pub fn move_up_tooltip(&self) -> &'static str {
        match self {
            Language::Fr => "Déplacer vers le haut",
            Language::En => "Move up",
        }
    }

    pub fn move_down_tooltip(&self) -> &'static str {
        match self {
            Language::Fr => "Déplacer vers le bas",
            Language::En => "Move down",
        }
    }

    // --- Footer & Global Controls ---
    pub fn record_btn(&self) -> &'static str {
        match self {
            Language::Fr => "Enregistrer",
            Language::En => "Record",
        }
    }

    pub fn stop_btn(&self) -> &'static str {
        match self {
            Language::Fr => "Arrêter",
            Language::En => "Stop",
        }
    }

    pub fn play_btn(&self) -> &'static str {
        match self {
            Language::Fr => "Rejouer",
            Language::En => "Play",
        }
    }

    pub fn emergency_stop_btn(&self) -> &'static str {
        match self {
            Language::Fr => "Arrêt Urgence",
            Language::En => "Emergency Stop",
        }
    }

    pub fn loop_mode_label(&self) -> &'static str {
        match self {
            Language::Fr => "🔁 Mode Boucle",
            Language::En => "🔁 Loop Mode",
        }
    }

    pub fn stop_image_cfg_btn(&self) -> &'static str {
        match self {
            Language::Fr => "🛑 Image d'arrêt",
            Language::En => "🛑 Stop Image",
        }
    }

    pub fn ready_status(&self) -> &'static str {
        match self {
            Language::Fr => {
                "Prêt. Appuyez sur F8 pour démarrer l'enregistrement ou F4 pour rejouer."
            }
            Language::En => "Ready. Press F8 to start recording or F4 to replay.",
        }
    }

    // --- Modals ---
    pub fn modal_add_action_title(&self) -> &'static str {
        match self {
            Language::Fr => "➕ Ajouter une Action Manuelle",
            Language::En => "➕ Add Manual Action",
        }
    }

    pub fn modal_edit_action_title(&self) -> &'static str {
        match self {
            Language::Fr => "✏️ Modifier l'Action",
            Language::En => "✏️ Edit Action",
        }
    }

    pub fn modal_save(&self) -> &'static str {
        match self {
            Language::Fr => "Valider",
            Language::En => "Apply",
        }
    }

    pub fn modal_cancel(&self) -> &'static str {
        match self {
            Language::Fr => "Annuler",
            Language::En => "Cancel",
        }
    }

    pub fn tab_keyboard(&self) -> &'static str {
        match self {
            Language::Fr => "⌨️ Clavier",
            Language::En => "⌨️ Keyboard",
        }
    }

    pub fn tab_mouse(&self) -> &'static str {
        match self {
            Language::Fr => "🖱️ Souris",
            Language::En => "🖱️ Mouse",
        }
    }

    pub fn tab_wait(&self) -> &'static str {
        match self {
            Language::Fr => "⏱️ Pause",
            Language::En => "⏱️ Wait",
        }
    }

    pub fn tab_image(&self) -> &'static str {
        match self {
            Language::Fr => "🖼️ Détection Image",
            Language::En => "🖼️ Image Detection",
        }
    }

    pub fn delay_label(&self) -> &'static str {
        match self {
            Language::Fr => "Délai avant exécution :",
            Language::En => "Delay before execution:",
        }
    }

    pub fn key_label(&self) -> &'static str {
        match self {
            Language::Fr => "Nom de la touche :",
            Language::En => "Key name:",
        }
    }

    pub fn vk_code_label(&self) -> &'static str {
        match self {
            Language::Fr => "Code Virtuel (VK) :",
            Language::En => "Virtual Key Code (VK):",
        }
    }

    pub fn mouse_action_type(&self) -> &'static str {
        match self {
            Language::Fr => "Type d'événement souris :",
            Language::En => "Mouse event type:",
        }
    }

    pub fn mouse_btn_label(&self) -> &'static str {
        match self {
            Language::Fr => "Bouton de souris :",
            Language::En => "Mouse button:",
        }
    }

    pub fn mouse_btn_left(&self) -> &'static str {
        match self {
            Language::Fr => "Gauche (1)",
            Language::En => "Left (1)",
        }
    }

    pub fn mouse_btn_right(&self) -> &'static str {
        match self {
            Language::Fr => "Droit (2)",
            Language::En => "Right (2)",
        }
    }

    pub fn mouse_btn_middle(&self) -> &'static str {
        match self {
            Language::Fr => "Milieu (3)",
            Language::En => "Middle (3)",
        }
    }

    pub fn stop_image_modal_title(&self) -> &'static str {
        match self {
            Language::Fr => "🛑 Configuration de l'Image d'Arrêt d'Urgence",
            Language::En => "🛑 Emergency Stop Image Configuration",
        }
    }

    pub fn stop_image_enable(&self) -> &'static str {
        match self {
            Language::Fr => "Activer la détection d'arrêt d'urgence par image",
            Language::En => "Enable emergency stop by image pattern",
        }
    }

    pub fn stop_image_path_label(&self) -> &'static str {
        match self {
            Language::Fr => "Chemin de l'image modèle :",
            Language::En => "Template image path:",
        }
    }

    pub fn browse_file_btn(&self) -> &'static str {
        match self {
            Language::Fr => "Parcourir...",
            Language::En => "Browse...",
        }
    }

    pub fn event_type_label(&self) -> &'static str {
        match self {
            Language::Fr => "Événement :",
            Language::En => "Event type:",
        }
    }

    pub fn capture_key_btn(&self) -> &'static str {
        match self {
            Language::Fr => "Capturer touche",
            Language::En => "Capture key",
        }
    }

    pub fn key_listening_prompt(&self) -> &'static str {
        match self {
            Language::Fr => "Appuyez sur une touche... (Échap pour annuler)",
            Language::En => "Press any key... (Esc to cancel)",
        }
    }

    pub fn extended_key_label(&self) -> &'static str {
        match self {
            Language::Fr => "Touche étendue (Extended Key)",
            Language::En => "Extended Key",
        }
    }

    pub fn capture_cursor_pos_btn(&self) -> &'static str {
        match self {
            Language::Fr => "Capturer position actuelle",
            Language::En => "Capture current position",
        }
    }

    pub fn wait_duration_label(&self) -> &'static str {
        match self {
            Language::Fr => "Durée de la pause :",
            Language::En => "Wait duration:",
        }
    }

    pub fn presets_label(&self) -> &'static str {
        match self {
            Language::Fr => "Préréglages :",
            Language::En => "Presets:",
        }
    }

    pub fn embedded_images_label(&self) -> &'static str {
        match self {
            Language::Fr => "Images intégrées :",
            Language::En => "Embedded templates:",
        }
    }

    pub fn timeout_label(&self) -> &'static str {
        match self {
            Language::Fr => "Délai max de détection (timeout ms) :",
            Language::En => "Max detection timeout (ms):",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_toggle() {
        let mut lang = Language::Fr;
        assert_eq!(lang.code(), "FR");
        lang.toggle();
        assert_eq!(lang.code(), "EN");
        lang.toggle();
        assert_eq!(lang.code(), "FR");
    }

    #[test]
    fn test_translations_presence() {
        let fr = Language::Fr;
        let en = Language::En;

        assert!(!fr.app_title().is_empty());
        assert!(!en.app_title().is_empty());
        assert_ne!(fr.filter_hide_mouse_moves(), en.filter_hide_mouse_moves());
        assert_ne!(fr.quick_add_key(), en.quick_add_key());
        assert_ne!(fr.capture_key_btn(), en.capture_key_btn());
        assert_ne!(fr.capture_cursor_pos_btn(), en.capture_cursor_pos_btn());
    }
}
