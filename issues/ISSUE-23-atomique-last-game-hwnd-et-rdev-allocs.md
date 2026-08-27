# 🎯 Issue #23 : Remplacement du Mutex de Focus Tracker par `AtomicIsize` & Optimisation `rdev`

- **Statut** : ✅ Résolu (intégré)
- **Priorité** : 🟢 Basse
- **Composants** : `macro_core.rs`, `start_focus_tracker()`, `handle_rdev_event()`
- **Agents Référents** : `.agents/agents/rust-core.md`

---

## 🎯 Description du Problème

1. **`LAST_GAME_HWND` sous Mutex** :
   Dans `macro_core.rs`, `LAST_GAME_HWND` est un `Mutex<isize>`. Le thread d'arrière-plan de suivi de focus le verrouille toutes les 200 ms.
   Un type atomique (`AtomicIsize` ou `AtomicUsize`) permet une lecture et écriture lock-free sans aucun overhead de mutex.

2. **Allocation `format!("{:?}", key)` dans `handle_rdev_event`** :
   Dans `handle_rdev_event(event: Event)` :
   `rdev_key_to_name_and_scan(key)` exécute `let name = format!("{:?}", key);` dès le début de la fonction sur chaque événement de touche, y compris pour les touches rejetées (raccourcis F8/F9/F4 ou doublons rapides).

---

## 📋 Tâches Techniques

1. Remplacer `pub static ref LAST_GAME_HWND: Mutex<isize>` par `pub static LAST_GAME_HWND: AtomicIsize = AtomicIsize::new(0);`.
2. Différer la construction du nom de touche (`format!("{:?}", key)`) ou utiliser un mappage statique `&'static str` pour les touches courantes (`Return`, `Space`, `A`..`Z`, `F1`..`F12`), évitant ainsi toute allocation de `String` à la frappe.

---

## ✅ Critères d'Acceptation

- [ ] `LAST_GAME_HWND` accessible de manière 100% lock-free via `AtomicIsize`.
- [ ] Suppression des allocations `String` anticipées dans le callback `rdev`.
- [ ] Tous les tests unitaires continuent de passer.
