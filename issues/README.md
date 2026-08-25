# 🚀 MacroForge — Issues Actives

## 🎯 Contexte
La migration vers l'application **100% Native Windows** (Rust / `egui` / `eframe`) est terminée.
Les issues de migration (#01 à #09 de la première vague) sont closes ou supprimées.
Ne restent ici que les **optimisations et correctifs encore à réaliser**.

---

## 📊 Issues Restantes

| Fiche Locale | Titre | Priorité |
|---|---|:---:|
| [`ISSUE-10`](./ISSUE-10-virtualisation-liste-actions-ui.md) | Virtualisation de la Liste d'Actions (ScrollArea) | 🟠 Haute |
| [`ISSUE-11`](./ISSUE-11-capture-ecran-dxgi-reutilisation-buffer.md) | Optimisation Capture Écran (Buffer Réutilisé / DXGI) | 🟠 Haute |
| [`ISSUE-12`](./ISSUE-12-timer-windows-granularite-sleep.md) | Granularité Timer Windows (timeBeginPeriod) | 🟠 Haute |
| [`ISSUE-13`](./ISSUE-13-logs-asynchrones-gated-debug.md) | Logs Asynchrones / Gating Debug | 🟡 Moyenne |
| [`ISSUE-15`](./ISSUE-15-cache-filtre-recherche-frame.md) | Cache du Filtrage & Recherche (par frame) | 🟡 Moyenne |

---

## 🔒 Rappel des Principes Non Négociables
- **Arrêt d'Urgence F4** : doit rester instantané en toutes circonstances.
- **Raccourcis Globaux F8 / F9** : enregistrement toujours réactif via `rdev`.
- **Thread Safety** : minimisation du temps de rétention du mutex `MACRO_STATE`.
- **Zéro Régression** : aucune dégradation de la précision temporelle ni du support Raw Input.
