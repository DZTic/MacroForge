# MacroForge ⚡

**MacroForge** est un moteur de macro ultra-haute performance pour Windows, développé à 100% en **Rust natif** avec **egui / eframe** et les API Win32 bas niveau. Conçu pour la vitesse d'exécution, la précision temporelle absolue et l'automatisation dans les jeux et applications.

---

## ✨ Fonctionnalités Principales

- 🚀 **100% Rust Autonome** : Zéro dépendance Node.js, Web, Vite ou WebView2. Binaire unique ultra-léger et démarrage instantané (< 50 ms).
- 🌍 **Internationalisation Native (i18n)** : Interface complète en **Français** et **Anglais** commutable instantanément sans redémarrage.
- ⏱️ **Précision Temporelle Absolue** : Horloge haute précision sans dérive temporelle pour les séquences longues.
- 🖱️ **Mode Raw Input (FPS)** : Capture et injection des mouvements relatifs ($\Delta X, \Delta Y$) pour les caméras de jeux 3D.
- 🖼️ **Vision par Ordinateur (GDI / Rayon)** : Détection d'image multi-threadée ultra-rapide (`WaitImage` & `StopImage`).
- 🛑 **Arrêt d'Urgence Visuel & Clavier** : Interruption immédiate via touche **F4** ou détection de motif d'image critique.
- 🗔 **Toolbar Flottante Native** : Mini-contrôleur compact multi-viewport déplaçable avec drag handle.
- 👻 **Overlay Transparent Click-Through** : HUD temps réel au-dessus des jeux sans bloquer les clics (`WS_EX_TRANSPARENT`) et invisible pour les captures GDI (`WDA_EXCLUDEFROMCAPTURE`).
- 📋 **Éditeur avec Drag & Drop** : Réorganisation intuitive des actions, filtres, recherche et duplication instantanée.
- 💾 **Profils `.mforge`** : Sauvegarde et ouverture rapides via les boîtes de dialogue natives Windows (`rfd`).

---

## 🚀 Installation & Compilation

### Prérequis

- [Rust & Cargo](https://www.rust-lang.org/tools/install) (Toolchain stable recommandée)
- Système d'exploitation Windows 10/11 (64-bit)

### Lancer en Mode Développement

```bash
cargo run
```

### Compiler le Binaire Release Optimisé

```bash
cargo build --release
```

Le fichier exécutable autonome est généré dans `target/release/macroforge.exe`.

---

## 🛠️ Stack Technique

- **Moteur & UI** : Rust 2021 + `eframe` / `egui` (Rendu matériel Glow / OpenGL / DirectX)
- **Capture & Injection d'Entrées** : `rdev` + Win32 `SendInput` / Raw Input
- **Traitement d'Image & Vision** : `rayon` (parallélisation CPU) + `image` + Win32 GDI
- **Dialogues de Fichiers** : `rfd` (Rust File Dialogs natif)
- **Sérialisation** : `serde` + `serde_json`

---

## ⌨️ Raccourcis Clavier Globaux

| Raccourci | Action |
|:---|:---|
| **F8** | Démarrer l'enregistrement en direct |
| **F9** | Arrêter l'enregistrement |
| **F4** | Arrêt d'urgence immédiat de la relecture |

---

## ⚠️ Sécurité & Éthique

MacroForge est un outil d'automatisation puissant. Veuillez l'utiliser de manière responsable. L'automatisation dans certains jeux en ligne peut être soumise à leurs conditions d'utilisation.
