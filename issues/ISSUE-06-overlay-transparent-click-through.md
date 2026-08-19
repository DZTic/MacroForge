# 📌 Issue #06 : Overlay Transparent Click-Through (HUD sans capture GDI)

- **Statut** : 📝 À faire
- **Priorité** : 🟠 Haute
- **Composants** : Overlay Window, Win32 Extended Styles, GDI Exclusion, HUD Display
- **Agents Référents** : `.agents/agents/rust-core.md`, `.agents/agents/vision-automation.md`
- **Dépendances** : Issue #01, Issue #02

---

## 🎯 Description du Besoin
Pendant la relecture d'une macro, MacroForge affiche un **Overlay transparent** par-dessus tout l'écran pour indiquer à l'utilisateur l'action en cours d'exécution (nom de la touche, position de la souris, étape dans la macro).
Cet overlay doit être **100% click-through** (ne jamais bloquer un clic de jeu ou d'application) et **invisible pour les captures d'écran GDI** de l'algorithme de détection d'image.

---

## 📋 Tâches Techniques

1. **Configuration de la Fenêtre Overlay Native sous Windows (Win32)** :
   - Créer une fenêtre plein écran sans bordure, transparente et toujours au premier plan :
     ```rust
     #[cfg(windows)]
     unsafe {
         use winapi::um::winuser::*;
         
         // Styles étendus essentiels :
         // WS_EX_LAYERED : Permet la transparence alpha
         // WS_EX_TRANSPARENT : Fait traverser tous les clics souris (click-through absolu)
         // WS_EX_TOOLWINDOW : Masque la fenêtre de la barre des tâches et de Alt+Tab
         // WS_EX_TOPMOST : Maintient au-dessus des jeux en mode fenêtré / borderless
         let ex_style = WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_TOPMOST;
         SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style as isize);
     }
     ```

2. **Exclusion des Captures d'Écran GDI (Anti-Auto-Détection)** :
   - Appliquer l'affinité d'affichage pour que la capture d'écran GDI (`BitBlt` / `GetDIBits`) ne voie jamais l'overlay :
     ```rust
     #[cfg(windows)]
     unsafe {
         use winapi::um::winuser::SetWindowDisplayAffinity;
         const WDA_EXCLUDEFROMCAPTURE: u32 = 0x00000011;
         SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE);
     }
     ```

3. **Rendu du HUD Temps Réel** :
   - Badge compact en haut ou au centre de l'écran affichant :
     - Type d'action (ex: `⌨️ KeyPress 'Space'`, `🖱️ Click Button 1 (X: 520, Y: 840)`, `⏱️ Pause 250ms`, `🖼️ WaitImage`).
     - Indicateur d'avancement (`Action #12 / 85`).
     - Réticule / curseur visuel discret à la position cible si configuré.

4. **Activation / Désactivation Automatique** :
   - L'overlay s'affiche automatiquement dès le démarrage de la lecture de la macro (`is_playing = true`).
   - L'overlay est masqué immédiatement dès la fin ou lors de l'arrêt d'urgence F4.

---

## ✅ Critères d'Acceptation
- [ ] L'overlay n'intercepte aucun clic ni événement de souris ou clavier (click-through 100% fonctionnel dans les jeux).
- [ ] L'overlay ne perturbe pas la détection d'image `WaitImage` ni `StopImage` grâce à `WDA_EXCLUDEFROMCAPTURE`.
- [ ] Le HUD s'affiche et se masque instantanément sans freeze ni clignotement.
