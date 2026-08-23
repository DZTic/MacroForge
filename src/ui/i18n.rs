//! Module d'internationalisation (i18n) pour MacroForge
//! Supporte le Français (FR) et l'Anglais (EN) avec typage strict et lookup universel.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Language {
    #[default]
    Fr,
    En,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub language: Language,
    pub loop_playback: bool,
    pub hide_mouse_moves: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: Language::Fr,
            loop_playback: false,
            hide_mouse_moves: false,
        }
    }
}

impl AppSettings {
    pub fn load() -> Self {
        if let Ok(content) = std::fs::read_to_string("macroforge_settings.json") {
            if let Ok(settings) = serde_json::from_str(&content) {
                return settings;
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write("macroforge_settings.json", json);
        }
    }
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

    /// Lookup universel par clé pour la compatibilité avec le dictionnaire web_legacy/utils.ts
    pub fn t(&self, key: &str) -> &'static str {
        match self {
            Language::Fr => match key {
                "btn_keyboard" => "Clavier",
                "btn_mouse" => "Souris",
                "btn_wait" => "Pause",
                "btn_image" => "Image",
                "btn_save" => "Sauvegarder",
                "btn_load" => "Ouvrir",
                "btn_toolbar" => "Toolbar",
                "sect_actions" => "Actions de la Macro",
                "lbl_jump" => "Aller à :",
                "btn_go" => "↵ Go",
                "lbl_loop" => "Boucler",
                "lbl_mouse_moves" => "Mouvements Souris",
                "btn_stop_image" => "Image d'arrêt",
                "lbl_inactive" => "Inactif",
                "lbl_active" => "Actif",
                "btn_record" => "Enregistrer (F8)",
                "btn_stop_rec" => "Arrêter (F9)",
                "btn_play" => "Jouer la Macro",
                "btn_playing" => "Lecture en cours...",
                "desc_no_actions" => "Aucune action enregistrée.",
                "desc_press_f8" => "Appuyez sur F8 pour commencer l'enregistrement ou utilisez la Toolbar.",
                "title_logo" => "MacroForge v0.2.0 - Puissant outil d'automatisation",
                "title_add_key" => "Ajouter Touche Clavier",
                "title_add_mouse" => "Ajouter Clic Souris",
                "title_add_wait" => "Ajouter Temps d'attente (Délai)",
                "title_add_image" => "Attendre l'apparition d'une Image",
                "title_save" => "Exporter la macro actuelle dans un fichier (.mforge)",
                "title_load" => "Importer une macro depuis un fichier existant",
                "title_toolbar" => "Ouvrir la barre d'outils flottante compacte",
                "title_loop" => "Relance la lecture de la macro automatiquement à la fin",
                "title_mouse_moves" => "Affiche ou masque les déplacements de souris dans la liste pour simplifier la vue",
                "title_stop_image_btn" => "Définir une image d'arrêt d'urgence",
                "title_stop_image_status" => "Indique si une image d'arrêt d'urgence est active (celle-ci arrêtera la macro si elle est détectée à l'écran)",
                "title_clear_stop_image" => "Supprimer l'image d'arrêt",
                "title_record" => "Démarre ou arrête l'enregistrement en temps réel (Raccourci: F8)",
                "title_play" => "Lance l'exécution de la macro actuelle (Raccourci stop: F4)",
                "title_move_action" => "Maintenir pour déplacer l'ordre de cette action",
                "title_delay_before" => "Délai avant cette action",
                "title_edit_action" => "Modifier les paramètres de cette action",
                "title_del_action" => "Supprimer définitivement cette action",
                "lbl_hidden_actions" => "+ actions non affichées pour les performances",
                "lbl_visible" => "visibles",
                "tb_no_actions" => "Aucune action visible.",
                "mod_timeout_title" => "Délai d'attente (ms)",
                "mod_timeout_desc" => "Veuillez entrer le délai maximum d'attente pour l'image :",
                "mod_cancel" => "Annuler",
                "mod_ok" => "OK",
                "mod_save" => "Sauvegarder",
                "mod_add" => "Ajouter",
                "mod_edit_title" => "Modifier l'action",
                "mod_val1" => "Valeur 1",
                "mod_val2" => "Valeur 2",
                "mod_wait" => "Attente (ms)",
                "mod_img_config" => "Configuration de l'image",
                "mod_img_desc" => "Souhaitez-vous utiliser une image de votre PC ou l'image de référence intégrée ?",
                "mod_img_warn" => "Si vous utilisez une image locale, votre macro ne fonctionnera pas sur un autre PC sans cette image.",
                "mod_img_embedded" => "Image Intégrée (Extreme)",
                "mod_img_embedded_desc" => "Utiliser extreme.png",
                "mod_img_failed" => "Image Intégrée (Failed)",
                "mod_img_failed_desc" => "Utiliser failed.PNG",
                "mod_img_local" => "Image Locale",
                "mod_img_local_desc" => "Choisir un fichier sur mon PC",
                "mod_add_key" => "Ajouter Touche",
                "mod_press_key" => "Appuyez sur une touche de votre clavier...",
                "mod_add_click" => "Ajouter Clic",
                "mod_coord" => "Coordonnées X, Y",
                "mod_add_pause" => "Ajouter Pause",
                "mod_duration" => "Durée en millisecondes",
                "act_nomouse" => "Activer Mouvements Souris pour les voir.",
                "act_record" => "Enregistrez une macro en appuyant sur F8.",
                "act_btn" => "Bouton",
                "act_key" => "Touche",
                "act_duration" => "Durée",
                "tb_play" => "Jouer",
                "tb_edit" => "Ouvrir l'Editeur",
                "tb_close" => "Fermer la Toolbar",
                "msg_save_success" => "Macro sauvegardée avec succès!",
                "msg_save_err" => "Erreur de sauvegarde: ",
                "msg_load_success" => "Macro chargée avec succès!",
                "msg_load_err" => "Erreur de chargement: ",
                "msg_success" => "Succès",
                "msg_error" => "Erreur",
                "Move" => "Déplacement",
                "Move Rel" => "Déplacement Relatif",
                "Click Down" => "Clic Pressé",
                "Click Up" => "Clic Relâché",
                "Key Down" => "Touche Pressée",
                "Key Up" => "Touche Relâchée",
                "Scroll" => "Défilement",
                "Wait Image" => "Attente Image",
                "Pause" => "Pause",
                "lbl_path" => "Chemin",
                "lbl_timeout" => "Délai",
                "lbl_action" => "Action",
                "btn_settings" => "Paramètres",
                "mod_settings_title" => "Paramètres",
                "lbl_show_progress" => "Afficher le défilement des actions",
                "mod_close" => "Fermer",
                "title_settings" => "Ouvrir les paramètres de l'application",
                _ => "Texte non trouvé",
            },
            Language::En => match key {
                "btn_keyboard" => "Keyboard",
                "btn_mouse" => "Mouse",
                "btn_wait" => "Wait",
                "btn_image" => "Image",
                "btn_save" => "Save",
                "btn_load" => "Open",
                "btn_toolbar" => "Toolbar",
                "sect_actions" => "Macro Actions",
                "lbl_jump" => "Go to:",
                "btn_go" => "↵ Go",
                "lbl_loop" => "Loop",
                "lbl_mouse_moves" => "Mouse Moves",
                "btn_stop_image" => "Stop Image",
                "lbl_inactive" => "Inactive",
                "lbl_active" => "Active",
                "btn_record" => "Record (F8)",
                "btn_stop_rec" => "Stop (F9)",
                "btn_play" => "Play Macro",
                "btn_playing" => "Playing...",
                "desc_no_actions" => "No recorded actions.",
                "desc_press_f8" => "Press F8 to start recording or use the Toolbar.",
                "title_logo" => "MacroForge v0.2.0 - Powerful automation tool",
                "title_add_key" => "Add Keyboard Key",
                "title_add_mouse" => "Add Mouse Click",
                "title_add_wait" => "Add Wait Time (Delay)",
                "title_add_image" => "Wait for Image to appear",
                "title_save" => "Export current macro to a file (.mforge)",
                "title_load" => "Import macro from existing file",
                "title_toolbar" => "Open compact floating toolbar",
                "title_loop" => "Automatically restart macro loop on finish",
                "title_mouse_moves" => "Show or hide mouse movements in the list to simplify view",
                "title_stop_image_btn" => "Set an emergency stop image",
                "title_stop_image_status" => "Indicates if an emergency stop image is active (it will stop the macro if detected on screen)",
                "title_clear_stop_image" => "Remove stop image",
                "title_record" => "Start or stop real-time recording (Shortcut: F8)",
                "title_play" => "Start macro playback (Shortcut to stop: F4)",
                "title_move_action" => "Hold to drag and reorder this action",
                "title_delay_before" => "Delay before this action",
                "title_edit_action" => "Edit action settings",
                "title_del_action" => "Permanently delete this action",
                "lbl_hidden_actions" => "+ other actions not shown for performance",
                "lbl_visible" => "visible",
                "tb_no_actions" => "No visible actions.",
                "mod_timeout_title" => "Wait Delay (ms)",
                "mod_timeout_desc" => "Please enter maximum wait delay for the image:",
                "mod_cancel" => "Cancel",
                "mod_ok" => "OK",
                "mod_save" => "Save",
                "mod_add" => "Add",
                "mod_edit_title" => "Edit Action",
                "mod_val1" => "Value 1",
                "mod_val2" => "Value 2",
                "mod_wait" => "Wait (ms)",
                "mod_img_config" => "Image Configuration",
                "mod_img_desc" => "Do you want to use an image from your PC or the embedded reference image?",
                "mod_img_warn" => "If you use a local image, your macro will not work on another PC without this image.",
                "mod_img_embedded" => "Embedded Image (Extreme)",
                "mod_img_embedded_desc" => "Use extreme.png",
                "mod_img_failed" => "Embedded Image (Failed)",
                "mod_img_failed_desc" => "Use failed.PNG",
                "mod_img_local" => "Local Image",
                "mod_img_local_desc" => "Choose a file on my PC",
                "mod_add_key" => "Add Key",
                "mod_press_key" => "Press a key on your keyboard...",
                "mod_add_click" => "Add Click",
                "mod_coord" => "Coordinates X, Y",
                "mod_add_pause" => "Add Pause",
                "mod_duration" => "Duration in milliseconds",
                "act_nomouse" => "Enable Mouse Moves to see them.",
                "act_record" => "Record a macro by pressing F8.",
                "act_btn" => "Button",
                "act_key" => "Key",
                "act_duration" => "Duration",
                "tb_play" => "Play",
                "tb_edit" => "Open Editor",
                "tb_close" => "Close Toolbar",
                "msg_save_success" => "Macro saved successfully!",
                "msg_save_err" => "Save error: ",
                "msg_load_success" => "Macro loaded successfully!",
                "msg_load_err" => "Load error: ",
                "msg_success" => "Success",
                "msg_error" => "Error",
                "Move" => "Move",
                "Move Rel" => "Relative Move",
                "Click Down" => "Mouse Down",
                "Click Up" => "Mouse Up",
                "Key Down" => "Key Down",
                "Key Up" => "Key Up",
                "Scroll" => "Scroll",
                "Wait Image" => "Wait Image",
                "Pause" => "Pause",
                "lbl_path" => "Path",
                "lbl_timeout" => "Timeout",
                "lbl_action" => "Action",
                "btn_settings" => "Settings",
                "mod_settings_title" => "Settings",
                "lbl_show_progress" => "Show action scrolling",
                "mod_close" => "Close",
                "title_settings" => "Open application settings",
                _ => "Text not found",
            },
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
            Language::Fr => "+ Clavier",
            Language::En => "+ Keyboard",
        }
    }

    pub fn quick_add_mouse(&self) -> &'static str {
        match self {
            Language::Fr => "+ Souris",
            Language::En => "+ Mouse",
        }
    }

    pub fn quick_add_wait(&self) -> &'static str {
        match self {
            Language::Fr => "+ Pause",
            Language::En => "+ Wait",
        }
    }

    pub fn quick_add_image(&self) -> &'static str {
        match self {
            Language::Fr => "+ Image",
            Language::En => "+ Image",
        }
    }

    pub fn toolbar_window_btn(&self) -> &'static str {
        match self {
            Language::Fr => "Toolbar",
            Language::En => "Toolbar",
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
            Language::Fr => "Séquence d'Actions",
            Language::En => "Action Sequence",
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
            Language::Fr => "Rechercher une action...",
            Language::En => "Search an action...",
        }
    }

    pub fn jump_to_action_label(&self) -> &'static str {
        match self {
            Language::Fr => "Aller à n°",
            Language::En => "Go to #",
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
            Language::Fr => "Mode Boucle",
            Language::En => "Loop Mode",
        }
    }

    pub fn stop_image_cfg_btn(&self) -> &'static str {
        match self {
            Language::Fr => "Image d'arrêt",
            Language::En => "Stop Image",
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
            Language::Fr => "Ajouter une Action",
            Language::En => "Add Action",
        }
    }

    pub fn modal_edit_action_title(&self) -> &'static str {
        match self {
            Language::Fr => "Modifier l'Action",
            Language::En => "Edit Action",
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
            Language::Fr => "Clavier",
            Language::En => "Keyboard",
        }
    }

    pub fn tab_mouse(&self) -> &'static str {
        match self {
            Language::Fr => "Souris",
            Language::En => "Mouse",
        }
    }

    pub fn tab_wait(&self) -> &'static str {
        match self {
            Language::Fr => "Pause",
            Language::En => "Wait",
        }
    }

    pub fn tab_image(&self) -> &'static str {
        match self {
            Language::Fr => "Image",
            Language::En => "Image",
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

    #[test]
    fn test_universal_lookup_dictionary() {
        let fr = Language::Fr;
        let en = Language::En;

        assert_eq!(fr.t("btn_keyboard"), "Clavier");
        assert_eq!(en.t("btn_keyboard"), "Keyboard");

        assert_eq!(fr.t("btn_mouse"), "Souris");
        assert_eq!(en.t("btn_mouse"), "Mouse");

        assert_eq!(fr.t("btn_save"), "Sauvegarder");
        assert_eq!(en.t("btn_save"), "Save");

        assert_eq!(fr.t("msg_save_success"), "Macro sauvegardée avec succès!");
        assert_eq!(en.t("msg_save_success"), "Macro saved successfully!");
    }

    #[test]
    fn test_settings_persistence() {
        let settings = AppSettings {
            language: Language::En,
            loop_playback: true,
            hide_mouse_moves: true,
        };

        let json = serde_json::to_string(&settings).unwrap();
        let loaded: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.language, Language::En);
        assert!(loaded.loop_playback);
        assert!(loaded.hide_mouse_moves);
    }
}
