# 🖼️ Issue #11 : Optimisation Capture Écran (Buffer Réutilisé / DXGI)

- **Statut** : 📝 À faire
- **Priorité** : 🟠 Haute
- **Composants** : `macro_core.rs`, GDI BitBlt, rayon, WaitImage
- **Agents Référents** : `.agents/agents/rust-core.md`

---

## 🎯 Description du Problème

`capture_screen_gdi()` alloue un nouveau `Vec<u8>` de taille écran x 4 octets **à chaque appel**, et capture **tout l'écran virtuel** (multi-moniteurs). En mode `WaitImage` avec polling a environ 30 fps sur un setup 4K :

- ~33 Mo alloués/libérés 30 fois/seconde, soit environ **1 Go/s de churn mémoire**
- Pression allocator inutile, latence de recherche image dégradée

Le matching parallèle rayon est efficace, mais il travaille sur une capture pleine page alors qu'une région plus petite suffirait souvent.

## 📋 Tâches Techniques

1. **Court terme** : réutiliser un buffer thread-local ou stocké dans `MACRO_STATE` entre deux captures consécutives (même résolution).
2. **Court terme** : permettre la restriction de la capture à une zone de recherche configurable au lieu du plein écran systématique.
3. **Moyen terme** : migrer vers **DXGI Desktop Duplication API** pour une capture GPU-accelérée sans blocage du pipeline GDI (gain typique : x2 a x5 sur la vitesse de capture).
4. Ajouter un early-exit ligne par ligne dans la boucle de matching (skip si aucun pixel de la première colonne du template ne matche).
5. Mesurer avant/après : temps moyen d'une itération `WaitImage` (capture + match) sur écran 1080p et 4K.

## ✅ Critères d'Acceptation

- [ ] Plus aucune allocation heap par frame en régime stable `WaitImage`.
- [ ] Temps de cycle capture+match mesuré et documenté (< 16 ms visé sur 1080p).
- [ ] La détection multi-moniteurs reste fonctionnelle.

