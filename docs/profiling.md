# Profilage CPU & benchmarks — MacroForge

> Issue #58 — Infrastructure de mesure : profiler et benchmarker **avant** toute
> nouvelle optimisation, pour passer d'impressions subjectives à des métriques
> exploitables et comparables entre révisions.

## Sommaire

1. [Benchmarks criterion](#1-benchmarks-criterion)
2. [Comparaison entre révisions (baselines / critcmp)](#2-comparaison-entre-révisions)
3. [Profilage CPU sous Windows (samply)](#3-profilage-cpu-sous-windows)
4. [Alternatives de profilage](#4-alternatives-de-profilage)
5. [Méthodologie recommandée](#5-méthodologie-recommandée)

---

## 1. Benchmarks criterion

Les benchmarks vivent dans `benches/macro_core_bench.rs` et couvrent les trois
chemins critiques du moteur (reproduisent les scénarios réels de `WaitImage` /
`ImageCondition` et de la modale de test d'image) :

| Groupe criterion | Ce qui est mesuré |
|---|---|
| `find_template_in_bgra` | Matching sur écrans synthétiques 1080p et 4K : hit au centre, hit au dernier pixel, pire cas (template absent), tolérance 25 et 90 |
| `downscale_template` | Sous-échantillonnage Triangle (filtre bilinéaire de la crate `image`) d'un template 2560×1440, facteurs 2 et 4 |
| `bgra_to_rgba` | Conversion BGRA (capture GDI) → RGBA d'un écran complet 1080p / 4K |

Le profil `bench` de Cargo hérite de `release` (opt-level 3) : les mesures sont
représentatives du binaire distribué.

### Lancement

```bash
cargo bench                          # tous les groupes
cargo bench find_template_in_bgra    # filtrer par nom de groupe
cargo bench -- --quick               # une itération par bench (test de fumée)
```

Chaque exécution stocke ses mesures dans `target/criterion/` et génère un
rapport HTML consultable : `target/criterion/report/index.html`.

### Test de non-régression manuel

Le seuil grossier « 1080p < 50 ms » du matching est conservé sous forme de test
ignoré (les mesures fines 1080p/4K vivent dans criterion) :

```bash
cargo test --bin macroforge -- --ignored test_find_template_regression_1080p
```

### Architecture : pourquoi `#[path]` et pas une cible lib

Le crate est volontairement **binary-only** (convention `cargo test --bin
macroforge`, aucun target lib). Un benchmark criterion doit pouvoir lier le
code : le fichier de bench inclut donc directement les modules sources via
`#[path = "../src/..."]`. Conséquence pratique : si un module de `src/` gagne
une dépendance vers un autre module racine (`crate::app`, etc.), il faut
l'ajouter à l'inclusion dans `benches/macro_core_bench.rs`.

`criterion` est une dev-dependency : rien de tout cela n'est lié dans le
binaire release distribué.

## 2. Comparaison entre révisions

Le cœur de la démarche : mesurer **avant** un changement, puis **après**, sur
la même machine.

### Baselines criterion (sans outil supplémentaire)

```bash
# sur main, avant le changement :
cargo bench -- --save-baseline avant
# ... appliquer l'optimisation ...
cargo bench -- --baseline avant
```

Criterion affiche alors pour chaque bench la variation (`Change: -12%`,
`p < 0.05`, etc.) et signale les régressions.

### critcmp (comparaison ponctuelle entre branches)

```bash
cargo install critcmp
cargo bench -- --save-baseline main    # sur main
git checkout feat/mon-optimisation
cargo bench -- --save-baseline branche
critcmp main branche                   # tableau comparatif unifié
```

## 3. Profilage CPU sous Windows

`cargo-flamegraph` s'appuie sur `perf` (Linux uniquement) : **inutilisable sous
Windows**. L'outil recommandé ici est **samply** — échantillonneur ETW natif
Windows, flamegraph interactif, symboles Rust résolus (PDB).

### Installation

```bash
cargo install samply
```

### Build avec symboles

Le profil `release` strip les symboles (`strip = true`), ce qui rend un
flamegraph illisible. Le profil dédié `profiling` (défini dans `Cargo.toml`)
hérite de `release` (LTO fat, opt-level 3 — donc représentatif du binaire
distribué) en conservant les symboles :

```bash
cargo build --profile profiling
# binaire : target/profiling/macroforge.exe
```

### Enregistrement

```bash
samply record target/profiling/macroforge.exe
```

1. samply lance l'application et enregistre via ETW.
2. Reproduire le scénario à mesurer (ex. lecture d'une macro avec un nœud
   `WaitImage`, ou enregistrement Raw Input).
3. Fermer l'application (samply termine et sauvegarde la trace), ou interrompre
   avec `Ctrl+C`.
4. samply affiche une URL locale à ouvrir dans le navigateur : viewer
   flamegraph interactif. Refaire afficher une trace sauvegardée :
   `samply load <fichier>`.

Si l'enregistrement ETW est refusé, relancer samply depuis un terminal
**administrateur**.

Pour un flamegraph statique : `samply record --save-flamegraph flamegraph.svg
target/profiling/macroforge.exe`.

## 4. Alternatives de profilage

- **WPR / WPA** (Windows Performance Recorder / Analyzer, fournis avec le
  Windows ADK) : traces système larges (CPU, GPU, disque), analyse offline
  dans WPA. Plus lourd à interpréter que samply, mais inclus dans Windows et
  observe tout le système, pas seulement le process.
- **Superluminal** (commercial) : très faible surcoût, timeline
  thread/lock/combinée — excellent pour les moteurs multithreadés (rayon).
- **Visual Studio Diagnostic Tools** (édition Community) : profileur CPU
  intégré, sampling ou instrumentation, symboles PDB.

## 5. Méthodologie recommandée

1. **Reproduire d'abord le coût en bench** : un chemin chaud identifié dans le
   code doit avoir un bench criterion qui le mesure ; sinon l'ajouter.
2. **Sauvegarder une baseline avant** (`--save-baseline avant`).
3. **Profiler avec samply** pour localiser le coût (fonction, ligne), pas le
   deviner.
4. **Optimiser, puis `cargo bench -- --baseline avant`** : une amélioration
   qui n'est pas mesurable au bench n'en est pas une.
5. **Vérifier les gates inchangés** : `cargo fmt`, `cargo clippy --all-targets
   --all-features -- -D warnings`, `cargo test --bin macroforge`.
6. Un changement à la fois ; en cas de gain ambigu, augmenter la taille
   d'échantillon (`cargo bench -- --sample-size 100`) et refaire la mesure.

---

*Suivi de la campagne de performance : issues GitHub #58 à #64 (vague 2) et
`issues/README.md` (vague 1, résolue).*
