# 📌 Issue #04 : Modales & Boîtes de Dialogue Natives (Clavier, Souris, Images, RFD)

- **Statut** : 📝 À faire
- **Priorité** : 🟠 Haute
- **Composants** : Dialogs UI, Native File Picker (`rfd`), Input Modals
- **Agents Référents** : `.agents/agents/frontend-ui.md`, `.agents/agents/rust-core.md`
- **Dépendances** : Issue #01, Issue #02, Issue #03

---

## 🎯 Description du Besoin
L'ajout et la modification d'actions dans MacroForge nécessitent des boîtes de dialogue interactives pour capturer des touches, ajuster des coordonnées ou sélectionner des images sur le disque.
De plus, la sauvegarde et l'ouverture de profils `.mforge` doivent utiliser les fenêtres de fichiers natives de Windows.

---

## 📋 Tâches Techniques

1. **Modale d'Ajout / Édition de Touche Clavier** :
   - Interface de capture interactive : dès que l'utilisateur appuie sur une touche, capture de son Virtual Key (`VK`), de son état étendu (`is_extended`) et affichage de son libellé convivial (ex: `KeyA`, `Enter`, `F5`, `ShiftLeft`).
   - Sélection du type d'événement : `KeyPress` (Pressée), `KeyRelease` (Relâchée), ou combinaison complète.
   - Saisie du délai avant exécution (`delay_ms`).

2. **Modale d'Ajout / Édition de Clic Souris** :
   - Sélection du bouton : Gauche (1), Droit (2), Molette/Milieu (3), Autre (4).
   - Sélection du type : `MousePress`, `MouseRelease`, `MouseMove`.
   - Saisie des coordonnées cibles `(X, Y)` ou deltas relatifs `(dx, dy)`.
   - Option d'acquisition de la position actuelle du curseur via un bouton "Capturer position actuelle".

3. **Modale d'Ajout / Édition de Pause (Délai)** :
   - Saisie de la durée d'attente en millisecondes (`ms`).
   - Présélections rapides (100ms, 250ms, 500ms, 1s, 2s, 5s).

4. **Modale d'Attente d'Image (`WaitImage`) & Image d'Arrêt (`StopImage`)** :
   - Choix de la source de l'image :
     - ⚡ Image intégrée de référence 1 : `embedded://extreme.png`.
     - ⚡ Image intégrée de référence 2 : `embedded://failed.PNG`.
     - 📁 Fichier image personnalisé sur le disque.
   - Intégration de la boîte de dialogue native Windows via la crate `rfd` (filtres `.png`, `.jpg`, `.bmp`).
   - Prévisualisation miniature de l'image sélectionnée.
   - Configuration du délai d'expiration (Timeout en ms) et de la tolérance de correspondance.

5. **Gestion de Sauvegarde / Chargement de Profils `.mforge`** :
   - Remplacement de `@tauri-apps/plugin-dialog` par `rfd::FileDialog` :
     ```rust
     // Exemple d'ouverture native :
     if let Some(path) = rfd::FileDialog::new()
         .add_filter("MacroForge Profile", &["mforge", "json"])
         .pick_file() {
         // Charger le fichier JSON...
     }
     ```
   - Messages de confirmation / erreur stylisés avec retours visuels (Toast notifications ou bannières).

---

## ✅ Critères d'Acceptation
- [ ] Les boîtes de dialogue natives de Windows (Open/Save) s'ouvrent sans aucun composant Web ni blocage de l'interface.
- [ ] La capture interactive de touches fonctionne sans conflit avec les raccourcis système.
- [ ] L'importation et l'exportation de fichiers `.mforge` conservent la compatibilité totale avec les profils existants.
