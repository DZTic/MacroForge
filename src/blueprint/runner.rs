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
            let mut max_steps = 100_000u32; // Protection anti-boucle infinie sans délai

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
                        let mut matched = false;
                        let timeout = Duration::from_millis(*timeout_ms);

                        loop {
                            if check_stopped() {
                                break;
                            }
                            if macro_core::check_image_present_with_tolerance(
                                image_path, *tolerance,
                            ) {
                                matched = true;
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

                        if matched {
                            let msg = match lang {
                                Language::Fr => {
                                    "✅ Image détectée ! Branche [Match] sélectionnée.".to_string()
                                }
                                Language::En => {
                                    "✅ Image found! [Match] branch selected.".to_string()
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
                    } => {
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
                        let mut found = false;
                        let timeout = Duration::from_millis(*timeout_ms);

                        loop {
                            if check_stopped() {
                                break;
                            }
                            if macro_core::check_image_present_with_tolerance(
                                image_path, *tolerance,
                            ) {
                                found = true;
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

                        if found {
                            Some(0) // Pin 0 = Found
                        } else {
                            Some(1) // Pin 1 = Timeout
                        }
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
                                Language::Fr => format!("🔁 Boucle itération #{}", current_counter),
                                Language::En => format!("🔁 Loop iteration #{}", current_counter),
                            };
                            set_status(&step_msg);
                            Some(0) // Pin 0 = Body
                        } else {
                            *current_counter = 0; // Réinitialiser pour une exécution future
                            Some(1) // Pin 1 = Completed
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
