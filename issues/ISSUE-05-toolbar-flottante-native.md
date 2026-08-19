# 📌 Issue #05 : Toolbar Flottante Native (Mini-contrôleur compact)

- **Statut** : 📝 À faire
- **Priorité** : 🟠 Haute
- **Composants** : Toolbar Window, Multi-viewport, Win32 Window Styles, Drag Handle
- **Agents Référents** : `.agents/agents/frontend-ui.md`, `.agents/agents/rust-core.md`
- **Dépendances** : Issue #01, Issue #02

---

## 🎯 Description du Besoin
MacroForge propose un mode **Toolbar flottante**, une fenêtre ultra-compacte sans bordure, transparente et toujours au premier plan, permettant de contrôler la macro (Record, Play, Stop, Edit, Close) pendant que l'utilisateur est dans son jeu ou application sans encombrer l'écran.

---

## 📋 Tâches Techniques

1. **Création du Viewport Toolbar Dédié** :
   - Configurer un `egui::ViewportId` spécifique pour la toolbar :
     ```rust
     let toolbar_viewport = egui::ViewportBuilder::default()
         .with_title("MacroForge Toolbar")
         .with_inner_size([280.0, 52.0])
         .with_decorations(false)
         .with_transparent(true)
         .with_always_on_top(true)
         .with_resizable(false);
     ```

2. **Poignée de Déplacement Native (Drag Handle)** :
   - Permettre de déplacer la toolbar à n'importe quel endroit de l'écran en cliquant-glissant sur la poignée de déplacement (`ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag)`).

3. **Boutons & Contrôles du Mini-Contrôleur** :
   - **Bouton Record** : Cercle rouge vibrant / Carré blanc lors de l'enregistrement.
   - **Bouton Play** : Icône triangle verte / état désactivé semi-transparent pendant la lecture.
   - **Bouton Stop (F4/F9)** : Carré rouge pour interruption immédiate.
   - **Bouton Éditeur (Edit)** : Réaffiche et remet au premier plan la fenêtre principale (`MainWindow`).
   - **Bouton Fermer (Close)** : Masque la toolbar.
   - **Indicateur de Progression Animé** : Affiche en temps réel le numéro de l'action en cours d'exécution (`Action X / Total`).

4. **Synchronisation du Cycle de Vie Multi-fenêtres** :
   - Si l'utilisateur ferme la fenêtre principale mais que la toolbar est visible : l'application reste active.
   - Si l'utilisateur ferme la toolbar alors que la fenêtre principale est masquée : l'application quitte proprement (`std::process::exit(0)`).
   - Les boutons de la toolbar mettent à jour instantanément l'état de l'application via les événements partagés.

---

## ✅ Critères d'Acceptation
- [ ] La toolbar apparaît sans bordure Windows classique, avec un fond sombre semi-transparent et des coins arrondis.
- [ ] Le déplacement de la toolbar à la souris est instantané et sans saccade.
- [ ] L'affichage du compteur d'actions s'anime fidèlement pendant la relecture.
- [ ] L'ouverture de l'éditeur depuis la toolbar réactive la fenêtre principale instantanément.
