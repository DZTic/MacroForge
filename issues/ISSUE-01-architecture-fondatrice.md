# 📌 Issue #01 : Architecture Fondatrice & Setup Standalone (`egui` / `eframe`)

- **Statut** : 📝 À faire
- **Priorité** : 🔴 Bloquant
- **Composants** : Backend Rust, Cargo.toml, Event Loop, Window Manager
- **Agent Référent** : `.agents/agents/rust-core.md`

---

## 🎯 Description du Besoin
Actuellement, l'application dépend de Tauri v2 et de WebView2 pour instancier ses fenêtres et gérer la communication IPC entre le backend Rust et l'interface TypeScript.
Il est nécessaire d'initialiser une application Rust autonome (standalone) propulsée par `eframe` / `egui`, sans dépendance à Tauri ni à un moteur de rendu Web.

---

## 📋 Tâches Techniques

1. **Configuration du Cargo Manifest (`Cargo.toml`)** :
   - Ajouter les dépendances GUI :
     - `eframe = { version = "0.29", default-features = false, features = ["default_fonts", "glow", "persistence", "winit"] }` (ou backend DirectX/wgpu).
     - `egui = "0.29"`
     - `egui_extras = { version = "0.29", features = ["all_loaders", "image"] }`
     - `rfd = "0.15"` (pour les boîtes de dialogue de fichiers natives Windows).
   - Conserver les dépendances essentielles du moteur : `rdev`, `winapi`, `rayon`, `image`, `serde`, `serde_json`, `lazy_static`.
   - Supprimer les dépendances `tauri`, `tauri-build`, `tauri-plugin-*`.

2. **Refactorisation de `macro_core.rs` (Découplage de Tauri)** :
   - Remplacer les appels `tauri::AppHandle` et `handle.emit(...)` par un système de canaux d'événements Rust (`std::sync::mpsc` ou `crossbeam-channel`) ou des signaux partagés atomiques.
   - Les événements à router vers l'UI native :
     - `RecordingStateChanged(bool)`
     - `PlaybackStateChanged(bool)`
     - `PlaybackAction(PlaybackActionPayload)`
   - Supprimer toutes les macros `#[tauri::command]`.

3. **Création de la Structure de l'Application Native (`MacroForgeApp`)** :
   - Implémenter le trait `eframe::App`.
   - Gérer l'état global partagé dans une structure propre :
     ```rust
     pub struct MacroForgeApp {
         pub state: Arc<Mutex<MacroState>>,
         pub rx_events: Receiver<EngineEvent>,
         pub active_window: WindowMode, // Main, ToolbarOnly, etc.
         pub show_toolbar: bool,
         pub show_overlay: bool,
         pub theme: CustomTheme,
         pub i18n: I18nService,
     }
     ```

4. **Gestion Multi-fenêtrage Native (Multi-viewport / Windows Hooks)** :
   - Configurer le support des viewports `egui` (`eframe::NativeOptions { viewport: egui::ViewportBuilder::default() }`).
   - Mettre en place la boucle d'écoute `rdev::listen` dans un thread détaché.
   - Démarrer le tracker de focus Win32 (`start_focus_tracker`).

---

## ✅ Critères d'Acceptation
- [ ] Le binaire se compile directement via `cargo build` sans aucune dépendance à Node.js ni WebView2.
- [ ] La fenêtre principale native s'ouvre avec un rendu GPU fluide.
- [ ] Les raccourcis globaux F8 (enregistrement), F9 (arrêt enregistrement) et F4 (arrêt d'urgence) continuent de fonctionner parfaitement en arrière-plan.
- [ ] Aucun verrouillage (`lock()`) bloquant sur `MACRO_STATE`.
