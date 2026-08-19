# Directives Antigravity pour MacroForge

Ce fichier définit les règles et directives d'ingénierie globale pour l'espace de travail **MacroForge**.

## 1. Organisation des Agents Spécialisés

Ce projet utilise le système de **Custom Agents Antigravity** situés dans [`.agents/agents/`](./.agents/agents/).
Lors de la résolution de tâches complexes, privilégier la division du travail :
- **Backend / Tauri / Win32** : Se référer à [`.agents/agents/rust-core.md`](./.agents/agents/rust-core.md).
- **Interface Utilisateur / CSS / TypeScript** : Se référer à [`.agents/agents/frontend-ui.md`](./.agents/agents/frontend-ui.md).
- **Vision par Ordinateur / Pattern Matching / Entrées FPS** : Se référer à [`.agents/agents/vision-automation.md`](./.agents/agents/vision-automation.md).
- **Contrôle Qualité / Sécurité Windows / Cas Limites** : Se référer à [`.agents/agents/qa-security.md`](./.agents/agents/qa-security.md).

## 2. Principes d'Implémentation

- **Simplicité et Précision** : Ne pas ajouter de complexité ou de code spéculatif inutile. Toucher uniquement aux fichiers nécessaires.
- **Sécurité des Entrées Windows** : Ne jamais désactiver ou contourner les mécanismes d'interruption d'urgence (touche **F4** pour l'arrêt de la relecture, touche **F9** pour l'arrêt de l'enregistrement).
- **Thread Safety** : Maintenir un verrouillage minimal du mutex `MACRO_STATE` sous Rust pour préserver la réactivité en temps réel.
- **Internationalisation** : Toute modification apportée aux textes de l'interface doit être disponible en Français et en Anglais.
