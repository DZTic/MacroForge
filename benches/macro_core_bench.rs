//! Benchmarks criterion des chemins critiques du moteur (issue #58).
//!
//! Le crate MacroForge est **binary-only** (aucune cible lib, convention
//! `cargo test --bin macroforge`) : les sources sont donc incluses ici via
//! `#[path]` pour rester liables. Les références `crate::` de macro_core
//! (blueprint, events, ui::i18n) se résolvent car ces modules sont déclarés
//! à la racine de ce crate de bench.
//!
//! Exécution : `cargo bench` — guide complet (baselines, samply) :
//! `docs/profiling.md`.

// Cargo active `cfg(test)` pour compiler les cibles bench : les modules
// `#[cfg(test)] mod tests` des sources incluses compilent donc ici comme du
// code mort (aucun harnais libtest ne les collecte). Ces allows sont
// ciblés sur les modules inclus — pas sur le code du bench lui-même.
#[path = "../src/macro_core.rs"]
#[allow(dead_code, unused_imports)]
pub mod macro_core;

#[path = "../src/events.rs"]
#[allow(dead_code, unused_imports)]
pub mod events;

// Seul `crate::ui::i18n::Language` est requis (blueprint/graph.rs et
// runner.rs) : on n'inclut pas toute l'arborescence UI (widgets, modales).
// NB : à l'intérieur d'un module inline, #[path] est relatif au répertoire
// virtuel `benches/ui/`, d'où le `../../`.
pub mod ui {
    #[path = "../../src/ui/i18n.rs"]
    #[allow(dead_code, unused_imports)]
    pub mod i18n;
}

#[path = "../src/blueprint/mod.rs"]
#[allow(dead_code, unused_imports)]
pub mod blueprint;

use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode};
use macro_core::{bgra_to_rgba, downscale_template, find_template_in_bgra};

/// Tailles d'écran couvertes (pixels BGRA).
struct ScreenSpec {
    name: &'static str,
    width: usize,
    height: usize,
}

const SCREENS: &[ScreenSpec] = &[
    ScreenSpec {
        name: "1080p",
        width: 1920,
        height: 1080,
    },
    ScreenSpec {
        name: "4K",
        width: 3840,
        height: 2160,
    },
];

/// Écran BGRA de fond uniforme 30 avec le template 32x32 (couleur
/// 200/100/50 en RGBA) estampé à (target_x, target_y) — mêmes valeurs que
/// le test de non-régression manuel, reproduisant un nœud de jeu sur fond
/// d'écran. Retourne (écran, template).
fn make_screen_with_hit(
    width: usize,
    height: usize,
    tw: usize,
    th: usize,
    target_x: usize,
    target_y: usize,
) -> (Vec<u8>, Vec<u8>) {
    let mut screen = vec![30u8; width * height * 4];
    let mut template = vec![0u8; tw * th * 4];
    for ty in 0..th {
        for tx in 0..tw {
            let t_idx = (ty * tw + tx) * 4;
            template[t_idx] = 200;
            template[t_idx + 1] = 100;
            template[t_idx + 2] = 50;
            template[t_idx + 3] = 255;

            let s_idx = ((target_y + ty) * width + (target_x + tx)) * 4;
            screen[s_idx] = 50;
            screen[s_idx + 1] = 100;
            screen[s_idx + 2] = 200;
            screen[s_idx + 3] = 255;
        }
    }
    (screen, template)
}

/// Écran BGRA de fond uniforme 30, template absent — pire cas : balayage
/// complet de l'écran sans hit possible.
fn make_screen_without_hit(
    width: usize,
    height: usize,
    tw: usize,
    th: usize,
) -> (Vec<u8>, Vec<u8>) {
    let screen = vec![30u8; width * height * 4];
    let mut template = vec![0u8; tw * th * 4];
    for i in 0..(tw * th) {
        template[i * 4] = 200;
        template[i * 4 + 1] = 100;
        template[i * 4 + 2] = 50;
        template[i * 4 + 3] = 255;
    }
    (screen, template)
}

fn bench_find_template(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_template_in_bgra");
    // Les pires cas 4K coûtent des dizaines de ms par itération : échantillonnage
    // plat et temps de mesure réduits pour garder `cargo bench` exécutable.
    group.sampling_mode(SamplingMode::Flat);
    group.measurement_time(Duration::from_secs(10));
    group.warm_up_time(Duration::from_secs(1));

    for spec in SCREENS {
        let tw = 32;
        let th = 32;

        // Cas 1 : hit immédiat — template dans les premières lignes scannées.
        let (screen, template) = make_screen_with_hit(spec.width, spec.height, tw, th, 16, 16);
        group.bench_function(BenchmarkId::new("hit_immediate", spec.name), |b| {
            b.iter(|| {
                black_box(find_template_in_bgra(
                    black_box(&screen),
                    spec.width,
                    spec.height,
                    black_box(&template),
                    tw,
                    th,
                    25,
                ))
            })
        });

        // Cas 2 : hit au dernier pixel testable (coin bas-droit) — balayage
        // complet avant de trouver.
        let (screen, template) = make_screen_with_hit(
            spec.width,
            spec.height,
            tw,
            th,
            spec.width - tw,
            spec.height - th,
        );
        group.bench_function(BenchmarkId::new("hit_last_pixel", spec.name), |b| {
            b.iter(|| {
                black_box(find_template_in_bgra(
                    black_box(&screen),
                    spec.width,
                    spec.height,
                    black_box(&template),
                    tw,
                    th,
                    25,
                ))
            })
        });

        // Cas 3 : pire cas, template absent — sans tolérance (exact) ...
        let (screen, template) = make_screen_without_hit(spec.width, spec.height, tw, th);
        group.bench_function(
            BenchmarkId::new("worst_no_hit_tolerance_0", spec.name),
            |b| {
                b.iter(|| {
                    black_box(find_template_in_bgra(
                        black_box(&screen),
                        spec.width,
                        spec.height,
                        black_box(&template),
                        tw,
                        th,
                        0,
                    ))
                })
            },
        );

        // ... et avec la tolérance par défaut de l'application (25) : la
        // comparaison entre les deux mesure le coût propre de la tolérance.
        group.bench_function(
            BenchmarkId::new("worst_no_hit_tolerance_25", spec.name),
            |b| {
                b.iter(|| {
                    black_box(find_template_in_bgra(
                        black_box(&screen),
                        spec.width,
                        spec.height,
                        black_box(&template),
                        tw,
                        th,
                        25,
                    ))
                })
            },
        );
    }
    group.finish();
}

fn bench_downscale_template(c: &mut Criterion) {
    let mut group = c.benchmark_group("downscale_template");
    // Template « screenshot » réaliste 2560x1440 (cas de test_image_search :
    // screenshot pris sur un écran plus grand que la zone de recherche),
    // sous-échantillonné facteur 2 puis 4 (filtre Triangle de la crate image).
    let (tw, th) = (2560u32, 1440u32);
    let raw = vec![128u8; (tw as usize) * (th as usize) * 4];
    for factor in [2u32, 4u32] {
        group.bench_function(
            BenchmarkId::new("2560x1440", format!("factor_{factor}")),
            |b| {
                b.iter(|| {
                    black_box(downscale_template(
                        black_box(&raw),
                        black_box(tw),
                        black_box(th),
                        black_box(factor),
                    ))
                })
            },
        );
    }
    group.finish();
}

fn bench_bgra_to_rgba(c: &mut Criterion) {
    let mut group = c.benchmark_group("bgra_to_rgba");
    // Conversion d'une capture GDI plein écran (chemin de
    // bgra_capture_to_egui_texture / cache BGRA du moteur).
    for spec in SCREENS {
        let bgra = vec![30u8; spec.width * spec.height * 4];
        group.bench_function(BenchmarkId::new("full_screen", spec.name), |b| {
            b.iter(|| black_box(bgra_to_rgba(black_box(&bgra))))
        });
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = bench_find_template, bench_downscale_template, bench_bgra_to_rgba
}
criterion_main!(benches);
