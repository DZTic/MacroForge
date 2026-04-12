# MacroForge 🛠️

**MacroForge** est un moteur de macros haute performance pour Windows, conçu pour la précision, la fiabilité et le support des jeux FPS. Développé avec **Tauri**, **Rust** et **TypeScript**, il offre une solution robuste pour l'automatisation avancée.

![MacroForge](public/logo.png) <!-- Optionnel : si le logo existe -->

## ✨ Fonctionnalités Clés

- 🌍 **Support Multilingue (i18n)** : Interface disponible en **Français** et **Anglais** avec basculement instantané.
- ⏱️ **Timing Haute Précision** : Système de chronologie absolue pour éliminer toute dérive temporelle, même sur de longues durées.
- 🖱️ **Mode Raw Input (FPS)** : Capture les deltas de la souris directement via l'API Windows, idéal pour les mouvements de caméra dans les jeux (Roblox, Minecraft, etc.).
- 🖼️ **Vision par Ordinateur (GDI/Rayon)** : Détection d'images ultra-rapide avec recherche parallélisée pour une latence minimale.
- 🛑 **Arrêt d'Urgence Visuel** : Définissez une "image d'arrêt" qui interrompt immédiatement la macro si elle est détectée à l'écran.
- 🗔 **Interface Flottante (Toolbar)** : Une barre d'outils compacte pour piloter vos macros tout en restant concentré sur votre tâche.
- ↕️ **Édition Intuitive** : Réorganisez vos actions par **Glisser-Déposer (Drag & Drop)** et modifiez les délais ou coordonnées en un clic.
- 📺 **Overlay de Monitoring** : Interface transparente affichant l'action en cours d'exécution par-dessus vos applications.
- 💾 **Gestion de Profils** : Exportez/Importez vos macros au format `.mforge` (JSON).

## 🚀 Installation

### Prérequis

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) (LTS recommandé)
- Windows (obligatoire pour les fonctionnalités bas niveau)

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
  - `rdev` : Captures globales d'entrées.
  - `winapi` : Interactions Raw Input, GDI, SendInput.
  - `rayon` : Parallélisation du traitement d'images.
- **Frontend** : TypeScript + HTML/CSS Moderne
  - `Vite 6` : Bundling ultra-rapide.
- **UI/UX** : Design Premium (Glassmorphism), animations fluides et système d'infobulles intelligent.

## ⌨️ Raccourcis Clavier (Hotkeys)

- **F8** : Démarrer / Arrêter l'enregistrement.
- **F9** : Arrêt forcé de l'enregistrement.
- **F10** : Arrêt d'urgence de la macro en cours de lecture.

## 📖 Utilisation

1. **Enregistrement** : Appuyez sur **F8** pour commencer.
   - **Note FPS** : Maintenez le **clic droit** pour capturer les mouvements relatifs (deltas) dans les jeux.
2. **Édition** : Utilisez la liste principale pour réordonner les actions (Drag & Drop) ou les supprimer/éditer.
3. **Barre d'outils** : Cliquez sur "Toolbar" pour passer en mode mini-contrôleur flottant.
4. **Image Search** : Ajoutez une action "Image" pour attendre qu'un élément visuel apparaisse avant de continuer.
5. **Boucles** : Activez "Boucler" pour répéter la macro à l'infini.

## ⚠️ Sécurité et Éthique

MacroForge est un outil puissant. Veillez à l'utiliser de manière responsable. L'automatisation dans certains jeux peut être à l'encontre de leurs conditions d'utilisation.

---

Développé avec ❤️ par [DZTic](https://github.com/DZTic)
