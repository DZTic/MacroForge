# 🧩 Issue #21 : Allocations de Chaînes et Layout Inconditionnel dans `GlassButton` et `ActionCard`

- **Statut** : ✅ Résolu (intégré)
- **Priorité** : 🟡 Moyenne
- **Composants** : `ui/widgets/glass_button.rs`, `ui/widgets/action_card.rs`
- **Agents Référents** : `.agents/agents/frontend-ui.md`

---

## 🎯 Description du Problème

Dans `GlassButton::ui()` (`src/ui/widgets/glass_button.rs`) :

```rust
// 1. Allocation systématique d'un String
let mut full_text = String::new();
if let Some(icon) = self.icon {
    full_text.push_str(icon);
    if !self.text.is_empty() {
        full_text.push(' ');
    }
}
full_text.push_str(self.text);

// 2. Calcul du layout de texte et de raccourci AVANT le test de visibilité
let galley = ui.painter().layout_no_wrap(full_text.clone(), font_id.clone(), colors::TEXT_PRIMARY);
let shortcut_galley = self.shortcut.map(|sc| {
    ui.painter().layout_no_wrap(sc.to_string(), shortcut_font_id.clone(), colors::TEXT_PRIMARY)
});

let (rect, response) = ui.allocate_exact_size(...);
if ui.is_rect_visible(rect) { ... }
```

Et dans `ActionCard::show()` :
Chaque carte visible formate 4 chaînes différentes (`format!("{} (VK: {:#04X})", name, vk)`, `format!("#{:03}", self.index + 1)`, etc.) par frame.

### Impact Performance
1. Même quand un bouton n'a pas d'icône, un `String` est alloué puis cloné vers `layout_no_wrap`.
2. Le layout de texte (mesure de glyphes, positionnement) est calculé avant de savoir si le widget est visible ou culled.
3. Des centaines d'allocations de petites chaînes par frame pour des boutons fixes dans la toolbar et le header.

---

## 📋 Tâches Techniques

1. Dans `GlassButton`, si `self.icon.is_none()`, passer directement `self.text` (ou `&str`) sans instancier de `String`.
2. Utiliser `std::borrow::Cow<'a, str>` ou formater dans un buffer local uniquement si l'icône est présente.
3. Ne pas allouer `sc.to_string()` pour les raccourcis : passer directement `sc`.
4. Dans `ActionCard`, utiliser un buffer réutilisable ou simplifier les `format!` avec des conversions directes sans allocation pour l'index et les libellés statiques.

---

## ✅ Critères d'Acceptation

- [ ] Zéro allocation de `String` pour les `GlassButton` sans icône et sans raccourci.
- [ ] Suppression des `full_text.clone()` et `sc.to_string()`.
- [ ] Rendu visuel et positionnement strictement identiques.
