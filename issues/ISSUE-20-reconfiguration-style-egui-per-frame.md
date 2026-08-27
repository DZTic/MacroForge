# 🎨 Issue #20 : Reconfiguration Inconditionnelle de `egui::Style` et `apply_visuals` à Chaque Frame

- **Statut** : ✅ Résolu (intégré)
- **Priorité** : 🟡 Moyenne
- **Composants** : `app.rs`, `theme.rs`, cycle de rendu egui
- **Agents Référents** : `.agents/agents/frontend-ui.md`, `.agents/agents/rust-core.md`

---

## 🎯 Description du Problème

Dans `MacroForgeApp::update()`, la première ligne exécutée est :

```rust
// Garantir le maintien du thème sombre Dark Glassmorphism face aux variations du thème Windows
theme::apply_visuals(ctx);
```

Et dans `theme::apply_visuals()` :

```rust
pub fn apply_visuals(ctx: &Context) {
    let mut visuals = Visuals::dark();
    // ... assignation des champs ...
    let mut style = (*ctx.style()).clone(); // ⚠️ Clone complet de egui::Style à chaque frame !
    style.visuals = visuals;
    style.text_styles = [ ... ].into();     // ⚠️ Réallocation des styles typographiques
    ctx.set_style(style);                   // ⚠️ Propagation de changement de style
}
```

### Impact Performance
1. `ctx.style()` est une structure complexe. La cloner 60 à 120 fois par seconde alloue inutilement des structures internes.
2. `ctx.set_style()` réinitialise certains caches de métriques de polices et force des recalculs internes dans egui.
3. Le style ne change quasiment jamais en cours d'exécution.

---

## 📋 Tâches Techniques

1. Appliquer le `Style` complet et les polices **une seule fois** à l'initialisation dans `MacroForgeApp::new()` via `theme::apply_theme(cc)`.
2. Si une synchronisation contre le thème système est nécessaire, ne réappliquer `apply_visuals()` que si `ctx.style().visuals.dark_mode == false` ou via un callback d'événement dédié.
3. Éliminer le clone de `Style` par frame dans la boucle `update()`.

---

## ✅ Critères d'Acceptation

- [ ] `ctx.set_style()` n'est plus appelé à chaque frame dans `update()`.
- [ ] Le thème sombre Dark Glassmorphism reste intact.
- [ ] Diminution mesurable du temps de passe UI (frame time).
