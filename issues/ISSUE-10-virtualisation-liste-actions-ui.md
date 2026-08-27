# ⚡ Issue #10 : Virtualisation de la Liste d'Actions (ScrollArea)

- **Statut** : ✅ Résolu (intégré)
- **Priorité** : 🟠 Haute
- **Composants** : `app.rs`, egui ScrollArea, Rendu UI
- **Agents Référents** : `.agents/agents/rust-core.md`

---

## 🎯 Description du Problème

Dans `MacroForgeApp::update()`, la timeline redessine **toutes** les `ActionCard` à chaque frame, y compris celles hors écran :

```rust
for (idx, action) in self.actions_cache.iter_mut().enumerate() {
    let card = ActionCard::new(idx, action).show(ui);
    // ...
}
```

Avec quelques centaines voire milliers d'actions (macros longues), le coût de layout/dessin devient linéaire sur la taille totale de la liste, ce qui fait chuter les FPS de l'UI et rend le scroll perceptiblement lent.

## 📋 Tâches Techniques

1. Remplacer `ScrollArea::vertical().show(...)` + boucle complète par `ScrollArea::vertical().show_rows(ui, row_height, total_rows, |ui, row_range| ...)`.
2. Ne layouter que les cartes dont l'index appartient à `row_range`.
3. Gérer correctement `scroll_to_me` avec l'index cible (`scroll_target_index`) dans le contexte virtualisé.
4. Préserver le Drag & Drop et les événements de carte (edit/delete/move) dans le rendu partiel.
5. Benchmarker avant/après avec une macro générée de 1000 actions (mesurer frame time via `ctx.input(|i| i.time)`).

## ✅ Critères d'Acceptation

- [ ] Une macro de 1000+ actions garde un frame time < 8 ms (120 FPS) au scroll.
- [ ] Le Drag & Drop et toutes les actions de carte restent opérationnels.
- [ ] La sélection et le saut vers une action (`jump_index`) fonctionnent toujours.

