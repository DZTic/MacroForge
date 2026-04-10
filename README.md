# MacroForge 🛠️

**MacroForge** est un moteur de macros haute performance pour Windows, conçu pour la précision, la fiabilité et le support des jeux FPS. Développé avec **Tauri**, **Rust** et **TypeScript**, il offre une solution robuste pour l'automatisation avancée.

![MacroForge](public/logo.png) <!-- Optionnel : si le logo existe -->

## ✨ Fonctionnalités Clés

- ⏱️ **Timing Haute Précision** : Utilise un système de chronologie absolue pour éliminer toute dérive temporelle, même sur de longues durées ou de nombreuses répétitions.
- 🖱️ **Mode Raw Input (FPS)** : Capture les deltas de la souris directement via l'API Windows Raw Input, permettant d'enregistrer et de rejouer des mouvements de caméra fluides dans des jeux comme Roblox, Minecraft ou tout autre FPS.
- 🖼️ **Détection d'Images Ultra-Rapide** : Capture d'écran via GDI et recherche parallélisée (système Rayon) pour détecter des éléments visuels à l'écran avec une latence minimale.
- 📺 **Overlay de Débogage** : Une interface transparente s'affiche par-dessus vos applications pour vous montrer en temps réel l'action en cours d'exécution.
- 🔄 **Boucles Robustes** : Mode répétition infinie ou limitée avec une gestion stable de la mémoire et des ressources.
- 🛑 **Arrêt d'Urgence (F10)** : Une touche de sécurité globale pour arrêter instantanément toute macro en cours.
- 💾 **Gestion de Profils** : Sauvegardez et chargez vos macros au format JSON pour les partager ou les réutiliser plus tard.
- 🗔 **Multi-Fenêtrage** : Interface principale, barre d'outils flottante et overlay de monitoring.

## 🚀 Installation

### Prérequis

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) (LTS recommandé)
- Windows (obligatoire pour les fonctionnalités bas niveau comme le Raw Input et GDI)

### Installation des dépendances

```bash
npm install
```

### Lancement en mode développement

```bash
npm run tauri dev
```

### Construction de l'exécutable

```bash
npm run tauri build
```

## 🛠️ Stack Technique

- **Backend** : Rust (Tauri v2)
  - `rdev` pour l'écoute globale des entrées.
  - `winapi` pour les interactions Windows de bas niveau (Raw Input, GDI, SendInput).
  - `rayon` pour le traitement d'image haute performance.
  - `serde` pour la sérialisation des macros.
- **Frontend** : TypeScript + HTML/CSS Vanille
  - `Vite` pour le bundling.
- **UI** : Design moderne avec effets de verre (glassmorphism) et animations fluides.

## ⌨️ Raccourcis Clavier (Hotkeys)

- **F8** : Démarrer l'enregistrement.
- **F9** : Arrêter l'enregistrement.
- **F10** : Arrêt d'urgence de la macro en cours de lecture.

## 📖 Utilisation

1. **Enregistrement** : Appuyez sur **F8** ou cliquez sur "Enregistrer". Effectuez vos actions.
   - **Mode FPS** : Pour capturer les mouvements de caméra dans un jeu FPS, maintenez le **clic droit** enfoncé pendant l'enregistrement. MacroForge passera automatiquement en mode **Raw Input** pour capturer les déplacements physiques de la souris (deltas) plutôt que les coordonnées à l'écran.
2. **Édition** : Visionnez et modifiez les actions capturées dans l'interface principale.
3. **Lecture** : Cliquez sur "Jouer la Macro". L'overlay apparaîtra pour suivre la progression et vous indiquer quelle action est en cours.
4. **Image Search** : Utilisez le bouton "Image" pour ajouter une recherche visuelle. La macro attendra que l'image cible apparaisse à l'écran avant de continuer.
5. **Boucles** : Activez l'option "Boucler" pour répéter la séquence indéfiniment.

## ⚠️ Sécurité et Éthique

MacroForge est un outil puissant. Veillez à l'utiliser de manière responsable. L'automatisation dans certains jeux peut être à l'encontre de leurs conditions d'utilisation.

---

Développé avec ❤️ par [DZTic](https://github.com/DZTic)
