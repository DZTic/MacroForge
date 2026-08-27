# 🐁 Issue #18 : Allocation Heap sur chaque Paquet Raw Input Souris (1000Hz - 8000Hz)

- **Statut** : ✅ Résolu (intégré)
- **Priorité** : 🔴 Critique
- **Composants** : `macro_core.rs`, `spawn_raw_input_listener()`, Win32 Raw Input API
- **Agents Référents** : `.agents/agents/rust-core.md`, `.agents/agents/vision-automation.md`

---

## 🎯 Description du Problème

Dans `spawn_raw_input_listener()`, la boucle de messages Win32 traite les événements `WM_INPUT` :

```rust
if msg.message == WM_INPUT {
    let mut size: u32 = 0;
    GetRawInputData(
        msg.lParam as *mut _,
        RID_INPUT,
        std::ptr::null_mut(),
        &mut size,
        std::mem::size_of::<RAWINPUTHEADER>() as u32,
    );

    let mut buffer = vec![0u8; size as usize]; // ⚠️ ALLOCATION HEAP À CHAQUE MESSAGE !
    if GetRawInputData(
        msg.lParam as *mut _,
        RID_INPUT,
        buffer.as_mut_ptr() as *mut _,
        &mut size,
        std::mem::size_of::<RAWINPUTHEADER>() as u32,
    ) == size
    {
        let raw = &*(buffer.as_ptr() as *const RAWINPUT);
        // ...
    }
}
```

### Impact Performance
1. Les souris gaming modernes envoient des rapports à **1000 Hz, 4000 Hz, voire 8000 Hz** (1 000 à 8 000 paquets `WM_INPUT` par seconde).
2. Allouer un `vec![0u8; size as usize]` pour chaque message produit des **milliers d'allocations et de désallocations heap par seconde**, créant une forte fragmentation et une contention sur l'allocateur global du runtime Rust.
3. Deux appels consécutifs à `GetRawInputData` (le premier pour interroger la taille, le second pour récupérer les données) doublent le coût système par rapport à une structure de taille fixe connue.

---

## 📋 Tâches Techniques

1. Utiliser un buffer fixe sur la pile de taille `std::mem::size_of::<RAWINPUT>()` (ou un tableau `[u8; 64]` aligné / struct `std::mem::MaybeUninit<RAWINPUT>`).
2. Effectuer un appel direct à `GetRawInputData` avec la taille fixe du buffer, éliminant ainsi le premier appel de détermination de taille.
3. Valider la taille retournée par rapport à `size_of::<RAWINPUTHEADER>()`.

---

## ✅ Critères d'Acceptation

- [ ] Zéro allocation heap (`Vec`, `Box`, `String`) dans la boucle de message `WM_INPUT`.
- [ ] Réduction de 2 à 1 appel système `GetRawInputData` par paquet de souris.
- [ ] Fonctionnement vérifié avec souris gaming 1000 Hz+ sans hausse d'allocation mémoire.
