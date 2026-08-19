# 📌 Issue #02 : Thème & Design System Natif (Dark UI / Glassmorphism)

- **Statut** : 📝 À faire
- **Priorité** : 🟠 Haute
- **Composants** : UI Rust, egui Styles, Widgets, Icons
- **Agent Référent** : `.agents/agents/frontend-ui.md`
- **Dépendances** : Issue #01

---

## 🎯 Description du Besoin
MacroForge possède actuellement une identité visuelle moderne (Glassmorphism, tons sombres bleutés, accents lumineux, animations et badges d'état).
Cette identité doit être transposée dans le moteur de rendu immédiat `egui` via des styles, palettes de couleurs et widgets graphiques personnalisés.

---

## 📋 Tâches Techniques

1. **Palette de Couleurs & Thème Sombre Personnalisé** :
   - Définir la palette dans `egui::Visuals` :
     - Fond d'application : `#0d1117` / `#161b22` (Dark Slate).
     - Surface des cartes / widgets : `rgba(30, 41, 59, 0.7)` avec bordures subtiles (`#30363d`).
     - Accent primaire : `#3b82f6` (Bleu lumineux).
     - Succès / Play : `#10b981` (Vert émeraude).
     - Danger / Record : `#ef4444` (Rouge vibrant).
     - Attention / Pause : `#f59e0b` (Ambre).
   - Configurer les arrondis de bordures (`Rounding::same(8.0)`), marges intérieures (`Frame::inner_margin`) et ombres portées (`epaint::Shadow`).

2. **Typographie & Polices Intégrées** :
   - Charger des polices modernes (Inter ou Segoe UI Variable) via `egui::FontDefinitions`.
   - Gérer des échelles typographiques claires (Titres H1/H2, labels, sous-titres, texte de timeline en monospace).

3. **Composants & Widgets Graphiques Personnalisés** :
   - **`ActionCard`** : Carte représentant une action dans la timeline avec icône de type, titre, détails de paramètres et badge de délai (ms).
   - **`StatusBadge`** : Badge d'état avec puce lumineuse (Enregistrement en cours, Lecture, En attente, Inactif).
   - **`GlassButton`** : Bouton stylisé avec état hover/active, gradient et icône vectorielle ou glyphe.
   - **`CustomToggleSwitch`** : Interrupteur à bascule fluide pour les options (Boucler, Mouvements souris).
   - **`NumericInputWithSlider`** : Champ d'édition numérique de précision pour les délais et coordonnées.

4. **Système de Tooltips Natifs** :
   - Intégrer les tooltips d'aide contextuelle sur chaque bouton et contrôle via `.on_hover_text(...)` et `.on_hover_ui(...)`.

---

## ✅ Critères d'Acceptation
- [ ] L'application native reproduit le rendu visuel Dark / Glassmorphism de la version actuelle.
- [ ] Tous les contrôles ont des retours visuels interactifs (hover, click, active).
- [ ] Le texte et les icônes sont nets, parfaitement alignés et lisibles sur toutes les résolutions DPI (support HiDPI Windows natif).
