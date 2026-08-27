# ⏱️ Issue #12 : Granularité Timer Windows (timeBeginPeriod)

- **Statut** : ✅ Résolu (intégré)
- **Priorité** : 🟠 Haute
- **Composants** : `macro_core.rs`, thread::sleep, winmm timeBeginPeriod
- **Agents Référents** : `.agents/agents/rust-core.md`

---

## 🎯 Description du Problème

La boucle d'attente du playback utilise :

```rust
if diff > 10 { thread::sleep(Duration::from_millis(1)); }
else if diff > 1 { thread::yield_now(); }
else { std::hint::spin_loop(); }
```

Sans `timeBeginPeriod(1)`, la granularité effective du scheduler Windows est d'environ **15,6 ms** (timer tick par défaut). Les `sleep(1ms)` deviennent donc des sleeps réels de 1 a 15 ms, ce qui introduit une gigue temporelle sur les macros a haute precision — contradictoire avec la promesse "High Precision Timing" du README.

Le spin-loop final compense en partie mais brûle inutilement du CPU sur la durée.

## 📋 Tâches Techniques

1. Au démarrage du playback (`play_macro()`), appeler `timeBeginPeriod(1)` via winapi (feature `winmm`).
2. À la fin du playback, appeler systématiquement `timeEndPeriod(1)` (y compris sur break `'main_loop`).
3. Envisager un guard RAII (`struct TimerResolutionGuard`) pour garantir l'appariement begin/end même en cas de panic.
4. Mesurer la gigue réelle avant/après sur une macro de 100 delays de 10ms chacun (écart-type du delay effectif).

## ✅ Critères d'Acceptation

- [ ] Gigue mesurée < ±2 ms sur des delays de 10 ms répétés.
- [ ] `timeEndPeriod` toujours appelé après arrêt du playback (F4 inclus).
- [ ] Aucune fuite de résolution timer après fermeture propre de l'app.

