# 🌊 Issue #17 : Inondation d'Événements UI et Sur-Repeinte egui lors du Playback

- **Statut** : ✅ Résolu (intégré)
- **Priorité** : 🟠 Haute
- **Composants** : `macro_core.rs`, `events.rs`, `app.rs`, canal MPSC, HUD Overlay
- **Agents Référents** : `.agents/agents/rust-core.md`, `.agents/agents/frontend-ui.md`

---

## 🎯 Description du Problème

Pendant la relecture d'une macro (`play_macro()`), chaque action émet un événement vers l'UI :

```rust
emit_playback_action(PlaybackActionPayload {
    index: action_index,
    total: total_actions,
    action_type: "Move".into(),
    x,
    y,
    detail: format!("Pos {}x{}", x, y),
});
```

Et la fonction `notify_event()` :

```rust
fn notify_event(event: EngineEvent) {
    if let Some(ref sender) = *EVENT_SENDER.lock().unwrap() {
        let _ = sender.send(event);
    }
    if let Some(ref ctx) = *EGUI_CTX.lock().unwrap() {
        ctx.request_repaint();
    }
}
```

### Impact Performance
1. **Sur-repeinte egui** : Pour une macro comportant des centaines ou milliers de micro-mouvements de souris (capturés à 60-120 Hz via RawInput), `ctx.request_repaint()` force egui à redessiner toute l'interface à plus de 125 FPS en continu.
2. **Allocations inutiles** : 2 allocations `String` par micro-mouvement (`"Move".into()` et `format!("Pos {}x{}", x, y)`), soit des milliers d'allocations heap par seconde.
3. **Pression sur la queue MPSC** : L'UI doit dépiler des milliers de messages `EngineEvent::PlaybackAction` à chaque frame.

---

## 📋 Tâches Techniques

1. **Throttling des notifications UI pour les mouvements de souris** : Limiter les émissions de `PlaybackAction` pour `MouseMove` et `MouseMoveRelative` à un intervalle minimal (ex. max 30 Hz ou toutes les 33 ms) pendant le playback, ou n'émettre que les changements de type d'action / actions clés (clics, touches, attentes).
2. **Éviter les allocations `String`** : Remplacer `action_type: String` par un `enum` ou un `&'static str` (`ActionKind`), et générer le texte `detail` à la demande côté UI uniquement si affiché.
3. **Conserver la réactivité du HUD Overlay** sans saturer le pipeline graphique.

---

## ✅ Critères d'Acceptation

- [ ] L'exécution d'une macro de 5 000 actions de déplacement souris n'augmente pas l'utilisation CPU de l'UI de plus de 5%.
- [ ] Le nombre de repeintes egui pendant le playback est plafonné à 60 FPS.
- [ ] L'overlay et la toolbar restent fluides et à jour.
