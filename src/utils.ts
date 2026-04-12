export const translations: Record<string, Record<string, string>> = {
  fr: {
    "btn_keyboard": "Clavier",
    "btn_mouse": "Souris",
    "btn_wait": "Pause",
    "btn_image": "Image",
    "btn_save": "Sauver",
    "btn_load": "Ouvrir",
    "btn_toolbar": "Toolbar",
    "sect_actions": "Actions de la Macro",
    "lbl_jump": "Aller à :",
    "btn_go": "↵ Go",
    "lbl_loop": "Boucler",
    "lbl_mouse_moves": "Mouvements Souris",
    "btn_stop_image": "Image d'arrêt",
    "lbl_inactive": "Inactif",
    "lbl_active": "Actif",
    "btn_record": "Enregistrer (F8)",
    "btn_stop_rec": "Arrêter (F9)",
    "btn_play": "Jouer la Macro",
    "btn_playing": "Lecture en cours...",
    "desc_no_actions": "Aucune action enregistrée.",
    "desc_press_f8": "Appuyez sur <strong>F8</strong> pour commencer l'enregistrement ou utilisez la Toolbar.",
    "title_logo": "MacroForge v0.1.0 - Puissant outil d'automatisation",
    "title_add_key": "Ajouter Touche Clavier",
    "title_add_mouse": "Ajouter Clic Souris",
    "title_add_wait": "Ajouter Temps d'attente (Délai)",
    "title_add_image": "Attendre l'apparition d'une Image",
    "title_save": "Exporter la macro actuelle dans un fichier (.mforge)",
    "title_load": "Importer une macro depuis un fichier existant",
    "title_toolbar": "Ouvrir la barre d'outils flottante compacte",
    "title_loop": "Relance la lecture de la macro automatiquement à la fin",
    "title_mouse_moves": "Affiche ou masque les déplacements de souris dans la liste pour simplifier la vue",
    "title_stop_image_btn": "Définir une image d'arrêt d'urgence",
    "title_stop_image_status": "Indique si une image d'arrêt d'urgence est active (celle-ci arrêtera la macro si elle est détectée à l'écran)",
    "title_clear_stop_image": "Supprimer l'image d'arrêt",
    "title_record": "Démarre ou arrête l'enregistrement en temps réel (Raccourci: F8)",
    "title_play": "Lance l'exécution de la macro actuelle (Raccourci stop: F4)",
    "title_move_action": "Maintenir pour déplacer l'ordre de cette action",
    "title_delay_before": "Délai avant cette action",
    "title_edit_action": "Modifier les paramètres de cette action",
    "title_del_action": "Supprimer définitivement cette action",
    "lbl_hidden_actions": "+ actions non affichées pour les performances",
    "lbl_visible": "visibles",
    "tb_no_actions": "Aucune action visible.",
    // Modals
    "mod_timeout_title": "Délai d'attente (ms)",
    "mod_timeout_desc": "Veuillez entrer le délai maximum d'attente pour l'image :",
    "mod_cancel": "Annuler",
    "mod_ok": "OK",
    "mod_save": "Sauvegarder",
    "mod_add": "Ajouter",
    "mod_edit_title": "Modifier l'action",
    "mod_val1": "Valeur 1",
    "mod_val2": "Valeur 2",
    "mod_wait": "Attente (ms)",
    "mod_img_config": "Configuration de l'image",
    "mod_img_desc": "Souhaitez-vous utiliser une image de votre PC ou l'image de référence intégrée ?",
    "mod_img_warn": "Si vous utilisez une image locale, votre macro ne fonctionnera pas sur un autre PC sans cette image.",
    "mod_img_embedded": "Image Intégrée (Extreme)",
    "mod_img_embedded_desc": 'Utiliser "extreme.png"',
    "mod_img_failed": "Image Intégrée (Failed)",
    "mod_img_failed_desc": 'Utiliser "failed.PNG"',
    "mod_img_local": "Image Locale",
    "mod_img_local_desc": "Choisir un fichier sur mon PC",
    "mod_add_key": "Ajouter Touche",
    "mod_press_key": "Appuyez sur une touche de votre clavier...",
    "mod_add_click": "Ajouter Clic",
    "mod_coord": "Coordonnées X, Y",
    "mod_add_pause": "Ajouter Pause",
    "mod_duration": "Durée en millisecondes",
    
    // Actions rendering
    "act_nomouse": "Activer 'Mouvements Souris' pour les voir.",
    "act_record": "Enregistrez une macro en appuyant sur F8.",
    "act_btn": "Bouton",
    "act_key": "Touche",
    "act_duration": "Durée",

    // Toolbar
    "tb_play": "Jouer",
    "tb_edit": "Ouvrir l'Editeur",
    "tb_close": "Fermer la Toolbar",
    "msg_save_success": "Macro sauvegardée avec succès!",
    "msg_save_err": "Erreur de sauvegarde: ",
    "msg_load_success": "Macro chargée avec succès!",
    "msg_load_err": "Erreur de chargement: ",
    "msg_success": "Succès",
    "msg_error": "Erreur",
    // Action types
    "Move": "Déplacement",
    "Move Rel": "Déplacement Rel",
    "Click Down": "Clic Pressé",
    "Click Up": "Clic Relâché",
    "Key Down": "Touche Pressée",
    "Key Up": "Touche Relâchée",
    "Scroll": "Défilement",
    "Wait Image": "Attente Image",
    "Pause": "Pause",
    "lbl_path": "Chemin",
    "lbl_timeout": "Délai"
  },
  en: {
    "btn_keyboard": "Keyboard",
    "btn_mouse": "Mouse",
    "btn_wait": "Wait",
    "btn_image": "Image",
    "btn_save": "Save",
    "btn_load": "Open",
    "btn_toolbar": "Toolbar",
    "sect_actions": "Macro Actions",
    "lbl_jump": "Go to:",
    "btn_go": "↵ Go",
    "lbl_loop": "Loop",
    "lbl_mouse_moves": "Mouse Moves",
    "btn_stop_image": "Stop Image",
    "lbl_inactive": "Inactive",
    "lbl_active": "Active",
    "btn_record": "Record (F8)",
    "btn_stop_rec": "Stop (F9)",
    "btn_play": "Play Macro",
    "btn_playing": "Playing...",
    "desc_no_actions": "No recorded actions.",
    "desc_press_f8": "Press <strong>F8</strong> to start recording or use the Toolbar.",
    "title_logo": "MacroForge v0.1.0 - Powerful automation tool",
    "title_add_key": "Add Keyboard Key",
    "title_add_mouse": "Add Mouse Click",
    "title_add_wait": "Add Wait Time (Delay)",
    "title_add_image": "Wait for Image to appear",
    "title_save": "Export current macro to a file (.mforge)",
    "title_load": "Import macro from existing file",
    "title_toolbar": "Open compact floating toolbar",
    "title_loop": "Automatically restart macro loop on finish",
    "title_mouse_moves": "Show or hide mouse movements in the list to simplify view",
    "title_stop_image_btn": "Set an emergency stop image",
    "title_stop_image_status": "Indicates if an emergency stop image is active (it will stop the macro if detected on screen)",
    "title_clear_stop_image": "Remove stop image",
    "title_record": "Start or stop real-time recording (Shortcut: F8)",
    "title_play": "Start macro playback (Shortcut to stop: F4)",
    "title_move_action": "Hold to drag and reorder this action",
    "title_delay_before": "Delay before this action",
    "title_edit_action": "Edit action settings",
    "title_del_action": "Permanently delete this action",
    "lbl_hidden_actions": "+ other actions not shown for performance",
    "lbl_visible": "visible",
    "tb_no_actions": "No visible actions.",
    "mod_timeout_title": "Wait Delay (ms)",
    "mod_timeout_desc": "Please enter maximum wait delay for the image:",
    "mod_cancel": "Cancel",
    "mod_ok": "OK",
    "mod_save": "Save",
    "mod_add": "Add",
    "mod_edit_title": "Edit Action",
    "mod_val1": "Value 1",
    "mod_val2": "Value 2",
    "mod_wait": "Wait (ms)",
    "mod_img_config": "Image Configuration",
    "mod_img_desc": "Do you want to use an image from your PC or the embedded reference image?",
    "mod_img_warn": "If you use a local image, your macro will not work on another PC without this image.",
    "mod_img_embedded": "Embedded Image (Extreme)",
    "mod_img_embedded_desc": 'Use "extreme.png"',
    "mod_img_failed": "Embedded Image (Failed)",
    "mod_img_failed_desc": 'Use "failed.PNG"',
    "mod_img_local": "Local Image",
    "mod_img_local_desc": "Choose a file on my PC",
    "mod_add_key": "Add Key",
    "mod_press_key": "Press a key on your keyboard...",
    "mod_add_click": "Add Click",
    "mod_coord": "Coordinates X, Y",
    "mod_add_pause": "Add Pause",
    "mod_duration": "Duration in milliseconds",
    
    "act_nomouse": "Enable 'Mouse Moves' to see them.",
    "act_record": "Record a macro by pressing F8.",
    "act_btn": "Button",
    "act_key": "Key",
    "act_duration": "Duration",

    "tb_play": "Play",
    "tb_edit": "Open Editor",
    "tb_close": "Close Toolbar",
    "msg_save_success": "Macro saved successfully!",
    "msg_save_err": "Save error: ",
    "msg_load_success": "Macro loaded successfully!",
    "msg_load_err": "Load error: ",
    "msg_success": "Success",
    "msg_error": "Error",
    // Action types
    "Move": "Move",
    "Move Rel": "Move Rel",
    "Click Down": "Click Down",
    "Click Up": "Click Up",
    "Key Down": "Key Down",
    "Key Up": "Key Up",
    "Scroll": "Scroll",
    "Wait Image": "Wait Image",
    "Pause": "Pause",
    "lbl_path": "Path",
    "lbl_timeout": "Timeout"
  }
};

let currentLang = localStorage.getItem("lang") || "fr";

export function setLanguage(lang: string) {
  if (translations[lang]) {
    currentLang = lang;
    localStorage.setItem("lang", lang);
    applyTranslations();
  }
}

export function getLanguage() {
  return currentLang;
}

export function t(key: string): string {
  return translations[currentLang]?.[key] || key;
}

export function applyTranslations() {
  document.querySelectorAll("[data-i18n]").forEach(el => {
    const key = el.getAttribute("data-i18n");
    if (key && translations[currentLang]?.[key]) {
      // If it contains HTML, use innerHTML
      if (translations[currentLang][key].includes("<")) {
          el.innerHTML = translations[currentLang][key];
      } else {
          // If it has children with SVG, don't destroy them, just find text node
          const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT, null);
          let textNode = walker.nextNode();
          
          let replaced = false;
          while(textNode) {
              if (textNode.nodeValue && textNode.nodeValue.trim().length > 0) {
                  textNode.nodeValue = translations[currentLang][key];
                  replaced = true;
                  break;
              }
              textNode = walker.nextNode();
          }
          if (!replaced) {
               // Append text if not found
               el.appendChild(document.createTextNode(" " + translations[currentLang][key]));
          }
      }
    }
  });

  document.querySelectorAll("[data-i18n-dyn]").forEach(el => {
    const key = el.getAttribute("data-i18n-dyn");
    if (key && translations[currentLang]?.[key]) {
      el.textContent = translations[currentLang][key];
    }
  });

  document.querySelectorAll("[data-tooltip-id]").forEach(el => {
    const key = el.getAttribute("data-tooltip-id");
    if (key && translations[currentLang]?.[key]) {
      el.setAttribute("data-tooltip", translations[currentLang][key]);
    }
  });
}

// Tooltip logic
let tooltipEl: HTMLDivElement | null = null;
let tooltipTimeout: number | null = null;

export function initGlobalFeatures() {
    // 1. Right Click Prevention in Production
    // @ts-ignore
    if (!(import.meta as any).env?.DEV) {
        document.addEventListener('contextmenu', e => e.preventDefault());
    }

    // 2. Custom Tooltip
    tooltipEl = document.createElement("div");
    tooltipEl.id = "custom-tooltip";
    document.body.appendChild(tooltipEl);

    document.addEventListener("mouseover", (e) => {
        const target = (e.target as HTMLElement).closest("[data-tooltip]");
        if (target) {
            const text = target.getAttribute("data-tooltip");
            if (text) {
                if (tooltipTimeout) clearTimeout(tooltipTimeout);
                tooltipTimeout = window.setTimeout(() => {
                    if (tooltipEl) {
                        tooltipEl.textContent = text;
                        tooltipEl.classList.add("show");
                    }
                }, 300); // 300ms delay like native tooltips
            }
        }
    });

    document.addEventListener("mouseout", (e) => {
        const target = (e.target as HTMLElement).closest("[data-tooltip]");
        if (target) {
            if (tooltipTimeout) clearTimeout(tooltipTimeout);
            if (tooltipEl) tooltipEl.classList.remove("show");
        }
    });

    document.addEventListener("mousemove", (e) => {
        if (tooltipEl && tooltipEl.classList.contains("show")) {
            // Position carefully so it doesn't overlap mouse or go out of bounds
            let x = e.clientX + 15;
            let y = e.clientY + 15;
            
            const rect = tooltipEl.getBoundingClientRect();
            const vw = window.innerWidth;
            const vh = window.innerHeight;

            if (x + rect.width > vw - 10) x = vw - rect.width - 10;
            if (y + rect.height > vh - 10) y = Math.max(10, e.clientY - rect.height - 15);
            tooltipEl.style.left = `${x}px`;
            tooltipEl.style.top = `${y}px`;
        }
    });

    // Initial translation application
    applyTranslations();
}
