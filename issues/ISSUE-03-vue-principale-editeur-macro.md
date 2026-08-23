# 📌 Issue #03 : Vue Principale & Éditeur de Macro (Drag & Drop natif)

- **Statut** : ✅ Terminé
- **Priorité** : 🔴 Critique
- **Composants** : MainWindow UI, Timeline, Drag & Drop, Filtering, Actions Editor
- **Agents Référents** : `.agents/agents/frontend-ui.md`, `.agents/agents/rust-core.md`
- **Dépendances** : Issue #01, Issue #02

---

## 🎯 Description du Besoin
La fenêtre principale (`MainWindow`) est le cœur opérationnel de MacroForge. Elle doit permettre de visualiser l'ensemble des actions enregistrées sous forme de liste chronologique (Timeline), d'en modifier l'ordre par glisser-déposer (Drag & Drop), de filtrer les déplacements souris, d'accéder rapidement à une action précise, et de piloter l'enregistrement et la lecture.

---

## 📋 Tâches Techniques

1. **Barre Supérieure d'Actions Rapides (Header & Quick Actions)** :
   - Bouton d'ajout d'action manuelle :
     - ⌨️ Clavier (`KeyPress` / `KeyRelease`)
     - 🖱️ Souris (`MousePress` / `MouseRelease` / `MouseMove`)
     - ⏱️ Pause (`Wait`)
     - 🖼️ Image (`WaitImage`)
   - Boutons de profil : 💾 Sauvegarder (`.mforge`) / 📂 Ouvrir (`.mforge`).
   - Bouton d'ouverture de la Toolbar flottante (`🗔 Toolbar`).
   - Sélecteur de langue (FR / EN).

2. **Panneau Central : Liste des Actions (Timeline)** :
   - Défilement haute performance (`egui::ScrollArea::vertical()`) capable d'afficher des milliers d'actions sans saccade.
   - **Glisser-Déposer Natif (Drag & Drop)** :
     - Utilisation d'une gestion intuitive par poignée de saisie (`drag_handle`).
     - Affichage d'une ligne d'insertion visuelle lors du survol d'un emplacement cible.
     - Réorganisation atomique du vecteur `actions` dans `MACRO_STATE`.
   - Actions inline sur chaque élément :
     - Bouton ✏️ Éditer (ouvre la modale d'édition).
     - Bouton 🗑️ Supprimer.
     - Affichage / modification directe du délai avant action (`delay_ms`).

3. **Barre d'Outils et Filtres de Vue** :
   - **Saut rapide ("Aller à l'action n°X")** : Champ de saisie numérique + validation avec défilement automatique vers l'élément (`scroll_to_me`).
   - **Filtre "Mouvements Souris"** : Interrupteur permettant de masquer les milliers d'événements `MouseMove` / `MouseMoveRelative` pour concentrer l'affichage sur les clics, touches et pauses.
   - **Compteur d'actions** : Badge affichant le nombre d'actions visibles vs total (`X visibles / Y total`).

4. **Barre Inférieure de Contrôle Global (Footer)** :
   - Bouton **Enregistrer (F8)** / **Arrêter (F9)** avec indicateur d'état dynamique.
   - Bouton **Jouer la Macro** (Arrêt d'urgence F4 rappelé).
   - Case à cocher **Boucler** (`loop_playback`).
   - Bouton de configuration **Image d'arrêt d'urgence** avec statut actif/inactif.

---

## ✅ Critères d'Acceptation
- [x] La liste des actions est fluide à 60/120 FPS même avec plus de 5 000 actions dans la macro.
- [x] Le réordonnancement par glisser-déposer fonctionne de manière fiable et instantanée.
- [x] Le masquage des mouvements souris allège l'interface sans altérer les données sous-jacentes.
- [x] Le lancement et l'arrêt de la macro depuis les boutons répliquent fidèlement le comportement du moteur Rust.
