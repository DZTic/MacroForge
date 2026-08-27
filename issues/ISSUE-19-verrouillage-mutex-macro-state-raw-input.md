# 🔒 Issue #19 : Verrouillage Bloquant de `MACRO_STATE` sur chaque Message Raw Input

- **Statut** : ✅ Résolu (intégré)
- **Priorité** : 🟠 Haute
- **Composants** : `macro_core.rs`, `spawn_raw_input_listener()`, `MACRO_STATE`
- **Agents Référents** : `.agents/agents/rust-core.md`

---

## 🎯 Description du Problème

Dans `spawn_raw_input_listener()`, chaque déplacement relatif de souris (détecté à très haute fréquence) déclenche :

```rust
if dx != 0 || dy != 0 {
    let mut state = MACRO_STATE.lock().unwrap();
    if state.is_recording && state.is_right_mouse_down {
        // ...
    } else if state.is_recording {
        state.pending_dx = 0;
        state.pending_dy = 0;
    }
}
```

### Impact Performance
1. Le mutex `MACRO_STATE` protège l'ensemble de l'état de l'application (liste des actions, configuration d'arrêt, flags).
2. Prendre ce verrou à chaque paquet de souris (jusqu'à 1000 fois/seconde), même quand l'application n'enregistre pas ou que le clic droit n'est pas actif, entre en compétition directe avec le thread principal egui qui lit/écrit dans `MACRO_STATE` pour afficher l'interface.
3. Risque de micro-gels (stuttering) dans les jeux et saccades de l'interface egui.

---

## 📋 Tâches Techniques

1. Déplacer `is_recording` et `is_right_mouse_down` vers des `AtomicBool` globaux rapides (ex. `RECORDING_ACTIVE` et `RIGHT_MOUSE_ACTIVE`).
2. Dans le thread Raw Input, vérifier ces atomiques avec `Ordering::Relaxed` **avant** de tenter d'acquérir le mutex `MACRO_STATE`.
3. Conserver l'accumulation des `pending_dx` et `pending_dy` dans des atomiques ou dans des variables locales au thread d'écoute, et ne verrouiller `MACRO_STATE` qu'au moment d'écrire l'action à intervalle régulier (8 ms).

---

## ✅ Critères d'Acceptation

- [ ] Zéro verrouillage de `MACRO_STATE` lorsque l'enregistrement est inactif ou en dehors du mode visée (clic droit).
- [ ] Réduction de > 90% du temps de rétention du mutex `MACRO_STATE`.
- [ ] Aucun impact sur la fluidité du jeu hôte ou de l'UI.
