# ⚡ Issue #16 : Contention de verrou Mutex dans la boucle de timing du Playback

- **Statut** : ✅ Résolu (intégré)
- **Priorité** : 🔴 Critique
- **Composants** : `macro_core.rs`, boucle de playback, gestion de l'arrêt d'urgence
- **Agents Référents** : `.agents/agents/rust-core.md`

---

## 🎯 Description du Problème

Dans `macro_core.rs`, la fonction `play_macro()` utilise une boucle d'attente active de haute précision pour aligner le timing de chaque action :

```rust
let target_time = timeline_origin + Duration::from_millis(total_recorded_delay);
loop {
    let now = Instant::now();
    if now >= target_time {
        break;
    }
    let diff = target_time.duration_since(now).as_millis();
    if diff > 10 {
        thread::sleep(Duration::from_millis(1));
    } else if diff > 1 {
        thread::yield_now();
    } else {
        std::hint::spin_loop();
    }

    if *stop_flag.lock().unwrap() {
        break 'main_loop;
    }
    // ...
}
```

Lorsque le délai restant est court (`diff <= 1 ms`), la boucle tourne en `spin_loop()`. À chaque itération de spin (pouvant atteindre des centaines de milliers d'itérations par seconde), le code appelle :
`*stop_flag.lock().unwrap()`

### Impact Performance
1. **Contention atomique et cache-line bouncing** : Verrouiller et déverrouiller un `std::sync::Mutex` en boucle serrée génère des instructions atomiques avec barrières mémoire lourdes (`LOCK CMPXCHG`), saturant le bus mémoire inter-cœurs.
2. **Gigue temporelle** : L'acquisition du verrou ajoute des micro-latences imprévisibles au moment critique où l'action doit être envoyée à la milliseconde exacte.
3. **Consommation CPU excessive**.

---

## 📋 Tâches Techniques

1. Remplacer le type de `stop_playback_flag` dans `MacroState` de `Arc<Mutex<bool>>` par `Arc<AtomicBool>`.
2. Dans la boucle de timing de `play_macro()`, remplacer `*stop_flag.lock().unwrap()` par `stop_flag.load(Ordering::Relaxed)`.
3. Mettre à jour `stop_playback()` et `emergency_stop()` pour exécuter `stop_flag.store(true, Ordering::SeqCst)`.
4. Réinitialiser le flag lors du démarrage de lecture avec `stop_flag.store(false, Ordering::SeqCst)`.

---

## ✅ Critères d'Acceptation

- [ ] Zéro appel à `Mutex::lock()` dans la boucle de spin-wait du playback.
- [ ] La lecture s'arrête instantanément lors de l'appui sur F4 (`emergency_stop()`).
- [ ] Les tests de timing et de macro manipulation passent avec succès.
