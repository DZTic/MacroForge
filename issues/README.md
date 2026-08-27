# 🚀 MacroForge — Suivi des Issues de Performance

## 🎯 Contexte
Audit approfondi et résolution des goulots d'étranglement de performance, de la latence d'entrée, du cycle de rendu egui et de la synchronisation thread du moteur natif Rust / Windows.

---

## 🏆 Issues Vague 2 Résolues & Intégrées (#16 à #23)

| Fiche Locale | Titre | Statut | Résumé de l'Optimisation |
|---|---|:---:|---|
| [`ISSUE-16`](./ISSUE-16-mutex-spin-wait-playback.md) | Contention Mutex dans la boucle de spin-wait du Playback | ✅ Résolu | `Arc<AtomicBool>` avec `load(Ordering::Relaxed)` dans le spin-wait |
| [`ISSUE-17`](./ISSUE-17-inondation-events-playback-ui.md) | Inondation d'événements UI & sur-repeinte egui lors du Playback | ✅ Résolu | Throttling à 30 FPS des événements de déplacement souris pour l'UI |
| [`ISSUE-18`](./ISSUE-18-allocation-heap-raw-input.md) | Allocation Heap sur chaque paquet Raw Input (1000Hz - 8000Hz) | ✅ Résolu | `MaybeUninit<RAWINPUT>` sur la pile et 1 seul appel `GetRawInputData` |
| [`ISSUE-19`](./ISSUE-19-verrouillage-mutex-macro-state-raw-input.md) | Verrouillage bloquant de `MACRO_STATE` sur chaque paquet Raw Input | ✅ Résolu | Pré-filtrage atomique `RAW_INPUT_RECORDING` & `RIGHT_MOUSE_DOWN` |
| [`ISSUE-20`](./ISSUE-20-reconfiguration-style-egui-per-frame.md) | Reconfiguration inconditionnelle de `egui::Style` à chaque frame | ✅ Résolu | Style initialisé au démarrage et préservé sans cloning par frame |
| [`ISSUE-21`](./ISSUE-21-allocations-layout-widgets-ui.md) | Allocations de chaînes et layout inconditionnel (`GlassButton`) | ✅ Résolu | Construction de libellé sans allocation superflue ni cloning |
| [`ISSUE-22`](./ISSUE-22-capture-multi-ecrans-zone-cible.md) | Capture plein écran virtuel multi-écrans en `WaitImage` et `StopImage` | ✅ Résolu | `get_screen_capture_bounds()` ciblant le rectangle de la fenêtre de jeu |
| [`ISSUE-23`](./ISSUE-23-atomique-last-game-hwnd-et-rdev-allocs.md) | Remplacement du Mutex de Focus Tracker par `AtomicIsize` & optimisations rdev | ✅ Résolu | `AtomicIsize` lock-free et `Cow<'static, str>` pour les touches `rdev` |

---

## 🏆 Issues Vague 1 Résolues & Intégrées (#10 à #15)

| Fiche Locale | Titre | Statut | Résumé |
|---|---|:---:|---|
| [`ISSUE-10`](./ISSUE-10-virtualisation-liste-actions-ui.md) | Virtualisation de la Liste d'Actions | ✅ Résolu | `ScrollArea::show_rows` virtualisé pour 10 000+ actions |
| [`ISSUE-11`](./ISSUE-11-capture-ecran-dxgi-reutilisation-buffer.md) | Optimisation Capture Écran | ✅ Résolu | Buffer réutilisé `TLS_SCREEN_BUFFER` & Rayon early-exit |
| [`ISSUE-12`](./ISSUE-12-timer-windows-granularite-sleep.md) | Granularité Timer Windows | ✅ Résolu | `TimerResolutionGuard` avec `timeBeginPeriod(1)` |
| [`ISSUE-13`](./ISSUE-13-logs-asynchrones-gated-debug.md) | Logs Asynchrones / Gating Debug | ✅ Résolu | `log` / `env_logger` configuré (warn en release, debug en dev) |
| [`ISSUE-15`](./ISSUE-15-cache-filtre-recherche-frame.md) | Cache du Filtrage & Recherche | ✅ Résolu | Cache `filtered_indices` indexé sur `actions_version` |

---

## 🔒 Rappel des Principes Non Négociables
- **Arrêt d'Urgence F4** : instantané via atomique `AtomicBool` (`Ordering::SeqCst`).
- **Raccourcis Globaux F8 / F9** : enregistrement réactif via `rdev` et `RegisterHotKey`.
- **Thread Safety** : zéro contention mutex sur les chemins chauds (Raw Input à 1000Hz+, spin-wait sub-milliseconde).
- **Zéro Régression** : précision temporelle absolue préservée, 39 tests unitaires validés au vert.
