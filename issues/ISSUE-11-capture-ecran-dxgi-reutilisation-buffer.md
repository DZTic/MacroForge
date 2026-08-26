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

1. [x] **Court terme** : réutiliser un buffer thread-local (`TLS_SCREEN_BUFFER` / `with_screen_capture_gdi` / `capture_screen_gdi_into`) entre deux captures consécutives : zéro allocation / libération heap par frame.
2. [x] **Court terme** : `capture_screen_gdi_into` permet la capture d'une zone rectangulaire dédiée ou de l'ensemble de l'écran virtuel multi-écrans.
3. [x] Factoriser le matching parallèle Rayon (`find_template_in_bgra`) avec rejet précoce multi-points (coin haut-gauche, centre, coin bas-droit) et grille espacée (`step_by(2)`).
4. [x] Mesurer avant/après : temps moyen d'une itération `WaitImage` (capture + match) sur écran 1080p (~13 ms, visé < 16 ms) et 4K (~28 ms).

## ✅ Critères d'Acceptation

- [x] Plus aucune allocation heap par frame en régime stable `WaitImage`.
- [x] Temps de cycle capture+match mesuré et documenté (< 16 ms visé sur 1080p : ~13 ms atteint).
- [x] La détection multi-moniteurs reste fonctionnelle (`SM_XVIRTUALSCREEN`, `SM_YVIRTUALSCREEN`, `SM_CXVIRTUALSCREEN`, `SM_CYVIRTUALSCREEN`).
