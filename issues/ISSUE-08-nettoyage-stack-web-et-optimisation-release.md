# 📌 Issue #08 : Nettoyage Stack Web & Optimisations Release Finales

- **Statut** : 📝 À faire
- **Priorité** : 🟢 Finale
- **Composants** : Workspace cleanup, Cargo Profiles, Binary Size, Benchmarks
- **Agents Référents** : `.agents/agents/rust-core.md`, `.agents/agents/qa-security.md`
- **Dépendances** : Issue #01 à Issue #07

---

## 🎯 Description du Besoin
Une fois la nouvelle interface native en Rust (`egui`/`eframe`) complètement fonctionnelle, il s'agit de purger tous les fichiers résiduels de la stack Web (Tauri, Node.js, Vite, TypeScript, HTML, CSS) et de configurer le profil de compilation Release pour une taille d'exécutable et une vitesse d'exécution maximales.

---

## 📋 Tâches Techniques

1. **Suppression Définitive de la Stack Web & Fichiers Obsolètes** :
   - Supprimer le dossier `node_modules/` et `dist/`.
   - Supprimer les fichiers de configuration Web : `package.json`, `package-lock.json`, `tsconfig.json`, `vite.config.ts`.
   - Supprimer les pages HTML et assets web : `index.html`, `toolbar.html`, `overlay.html`, `src/`.
   - Restructurer le projet pour placer le code Rust directement dans `src/` (ou un workspace Cargo racine propre au lieu du sous-dossier `src-tauri/`).

2. **Configuration du Profil `[profile.release]` dans `Cargo.toml`** :
   - Activer les optimisations de pointe du compilateur Rust :
     ```toml
     [profile.release]
     opt-level = 3
     lto = "fat"
     codegen-units = 1
     panic = "abort"
     strip = true
     ```

3. **Validation des Métriques de Performance (Benchmarks)** :
   - **Consommation Mémoire RAM** : Vérifier que l'empreinte mémoire reste sous les **20-25 Mo** en pleine relecture et détection d'image.
   - **Temps de Démarrage** : Valider un démarrage à froid en **< 50 ms**.
   - **Taille de l'Exécutable** : Obtenir un binaire unique `.exe` de **< 15 Mo**.
   - **Stabilité de l'Arrêt d'Urgence F4** : Tester des relectures intensives de 10 000+ actions avec arrêt immédiat.

4. **Mise à Jour de la Documentation (`README.md`, `GEMINI.md`, `CLAUDE.md`)** :
   - Actualiser les instructions de build : simple `cargo build --release`.
   - Mettre à jour la description technique pour refléter l'architecture 100% Rust / Win32.

---

## ✅ Critères d'Acceptation
- [ ] Plus aucun fichier lié à Node.js, Vite ou WebView2 n'est présent dans le dépôt.
- [ ] La compilation s'effectue uniquement avec la toolchain Rust standard (`cargo build --release`).
- [ ] Le binaire produit est 100% autonome, ultra-rapide et stable.
