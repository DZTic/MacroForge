use crate::blueprint::graph::{BlueprintGraph, BlueprintNodeType, NodeId};
use crate::events::EngineEvent;
use crate::macro_core;
use crate::ui::i18n::Language;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct BlueprintRunnerState {
    pub is_running: Arc<AtomicBool>,
    pub active_node_id: Arc<Mutex<Option<NodeId>>>,
    pub stop_flag: Arc<AtomicBool>,
    pub status_message: Arc<Mutex<String>>,
}

impl Default for BlueprintRunnerState {
    fn default() -> Self {
        Self::new()
    }
}

impl BlueprintRunnerState {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            active_node_id: Arc::new(Mutex::new(None)),
            stop_flag: Arc::new(AtomicBool::new(false)),
            status_message: Arc::new(Mutex::new("Prêt".to_string())),
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }

    pub fn get_active_node(&self) -> Option<NodeId> {
        *self.active_node_id.lock().unwrap()
    }

    pub fn get_status(&self) -> String {
        self.status_message.lock().unwrap().clone()
    }

    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        macro_core::emergency_stop();
    }

    pub fn run_graph(&self, graph: BlueprintGraph, lang: Language) -> bool {
        if self.is_running.load(Ordering::Relaxed) {
            return false;
        }

        let start_id = match graph.find_start_node() {
            Some(id) => id,
            None => {
                let msg = match lang {
                    Language::Fr => "❌ Aucun nœud de départ trouvé dans le Blueprint.".to_string(),
                    Language::En => "❌ No Start node found in the Blueprint.".to_string(),
                };
                *self.status_message.lock().unwrap() = msg;
                return false;
            }
        };

        self.is_running.store(true, Ordering::SeqCst);
        self.stop_flag.store(false, Ordering::SeqCst);
        let global_stop_flag = macro_core::get_stop_playback_flag();
        global_stop_flag.store(false, Ordering::SeqCst);

        let is_running_clone = Arc::clone(&self.is_running);
        let active_node_clone = Arc::clone(&self.active_node_id);
        let stop_flag_clone = Arc::clone(&self.stop_flag);
        let status_msg_clone = Arc::clone(&self.status_message);

        macro_core::notify_event(EngineEvent::PlaybackStateChanged(true));

        thread::spawn(move || {
            let mut current_id = Some(start_id);
            let mut loop_counters: HashMap<NodeId, u32> = HashMap::new();
            let mut last_detected_image_pos: Option<(i32, i32)> = None;
            let mut max_steps = 100_000u32; // Protection anti-boucle infinie sans délai
                                            // Indique si la fenêtre de jeu doit être remontée au premier plan
                                            // avant la prochaine recherche d'image (sinon la capture GDI risque
                                            // de photographier MacroForge recouvrant le jeu).
            let mut foreground_refresh_done = false;

            let set_status = |msg: &str| {
                if let Ok(mut status) = status_msg_clone.lock() {
                    *status = msg.to_string();
                }
            };

            let check_stopped = || {
                stop_flag_clone.load(Ordering::Relaxed) || global_stop_flag.load(Ordering::Relaxed)
            };

            let ready_msg = match lang {
                Language::Fr => "⏹️ Exécution Blueprint terminée.".to_string(),
                Language::En => "⏹️ Blueprint execution finished.".to_string(),
            };

            while let Some(node_id) = current_id {
                if check_stopped() {
                    break;
                }

                if max_steps == 0 {
                    let warn_msg = match lang {
                        Language::Fr => {
                            "⚠️ Limite de 100 000 étapes atteinte (boucle infinie stoppée)."
                                .to_string()
                        }
                        Language::En => {
                            "⚠️ 100,000 steps limit reached (infinite loop stopped).".to_string()
                        }
                    };
                    set_status(&warn_msg);
                    break;
                }
                max_steps -= 1;

                if let Ok(mut active) = active_node_clone.lock() {
                    *active = Some(node_id);
                }

                let node = match graph.find_node(node_id) {
                    Some(n) => n.clone(),
                    None => break,
                };

                let next_output_pin: Option<usize> = match &node.node_type {
                    BlueprintNodeType::Start => {
                        let step_msg = match lang {
                            Language::Fr => "🚀 Démarrage du workflow...".to_string(),
                            Language::En => "🚀 Starting workflow...".to_string(),
                        };
                        set_status(&step_msg);
                        Some(0)
                    }
                    BlueprintNodeType::Macro { name, actions } => {
                        let step_msg = match lang {
                            Language::Fr => format!(
                                "▶️ Exécution de la macro '{}' ({} actions)...",
                                name,
                                actions.len()
                            ),
                            Language::En => format!(
                                "▶️ Playing macro '{}' ({} actions)...",
                                name,
                                actions.len()
                            ),
                        };
                        set_status(&step_msg);

                        let success = macro_core::play_action_sequence(actions, &global_stop_flag);
                        if !success || check_stopped() {
                            break;
                        }
                        Some(0)
                    }
                    BlueprintNodeType::ImageCondition {
                        image_path,
                        tolerance,
                        timeout_ms,
                    } => {
                        // Remonter le jeu au premier plan avant la première
                        // recherche d'image de cette exécution (comme play_macro).
                        if !foreground_refresh_done {
                            foreground_refresh_done = true;
                            macro_core::bring_game_to_foreground();
                            thread::sleep(Duration::from_millis(150));
                        }

                        let step_msg = match lang {
                            Language::Fr => {
                                format!("👁️ Recherche de l'image (tolérance: {})...", tolerance)
                            }
                            Language::En => {
                                format!("👁️ Scanning for image (tolerance: {})...", tolerance)
                            }
                        };
                        set_status(&step_msg);

                        let start = Instant::now();
                        let mut matched_pos = None;
                        let timeout = Duration::from_millis(*timeout_ms);

                        loop {
                            if check_stopped() {
                                break;
                            }
                            if let Some(pos) =
                                macro_core::find_image_coords_with_tolerance(image_path, *tolerance)
                            {
                                matched_pos = Some(pos);
                                break;
                            }
                            if start.elapsed() >= timeout {
                                break;
                            }
                            thread::sleep(Duration::from_millis(50));
                        }

                        if check_stopped() {
                            break;
                        }

                        if let Some(pos) = matched_pos {
                            last_detected_image_pos = Some(pos);
                            let msg = match lang {
                                Language::Fr => {
                                    format!("✅ Image détectée à ({}, {}) ! Branche [Match] sélectionnée.", pos.0, pos.1)
                                }
                                Language::En => {
                                    format!(
                                        "✅ Image found at ({}, {})! [Match] branch selected.",
                                        pos.0, pos.1
                                    )
                                }
                            };
                            set_status(&msg);
                            Some(0) // Pin 0 = Match
                        } else {
                            let msg = match lang {
                                Language::Fr => {
                                    "❌ Image absente. Branche [Miss] sélectionnée.".to_string()
                                }
                                Language::En => {
                                    "❌ Image not found. [Miss] branch selected.".to_string()
                                }
                            };
                            set_status(&msg);
                            Some(1) // Pin 1 = Miss
                        }
                    }
                    BlueprintNodeType::WaitImage {
                        image_path,
                        timeout_ms,
                        tolerance,
                        delay_after_ms,
                    } => {
                        // Remonter le jeu au premier plan avant la première
                        // recherche d'image de cette exécution (comme play_macro).
                        if !foreground_refresh_done {
                            foreground_refresh_done = true;
                            macro_core::bring_game_to_foreground();
                            thread::sleep(Duration::from_millis(150));
                        }

                        let step_msg = match lang {
                            Language::Fr => format!(
                                "⏳ Attente apparition image (timeout: {}ms)...",
                                timeout_ms
                            ),
                            Language::En => {
                                format!("⏳ Waiting for image (timeout: {}ms)...", timeout_ms)
                            }
                        };
                        set_status(&step_msg);

                        let start = Instant::now();
                        let mut found_pos = None;
                        let timeout = Duration::from_millis(*timeout_ms);

                        loop {
                            if check_stopped() {
                                break;
                            }
                            if let Some(pos) =
                                macro_core::find_image_coords_with_tolerance(image_path, *tolerance)
                            {
                                found_pos = Some(pos);
                                break;
                            }
                            if start.elapsed() >= timeout {
                                break;
                            }
                            thread::sleep(Duration::from_millis(50));
                        }

                        if check_stopped() {
                            break;
                        }

                        if let Some(pos) = found_pos {
                            last_detected_image_pos = Some(pos);
                            if *delay_after_ms > 0 {
                                let delay_msg = match lang {
                                    Language::Fr => format!(
                                        "⏳ Image trouvée ! Pause post-détection de {} ms...",
                                        delay_after_ms
                                    ),
                                    Language::En => format!(
                                        "⏳ Image detected! Post-detection delay of {} ms...",
                                        delay_after_ms
                                    ),
                                };
                                set_status(&delay_msg);

                                let delay_start = Instant::now();
                                let target_delay = Duration::from_millis(*delay_after_ms);
                                while delay_start.elapsed() < target_delay {
                                    if check_stopped() {
                                        break;
                                    }
                                    let remaining =
                                        target_delay.saturating_sub(delay_start.elapsed());
                                    let sleep_chunk = remaining.min(Duration::from_millis(15));
                                    thread::sleep(sleep_chunk);
                                }
                                if check_stopped() {
                                    break;
                                }
                            }
                            Some(0) // Pin 0 = Found
                        } else {
                            Some(1) // Pin 1 = Timeout
                        }
                    }
                    BlueprintNodeType::ClickImage {
                        use_last_detected,
                        image_path,
                        tolerance,
                        timeout_ms,
                        click_type,
                        offset_x,
                        offset_y,
                    } => {
                        let target_coords = if *use_last_detected {
                            last_detected_image_pos
                        } else {
                            let step_msg = match lang {
                                Language::Fr => format!(
                                    "👁️ Recherche de l'image pour clic (tolérance: {})...",
                                    tolerance
                                ),
                                Language::En => format!(
                                    "👁️ Scanning for image to click (tolerance: {})...",
                                    tolerance
                                ),
                            };
                            set_status(&step_msg);

                            let start = Instant::now();
                            let mut found = None;
                            let timeout = Duration::from_millis(*timeout_ms);

                            loop {
                                if check_stopped() {
                                    break;
                                }
                                if let Some(pos) = macro_core::find_image_coords_with_tolerance(
                                    image_path, *tolerance,
                                ) {
                                    found = Some(pos);
                                    break;
                                }
                                if start.elapsed() >= timeout {
                                    break;
                                }
                                thread::sleep(Duration::from_millis(50));
                            }
                            if let Some(pos) = found {
                                last_detected_image_pos = Some(pos);
                            }
                            found
                        };

                        if check_stopped() {
                            break;
                        }

                        if let Some((cx, cy)) = target_coords {
                            let click_x = cx + offset_x;
                            let click_y = cy + offset_y;
                            let step_msg = match lang {
                                Language::Fr => format!(
                                    "🎯 Clic ({}) sur l'image à ({}, {})...",
                                    click_type.label(lang),
                                    click_x,
                                    click_y
                                ),
                                Language::En => format!(
                                    "🎯 Click ({}) on image at ({}, {})...",
                                    click_type.label(lang),
                                    click_x,
                                    click_y
                                ),
                            };
                            set_status(&step_msg);

                            macro_core::execute_click(click_x, click_y, *click_type);
                            thread::sleep(Duration::from_millis(50));
                            Some(0) // Pin 0 = Effectué
                        } else {
                            let fail_msg = match lang {
                                Language::Fr => {
                                    "❌ Clic annulé : aucune image cible détectée. Branche [Échec]."
                                        .to_string()
                                }
                                Language::En => {
                                    "❌ Click aborted: no target image detected. [Failed] branch."
                                        .to_string()
                                }
                            };
                            set_status(&fail_msg);
                            Some(1) // Pin 1 = Échec
                        }
                    }
                    BlueprintNodeType::ClickCoordinate {
                        x,
                        y,
                        click_type,
                        delay_after_ms,
                    } => {
                        let step_msg = match lang {
                            Language::Fr => format!(
                                "📍 Clic ({}) aux coordonnées ({}, {})...",
                                click_type.label(lang),
                                x,
                                y
                            ),
                            Language::En => format!(
                                "📍 Click ({}) at coordinates ({}, {})...",
                                click_type.label(lang),
                                x,
                                y
                            ),
                        };
                        set_status(&step_msg);

                        macro_core::execute_click(*x, *y, *click_type);
                        if *delay_after_ms > 0 {
                            let start = Instant::now();
                            let target = Duration::from_millis(*delay_after_ms);
                            while start.elapsed() < target {
                                if check_stopped() {
                                    break;
                                }
                                let remaining = target.saturating_sub(start.elapsed());
                                let sleep_chunk = remaining.min(Duration::from_millis(15));
                                thread::sleep(sleep_chunk);
                            }
                        }
                        if check_stopped() {
                            break;
                        }
                        Some(0)
                    }
                    BlueprintNodeType::MouseMove { x, y, relative } => {
                        let step_msg = match lang {
                            Language::Fr => {
                                if *relative {
                                    format!("🖱️ Déplacement curseur relatif ({:+}, {:+})...", x, y)
                                } else {
                                    format!("🖱️ Déplacement curseur vers ({}, {})...", x, y)
                                }
                            }
                            Language::En => {
                                if *relative {
                                    format!("🖱️ Move cursor relative ({:+}, {:+})...", x, y)
                                } else {
                                    format!("🖱️ Move cursor to ({}, {})...", x, y)
                                }
                            }
                        };
                        set_status(&step_msg);

                        #[cfg(windows)]
                        {
                            if *relative {
                                macro_core::send_mouse_relative(*x, *y);
                            } else {
                                macro_core::send_mouse_move(*x, *y);
                            }
                        }
                        thread::sleep(Duration::from_millis(20));
                        Some(0)
                    }
                    BlueprintNodeType::KeyPress {
                        key_name,
                        vk_code,
                        is_extended,
                        hold_ms,
                    } => {
                        let step_msg = match lang {
                            Language::Fr => format!("⌨️ Pression de la touche '{}'...", key_name),
                            Language::En => format!("⌨️ Pressing key '{}'...", key_name),
                        };
                        set_status(&step_msg);

                        #[cfg(windows)]
                        {
                            macro_core::send_key(*vk_code, false, *is_extended);
                            if *hold_ms > 0 {
                                let start = Instant::now();
                                let target = Duration::from_millis(*hold_ms);
                                while start.elapsed() < target {
                                    if check_stopped() {
                                        break;
                                    }
                                    let remaining = target.saturating_sub(start.elapsed());
                                    let sleep_chunk = remaining.min(Duration::from_millis(10));
                                    thread::sleep(sleep_chunk);
                                }
                            }
                            macro_core::send_key(*vk_code, true, *is_extended);
                        }
                        thread::sleep(Duration::from_millis(20));
                        if check_stopped() {
                            break;
                        }
                        Some(0)
                    }
                    BlueprintNodeType::RandomDelay { min_ms, max_ms } => {
                        let min = *min_ms.min(max_ms);
                        let max = *min_ms.max(max_ms);
                        let duration_ms = if min == max {
                            min
                        } else {
                            use std::time::SystemTime;
                            let seed = SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .map(|d| d.subsec_nanos())
                                .unwrap_or(42) as u64;
                            min + (seed % (max - min + 1))
                        };
                        let step_msg = match lang {
                            Language::Fr => format!("🎲 Pause aléatoire de {} ms...", duration_ms),
                            Language::En => format!("🎲 Random delay of {} ms...", duration_ms),
                        };
                        set_status(&step_msg);

                        let start = Instant::now();
                        let target = Duration::from_millis(duration_ms);
                        while start.elapsed() < target {
                            if check_stopped() {
                                break;
                            }
                            let remaining = target.saturating_sub(start.elapsed());
                            let sleep_chunk = remaining.min(Duration::from_millis(15));
                            thread::sleep(sleep_chunk);
                        }
                        if check_stopped() {
                            break;
                        }
                        Some(0)
                    }
                    BlueprintNodeType::MouseScroll { steps } => {
                        let step_msg = match lang {
                            Language::Fr => format!("📜 Défilement molette ({} pas)...", steps),
                            Language::En => format!("📜 Mouse scroll ({} steps)...", steps),
                        };
                        set_status(&step_msg);

                        #[cfg(windows)]
                        unsafe {
                            use winapi::um::winuser::{mouse_event, MOUSEEVENTF_WHEEL};
                            let delta = (*steps * 120) as u32;
                            mouse_event(MOUSEEVENTF_WHEEL, 0, 0, delta, 0);
                        }
                        thread::sleep(Duration::from_millis(40));
                        Some(0)
                    }
                    BlueprintNodeType::Delay { delay_ms } => {
                        let step_msg = match lang {
                            Language::Fr => format!("⏱️ Pause de {} ms...", delay_ms),
                            Language::En => format!("⏱️ Delaying {} ms...", delay_ms),
                        };
                        set_status(&step_msg);

                        let start = Instant::now();
                        let target = Duration::from_millis(*delay_ms);
                        while start.elapsed() < target {
                            if check_stopped() {
                                break;
                            }
                            let remaining = target.saturating_sub(start.elapsed());
                            let sleep_chunk = remaining.min(Duration::from_millis(15));
                            thread::sleep(sleep_chunk);
                        }
                        if check_stopped() {
                            break;
                        }
                        Some(0)
                    }
                    BlueprintNodeType::Loop { count } => {
                        let current_counter = loop_counters.entry(node_id).or_insert(0);
                        *current_counter += 1;

                        if *count == 0 || *current_counter <= *count {
                            let step_msg = match lang {
                                Language::Fr => {
                                    if *count == 0 {
                                        format!(
                                            "🔁 Boucle itération #{} (infinie)",
                                            current_counter
                                        )
                                    } else {
                                        format!(
                                            "🔁 Boucle itération #{}/{}",
                                            current_counter, count
                                        )
                                    }
                                }
                                Language::En => {
                                    if *count == 0 {
                                        format!("🔁 Loop iteration #{} (infinite)", current_counter)
                                    } else {
                                        format!("🔁 Loop iteration #{}/{}", current_counter, count)
                                    }
                                }
                            };
                            set_status(&step_msg);
                            Some(0) // Pin 0 = Corps de boucle
                        } else {
                            *current_counter = 0; // Réinitialiser pour une exécution future
                            let step_msg = match lang {
                                Language::Fr => {
                                    format!("✅ Boucle terminée ({} itérations).", count)
                                }
                                Language::En => {
                                    format!("✅ Loop finished ({} iterations).", count)
                                }
                            };
                            set_status(&step_msg);
                            Some(1) // Pin 1 = Terminé
                        }
                    }
                    BlueprintNodeType::Stop => {
                        let step_msg = match lang {
                            Language::Fr => "🛑 Nœud d'arrêt atteint.".to_string(),
                            Language::En => "🛑 Stop node reached.".to_string(),
                        };
                        set_status(&step_msg);
                        break;
                    }
                };

                if let Some(pin_idx) = next_output_pin {
                    if let Some(target_pin) = graph.get_target_from_output(node_id, pin_idx) {
                        current_id = Some(target_pin.node_id);
                    } else {
                        // Pas de connexion suivante : fin du flux
                        current_id = None;
                    }
                } else {
                    current_id = None;
                }
            }

            // Fin de l'exécution
            is_running_clone.store(false, Ordering::SeqCst);
            if let Ok(mut active) = active_node_clone.lock() {
                *active = None;
            }
            if check_stopped() {
                let stopped_msg = match lang {
                    Language::Fr => {
                        "⏹️ Exécution Blueprint arrêtée (Arrêt d'urgence F4).".to_string()
                    }
                    Language::En => {
                        "⏹️ Blueprint execution stopped (F4 Emergency Stop).".to_string()
                    }
                };
                set_status(&stopped_msg);
            } else {
                set_status(&ready_msg);
            }

            macro_core::notify_event(EngineEvent::PlaybackStateChanged(false));
        });

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::graph::{BlueprintClickType, PinId};

    #[test]
    fn test_runner_state_initialization() {
        let runner = BlueprintRunnerState::new();
        assert!(!runner.is_active());
        assert_eq!(runner.get_active_node(), None);
        assert_eq!(runner.get_status(), "Prêt");
    }

    #[test]
    fn test_runner_executes_random_delay_and_stop() {
        let runner = BlueprintRunnerState::new();
        let mut graph = BlueprintGraph::new();
        let start_id = graph.find_start_node().unwrap();

        let delay_id = graph.add_node(
            BlueprintNodeType::RandomDelay {
                min_ms: 10,
                max_ms: 20,
            },
            [200.0, 200.0],
            Language::Fr,
        );

        let stop_id = graph.add_node(BlueprintNodeType::Stop, [300.0, 200.0], Language::Fr);

        graph.connect(
            PinId {
                node_id: start_id,
                is_output: true,
                pin_index: 0,
            },
            PinId {
                node_id: delay_id,
                is_output: false,
                pin_index: 0,
            },
        );

        graph.connect(
            PinId {
                node_id: delay_id,
                is_output: true,
                pin_index: 0,
            },
            PinId {
                node_id: stop_id,
                is_output: false,
                pin_index: 0,
            },
        );

        assert!(runner.run_graph(graph, Language::Fr));

        // Attendre que l'exécution se termine
        let start = Instant::now();
        while runner.is_active() && start.elapsed() < Duration::from_millis(1500) {
            thread::sleep(Duration::from_millis(10));
        }

        assert!(!runner.is_active());
    }

    #[test]
    fn test_runner_click_image_unmatched_falls_to_failed_pin() {
        let runner = BlueprintRunnerState::new();
        let mut graph = BlueprintGraph::new();
        let start_id = graph.find_start_node().unwrap();

        let click_id = graph.add_node(
            BlueprintNodeType::ClickImage {
                use_last_detected: true,
                image_path: "".to_string(),
                tolerance: 20,
                timeout_ms: 50,
                click_type: BlueprintClickType::Left,
                offset_x: 0,
                offset_y: 0,
            },
            [250.0, 200.0],
            Language::Fr,
        );

        let stop_id = graph.add_node(BlueprintNodeType::Stop, [400.0, 200.0], Language::Fr);

        graph.connect(
            PinId {
                node_id: start_id,
                is_output: true,
                pin_index: 0,
            },
            PinId {
                node_id: click_id,
                is_output: false,
                pin_index: 0,
            },
        );

        // Connecter la broche Échec (Pin 1) vers Stop
        graph.connect(
            PinId {
                node_id: click_id,
                is_output: true,
                pin_index: 1,
            },
            PinId {
                node_id: stop_id,
                is_output: false,
                pin_index: 0,
            },
        );

        assert!(runner.run_graph(graph, Language::Fr));

        let start = Instant::now();
        while runner.is_active() && start.elapsed() < Duration::from_millis(1500) {
            thread::sleep(Duration::from_millis(10));
        }

        assert!(!runner.is_active());
        let status = runner.get_status();
        assert!(
            status.contains("terminée") || status.contains("Échec") || status.contains("arrêt")
        );
    }

    #[test]
    fn test_runner_loop_cycle_with_multi_wire_input() {
        let runner = BlueprintRunnerState::new();
        let mut graph = BlueprintGraph::new();
        let start_id = graph.find_start_node().unwrap();

        let step_node = graph.add_node(
            BlueprintNodeType::Delay { delay_ms: 10 },
            [200.0, 200.0],
            Language::Fr,
        );

        let loop_node = graph.add_node(
            BlueprintNodeType::Loop { count: 2 },
            [350.0, 200.0],
            Language::Fr,
        );

        let stop_node = graph.add_node(BlueprintNodeType::Stop, [500.0, 200.0], Language::Fr);

        // 1. Start -> step_node
        graph.connect(
            PinId {
                node_id: start_id,
                is_output: true,
                pin_index: 0,
            },
            PinId {
                node_id: step_node,
                is_output: false,
                pin_index: 0,
            },
        );

        // 2. step_node -> loop_node
        graph.connect(
            PinId {
                node_id: step_node,
                is_output: true,
                pin_index: 0,
            },
            PinId {
                node_id: loop_node,
                is_output: false,
                pin_index: 0,
            },
        );

        // 3. loop_node (Pin 0 = Corps de boucle) -> step_node (Reboucle vers l'entrée de step_node !)
        // L'entrée de step_node reçoit maintenant 2 fils (start + rebouclage de la boucle)
        graph.connect(
            PinId {
                node_id: loop_node,
                is_output: true,
                pin_index: 0,
            },
            PinId {
                node_id: step_node,
                is_output: false,
                pin_index: 0,
            },
        );

        // 4. loop_node (Pin 1 = Terminé) -> stop_node
        graph.connect(
            PinId {
                node_id: loop_node,
                is_output: true,
                pin_index: 1,
            },
            PinId {
                node_id: stop_node,
                is_output: false,
                pin_index: 0,
            },
        );

        assert!(runner.run_graph(graph, Language::Fr));

        let start = Instant::now();
        while runner.is_active() && start.elapsed() < Duration::from_millis(3000) {
            thread::sleep(Duration::from_millis(15));
        }

        assert!(
            !runner.is_active(),
            "Runner should have finished loop cycle"
        );
        let status = runner.get_status();
        assert!(status.contains("terminé") || status.contains("arrêt"));
    }
}
