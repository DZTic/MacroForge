# 🔇 Issue #13 : Logs Asynchrones / Gating Debug

- **Statut** : ✅ Résolu (intégré)
- **Priorité** : 🟡 Moyenne
- **Composants** : `macro_core.rs`, println!, I/O console synchrone
- **Agents Référents** : `.agents/agents/rust-core.md`, `.agents/agents/qa-security.md`

---

## 🎯 Description du Problème

Une quinzaine de `println!` parsèment le chemin chaud du playback (`play_macro()`, `WaitImage`, stop-image...). L'I/O console sous Windows est **synchrone et lente** (syscall WriteConsole) — chaque log peut coûter 0,1 a 5 ms selon le terminal. Sur une macro de 500 actions rapides, cela cumule plusieurs dizaines de ms de derive.

En release, ces logs restent également visibles si un stdout existe (fenêtre console cachée par `windows_subsystem = "windows"` mais écritures toujours exécutées).

## 📋 Tâches Techniques

1. Introduire le crate `log` + `env_logger` (ou `tracing`) en remplacement direct des `println!`.
2. Niveau par défaut `warn` en release ; `debug`/`trace` activables via variable d'environnement `RUST_LOG=debug`.
3. Alternative légère : macro custom gated derrière `#[cfg(debug_assertions)]` si on veut éviter une dépendance.
4. Supprimer les logs de debug verbeux (`[#idx/total]` par action) ou les passer en niveau `trace`.
5. Conserver les messages d'erreur réels (échec chargement image, échec registration RawInput) en niveau `error`.

## ✅ Critères d'Acceptation

- [ ] Aucun `println!` restant dans le chemin chaud du playback.
- [ ] Mode verbose activable sans recompilation via `RUST_LOG`.
- [ ] Temps d'exécution d'une macro de 200 actions identiques avec/sans logging activé (écart < 1%).

