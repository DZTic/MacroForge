# 🔍 Issue #15 : Cache du Filtrage & Recherche (éviter recalcul par frame)

- **Statut** : 📝 À faire
- **Priorité** : 🟡 Moyenne
- **Composants** : `app.rs`, matches_filter(), filtered_indices, Allocations
- **Agents Référents** : `.agents/agents/rust-core.md`

---

## 🎯 Description du Problème

`filtered_indices` est reconstruit a **chaque frame** egui, et `matches_filter()` effectue des allocations par action :

```rust
let q = self.search_query.trim().to_lowercase();       // allocation/frame
format!("{} {}", x, y).contains(&q)                    // alloc/action/frame
name.to_lowercase().contains(&q)                       // alloc/action/frame
```

Pour 1000 actions a 60 FPS : environ 180 000 petites allocations/seconde rien que pour la barre de filtre.

## 📋 Tâches Techniques

1. Mettre en cache `filtered_indices: Vec<usize>` dans `MacroForgeApp`.
2. Recalculer uniquement quand (`search_query` | `hide_mouse_moves` | version des actions) change — utiliser un compteur de version incrémenté a chaque mutation de la liste.
3. Pré-calculer le `to_lowercase` de la requête une fois par changement de saisie (pas par frame).
4. Remplacer les `format!` par comparaisons sans allocation (match sur discriminant + comparaison numérique directe).
5. Optionnel : précalculer les chaînes de recherche par action lors de l'ajout/modification.

## ✅ Critères d'Acceptation

- [ ] Aucun recalcul du filtre quand ni la requête ni la liste ne changent.
- [ ] Frame time UI inchangé avec une macro de 1000 actions + filtre actif.
- [ ] Résultats de recherche identiques au comportement actuel (pas de régression fonctionnelle).

