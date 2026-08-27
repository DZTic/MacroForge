# 🖥️ Issue #22 : Capture Plein Écran Virtuel Multi-Écrans en `WaitImage` et `StopImage`

- **Statut** : ✅ Résolu (intégré)
- **Priorité** : 🟡 Moyenne
- **Composants** : `macro_core.rs`, `check_image_present()`, `WaitImage`, Win32 GDI
- **Agents Référents** : `.agents/agents/vision-automation.md`, `.agents/agents/rust-core.md`

---

## 🎯 Description du Problème

Dans `WaitImage` et `check_image_present()`, la capture d'écran est effectuée sur l'ensemble de l'écran virtuel Windows :

```rust
let vx = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
let vy = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };
let vw = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
let vh = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };

with_screen_capture_gdi(vx, vy, vw, vh, |screen_raw| {
    find_template_in_bgra(...)
})
```

### Impact Performance
1. Sur des configurations multi-écrans (ex. 2 écrans 4K ou 3 écrans 1440p), la taille de l'écran virtuel atteint 7680x2160 pixels ou plus.
2. Un seul buffer de capture représente **66 Mo à 100 Mo** de données de pixels.
3. Même si le buffer est réutilisé via thread-local, la fonction GDI `BitBlt` et `GetDIBits` doit transférer des dizaines de mégaoctets de mémoire vidéo vers la mémoire système à chaque cycle (toutes les 33 ms pour `WaitImage` ou 3s pour l'arrêt).
4. Le template recherché ne se trouve généralement que sur la fenêtre de jeu ou sur l'écran principal.

---

## 📋 Tâches Techniques

1. Cibler en priorité le rectangle de la fenêtre de jeu active (`LAST_GAME_HWND`) via `GetWindowRect` au lieu du bureau virtuel entier.
2. Si la fenêtre n'est pas connue ou si l'utilisateur demande une recherche globale, restreindre au moniteur principal (`SM_CXSCREEN`, `SM_CYSCREEN`) ou au moniteur contenant la fenêtre active.
3. Conserver l'option de capture multi-écrans en fallback.

---

## ✅ Critères d'Acceptation

- [ ] La capture d'image lors d'un `WaitImage` cible la région active (taille réduite de 50% à 75% par rapport à l'écran virtuel complet).
- [ ] Le temps de capture BitBlt passe sous la barre des 5 ms sur setup multi-écrans.
- [ ] La détection d'image reste fiable et exacte.
