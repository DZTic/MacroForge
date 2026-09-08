use crate::macro_core::MacroAction;
use crate::ui::i18n::Language;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PinId {
    pub node_id: NodeId,
    pub is_output: bool,
    pub pin_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlueprintClickType {
    Left,
    Right,
    Middle,
    DoubleLeft,
}

impl BlueprintClickType {
    pub fn label(&self, lang: Language) -> &'static str {
        match (self, lang) {
            (BlueprintClickType::Left, Language::Fr) => "Clic Gauche",
            (BlueprintClickType::Left, Language::En) => "Left Click",
            (BlueprintClickType::Right, Language::Fr) => "Clic Droit",
            (BlueprintClickType::Right, Language::En) => "Right Click",
            (BlueprintClickType::Middle, Language::Fr) => "Clic Milieu",
            (BlueprintClickType::Middle, Language::En) => "Middle Click",
            (BlueprintClickType::DoubleLeft, Language::Fr) => "Double-clic",
            (BlueprintClickType::DoubleLeft, Language::En) => "Double Click",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlueprintNodeType {
    /// Point d'entrée du graphe
    Start,
    /// Exécute une macro enregistrée (actions de clics/touches)
    Macro {
        name: String,
        actions: Vec<MacroAction>,
    },
    /// Vérifie instantanément si une image est visible à l'écran
    /// Sortie 0: Si Détectée (Match), Sortie 1: Si Absente (Miss)
    ImageCondition {
        image_path: String,
        tolerance: u8,
        timeout_ms: u64,
    },
    /// Attend jusqu'à timeout_ms qu'une image apparaisse
    /// Sortie 0: Trouvée (Found), Sortie 1: Délai dépassé (Timeout)
    WaitImage {
        image_path: String,
        timeout_ms: u64,
        tolerance: u8,
    },
    /// Clique sur une image : soit la dernière détectée, soit une image spécifique
    /// Sortie 0: Effectué (Done), Sortie 1: Échec / Absent (Failed)
    ClickImage {
        use_last_detected: bool,
        image_path: String,
        tolerance: u8,
        timeout_ms: u64,
        click_type: BlueprintClickType,
        offset_x: i32,
        offset_y: i32,
    },
    /// Clique aux coordonnées d'écran spécifiques (X, Y)
    /// Sortie 0: Suivant
    ClickCoordinate {
        x: i32,
        y: i32,
        click_type: BlueprintClickType,
        delay_after_ms: u64,
    },
    /// Déplace le curseur de la souris (absolu ou relatif)
    /// Sortie 0: Suivant
    MouseMove {
        x: i32,
        y: i32,
        relative: bool,
    },
    /// Simule l'appui sur une touche clavier
    /// Sortie 0: Suivant
    KeyPress {
        key_name: String,
        vk_code: u16,
        is_extended: bool,
        hold_ms: u64,
    },
    /// Pause aléatoire entre min_ms et max_ms
    /// Sortie 0: Suivant
    RandomDelay {
        min_ms: u64,
        max_ms: u64,
    },
    /// Défilement molette souris (steps > 0 haut, steps < 0 bas)
    /// Sortie 0: Suivant
    MouseScroll {
        steps: i32,
    },
    /// Pause temporelle en millisecondes
    Delay { delay_ms: u64 },
    /// Boucle d'exécution (count = 0 pour boucle infinie)
    /// Sortie 0: Corps de boucle (Body), Sortie 1: Terminé (Completed)
    Loop { count: u32 },
    /// Termine l'exécution de la branche active
    Stop,
}

impl BlueprintNodeType {
    pub fn default_title(&self, lang: Language) -> String {
        match (self, lang) {
            (BlueprintNodeType::Start, Language::Fr) => "Départ".to_string(),
            (BlueprintNodeType::Start, Language::En) => "Start".to_string(),
            (BlueprintNodeType::Macro { name, .. }, _) => {
                if name.is_empty() {
                    match lang {
                        Language::Fr => "Macro".to_string(),
                        Language::En => "Macro".to_string(),
                    }
                } else {
                    name.clone()
                }
            }
            (BlueprintNodeType::ImageCondition { .. }, Language::Fr) => {
                "Condition Image".to_string()
            }
            (BlueprintNodeType::ImageCondition { .. }, Language::En) => {
                "Image Condition".to_string()
            }
            (BlueprintNodeType::WaitImage { .. }, Language::Fr) => "Attente Image".to_string(),
            (BlueprintNodeType::WaitImage { .. }, Language::En) => "Wait Image".to_string(),
            (BlueprintNodeType::ClickImage { .. }, Language::Fr) => {
                "Cliquer sur l'image".to_string()
            }
            (BlueprintNodeType::ClickImage { .. }, Language::En) => "Click Image".to_string(),
            (BlueprintNodeType::ClickCoordinate { .. }, Language::Fr) => {
                "Clic Coordonnées".to_string()
            }
            (BlueprintNodeType::ClickCoordinate { .. }, Language::En) => {
                "Click Coordinates".to_string()
            }
            (BlueprintNodeType::MouseMove { .. }, Language::Fr) => "Déplacer Curseur".to_string(),
            (BlueprintNodeType::MouseMove { .. }, Language::En) => "Move Cursor".to_string(),
            (BlueprintNodeType::KeyPress { .. }, Language::Fr) => "Touche Clavier".to_string(),
            (BlueprintNodeType::KeyPress { .. }, Language::En) => "Key Press".to_string(),
            (BlueprintNodeType::RandomDelay { .. }, Language::Fr) => "Délai Aléatoire".to_string(),
            (BlueprintNodeType::RandomDelay { .. }, Language::En) => "Random Delay".to_string(),
            (BlueprintNodeType::MouseScroll { .. }, Language::Fr) => {
                "Défilement Molette".to_string()
            }
            (BlueprintNodeType::MouseScroll { .. }, Language::En) => "Mouse Scroll".to_string(),
            (BlueprintNodeType::Delay { .. }, Language::Fr) => "Pause / Délai".to_string(),
            (BlueprintNodeType::Delay { .. }, Language::En) => "Delay / Pause".to_string(),
            (BlueprintNodeType::Loop { .. }, Language::Fr) => "Boucle".to_string(),
            (BlueprintNodeType::Loop { .. }, Language::En) => "Loop".to_string(),
            (BlueprintNodeType::Stop, Language::Fr) => "Arrêt".to_string(),
            (BlueprintNodeType::Stop, Language::En) => "Stop".to_string(),
        }
    }

    pub fn input_pins(&self, lang: Language) -> Vec<String> {
        match self {
            BlueprintNodeType::Start => vec![],
            BlueprintNodeType::Macro { .. }
            | BlueprintNodeType::ImageCondition { .. }
            | BlueprintNodeType::WaitImage { .. }
            | BlueprintNodeType::ClickImage { .. }
            | BlueprintNodeType::ClickCoordinate { .. }
            | BlueprintNodeType::MouseMove { .. }
            | BlueprintNodeType::KeyPress { .. }
            | BlueprintNodeType::RandomDelay { .. }
            | BlueprintNodeType::MouseScroll { .. }
            | BlueprintNodeType::Delay { .. }
            | BlueprintNodeType::Stop => {
                vec!["Exec".to_string()]
            }
            BlueprintNodeType::Loop { .. } => match lang {
                Language::Fr => vec!["Exec".to_string(), "Réinit".to_string()],
                Language::En => vec!["Exec".to_string(), "Reset".to_string()],
            },
        }
    }

    pub fn output_pins(&self, lang: Language) -> Vec<String> {
        match self {
            BlueprintNodeType::Start => vec!["Exec".to_string()],
            BlueprintNodeType::Macro { .. }
            | BlueprintNodeType::Delay { .. }
            | BlueprintNodeType::ClickCoordinate { .. }
            | BlueprintNodeType::MouseMove { .. }
            | BlueprintNodeType::KeyPress { .. }
            | BlueprintNodeType::RandomDelay { .. }
            | BlueprintNodeType::MouseScroll { .. } => match lang {
                Language::Fr => vec!["Suivant".to_string()],
                Language::En => vec!["Next".to_string()],
            },
            BlueprintNodeType::ImageCondition { .. } => match lang {
                Language::Fr => vec![
                    "Si Présente (Match)".to_string(),
                    "Si Absente (Miss)".to_string(),
                ],
                Language::En => vec!["If Match".to_string(), "If Miss".to_string()],
            },
            BlueprintNodeType::WaitImage { .. } => match lang {
                Language::Fr => vec!["Trouvée".to_string(), "Délai Dépassé".to_string()],
                Language::En => vec!["Found".to_string(), "Timeout".to_string()],
            },
            BlueprintNodeType::ClickImage { .. } => match lang {
                Language::Fr => vec!["Effectué".to_string(), "Échec (Absent)".to_string()],
                Language::En => vec!["Done".to_string(), "Failed (Miss)".to_string()],
            },
            BlueprintNodeType::Loop { .. } => match lang {
                Language::Fr => vec!["Répéter".to_string(), "Terminé".to_string()],
                Language::En => vec!["Loop Body".to_string(), "Completed".to_string()],
            },
            BlueprintNodeType::Stop => vec![],
        }
    }

    pub fn min_dimensions(&self) -> (f32, f32) {
        match self {
            BlueprintNodeType::ClickImage { use_last_detected, .. } => {
                if *use_last_detected {
                    (260.0, 160.0)
                } else {
                    (270.0, 205.0)
                }
            }
            BlueprintNodeType::ClickCoordinate { .. } => (260.0, 160.0),
            BlueprintNodeType::MouseMove { .. } => (250.0, 145.0),
            BlueprintNodeType::KeyPress { .. } => (240.0, 130.0),
            BlueprintNodeType::RandomDelay { .. } => (220.0, 115.0),
            BlueprintNodeType::MouseScroll { .. } => (210.0, 105.0),
            BlueprintNodeType::ImageCondition { .. } | BlueprintNodeType::WaitImage { .. } => {
                (260.0, 150.0)
            }
            BlueprintNodeType::Loop { .. } => (220.0, 125.0),
            BlueprintNodeType::Macro { .. } => (240.0, 105.0),
            BlueprintNodeType::Delay { .. } => (210.0, 100.0),
            BlueprintNodeType::Start | BlueprintNodeType::Stop => (180.0, 85.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlueprintNode {
    pub id: NodeId,
    pub title: String,
    pub node_type: BlueprintNodeType,
    pub position: [f32; 2],
    pub width: f32,
    pub height: f32,
}

impl BlueprintNode {
    pub fn new(
        id: NodeId,
        node_type: BlueprintNodeType,
        position: [f32; 2],
        lang: Language,
    ) -> Self {
        let title = node_type.default_title(lang);
        let (width, height) = node_type.min_dimensions();

        Self {
            id,
            title,
            node_type,
            position,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BlueprintConnection {
    pub from: PinId,
    pub to: PinId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintGraph {
    pub name: String,
    pub nodes: Vec<BlueprintNode>,
    pub connections: Vec<BlueprintConnection>,
    pub next_node_id: u64,
    pub pan: [f32; 2],
    pub zoom: f32,
}

impl Default for BlueprintGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl BlueprintGraph {
    pub fn new() -> Self {
        let mut graph = Self {
            name: "Nouveau Blueprint".to_string(),
            nodes: Vec::new(),
            connections: Vec::new(),
            next_node_id: 1,
            pan: [100.0, 100.0],
            zoom: 1.0,
        };
        // Ajouter un nœud de départ par défaut
        graph.add_node(BlueprintNodeType::Start, [150.0, 200.0], Language::Fr);
        graph
    }

    pub fn next_id(&mut self) -> NodeId {
        let id = NodeId(self.next_node_id);
        self.next_node_id += 1;
        id
    }

    pub fn add_node(
        &mut self,
        node_type: BlueprintNodeType,
        position: [f32; 2],
        lang: Language,
    ) -> NodeId {
        let id = self.next_id();
        let node = BlueprintNode::new(id, node_type, position, lang);
        self.nodes.push(node);
        id
    }

    pub fn remove_node(&mut self, node_id: NodeId) {
        self.nodes.retain(|n| n.id != node_id);
        self.connections
            .retain(|c| c.from.node_id != node_id && c.to.node_id != node_id);
    }

    pub fn connect(&mut self, from: PinId, to: PinId) -> bool {
        // Validation : from doit être output, to doit être input
        if !from.is_output || to.is_output {
            return false;
        }
        // Pas d'auto-connexion sur le même nœud
        if from.node_id == to.node_id {
            return false;
        }
        // Remplacer une connexion existante vers cette broche d'entrée
        self.connections.retain(|c| c.to != to);
        // Éviter les doublons exacts
        let conn = BlueprintConnection { from, to };
        if !self.connections.contains(&conn) {
            self.connections.push(conn);
            true
        } else {
            false
        }
    }

    pub fn disconnect_pin(&mut self, pin: PinId) {
        self.connections.retain(|c| c.from != pin && c.to != pin);
    }

    pub fn remove_connection(&mut self, from: PinId, to: PinId) {
        self.connections.retain(|c| !(c.from == from && c.to == to));
    }

    pub fn find_node(&self, id: NodeId) -> Option<&BlueprintNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn find_node_mut(&mut self, id: NodeId) -> Option<&mut BlueprintNode> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    pub fn find_start_node(&self) -> Option<NodeId> {
        self.nodes
            .iter()
            .find(|n| matches!(n.node_type, BlueprintNodeType::Start))
            .map(|n| n.id)
            .or_else(|| self.nodes.first().map(|n| n.id))
    }

    /// Trouve la connexion sortante d'une broche spécifique
    pub fn get_target_from_output(&self, node_id: NodeId, pin_index: usize) -> Option<PinId> {
        self.connections
            .iter()
            .find(|c| {
                c.from.node_id == node_id && c.from.pin_index == pin_index && c.from.is_output
            })
            .map(|c| c.to)
    }

    /// Convertit la macro actuelle en nœud sur le canvas
    pub fn create_macro_node(
        &mut self,
        name: &str,
        actions: Vec<MacroAction>,
        position: [f32; 2],
        lang: Language,
    ) -> NodeId {
        let node_type = BlueprintNodeType::Macro {
            name: name.to_string(),
            actions,
        };
        self.add_node(node_type, position, lang)
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        let json = self.to_json()?;
        std::fs::write(path, json).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        Self::from_json(&content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macro_core::ActionType;

    #[test]
    fn test_blueprint_graph_creation_and_defaults() {
        let graph = BlueprintGraph::new();
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].node_type, BlueprintNodeType::Start);
        assert_eq!(graph.find_start_node(), Some(graph.nodes[0].id));
        assert_eq!(graph.pan, [100.0, 100.0]);
        assert_eq!(graph.zoom, 1.0);
    }

    #[test]
    fn test_blueprint_add_and_remove_node() {
        let mut graph = BlueprintGraph::new();
        let start_id = graph.nodes[0].id;

        let macro_id = graph.create_macro_node(
            "Test Macro",
            vec![MacroAction {
                action_type: ActionType::Wait(500),
                delay_ms: 100,
            }],
            [300.0, 200.0],
            Language::Fr,
        );
        assert_eq!(graph.nodes.len(), 2);

        let out_pin = PinId {
            node_id: start_id,
            is_output: true,
            pin_index: 0,
        };
        let in_pin = PinId {
            node_id: macro_id,
            is_output: false,
            pin_index: 0,
        };

        let connected = graph.connect(out_pin, in_pin);
        assert!(connected);
        assert_eq!(graph.connections.len(), 1);
        assert_eq!(graph.get_target_from_output(start_id, 0), Some(in_pin));

        // Supprimer le nœud Macro doit aussi nettoyer la connexion
        graph.remove_node(macro_id);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.connections.len(), 0);
        assert_eq!(graph.get_target_from_output(start_id, 0), None);
    }

    #[test]
    fn test_blueprint_connection_validation() {
        let mut graph = BlueprintGraph::new();
        let n1 = graph.nodes[0].id;
        let n2 = graph.add_node(
            BlueprintNodeType::Delay { delay_ms: 200 },
            [400.0, 100.0],
            Language::En,
        );

        let out1 = PinId {
            node_id: n1,
            is_output: true,
            pin_index: 0,
        };
        let out2 = PinId {
            node_id: n2,
            is_output: true,
            pin_index: 0,
        };
        let in1 = PinId {
            node_id: n1,
            is_output: false,
            pin_index: 0,
        };
        let in2 = PinId {
            node_id: n2,
            is_output: false,
            pin_index: 0,
        };

        // Rejeter connexion Sortie vers Sortie
        assert!(!graph.connect(out1, out2));
        // Rejeter connexion Entrée vers Entrée
        assert!(!graph.connect(in1, in2));
        // Rejeter auto-connexion sur le même nœud
        assert!(!graph.connect(out2, in2));

        // Connexion valide
        assert!(graph.connect(out1, in2));
        assert_eq!(graph.connections.len(), 1);

        // Déconnexion de broche
        graph.disconnect_pin(out1);
        assert_eq!(graph.connections.len(), 0);
    }

    #[test]
    fn test_blueprint_image_condition_and_pins() {
        let node_type = BlueprintNodeType::ImageCondition {
            image_path: "boss.png".to_string(),
            tolerance: 30,
            timeout_ms: 2000,
        };

        let inputs_fr = node_type.input_pins(Language::Fr);
        let outputs_fr = node_type.output_pins(Language::Fr);
        let outputs_en = node_type.output_pins(Language::En);

        assert_eq!(inputs_fr.len(), 1);
        assert_eq!(outputs_fr.len(), 2);
        assert!(outputs_fr[0].contains("Match"));
        assert!(outputs_fr[1].contains("Miss"));
        assert_eq!(outputs_en.len(), 2);
    }

    #[test]
    fn test_blueprint_json_serialization_roundtrip() {
        let mut graph = BlueprintGraph::new();
        let start_id = graph.nodes[0].id;
        let cond_id = graph.add_node(
            BlueprintNodeType::ImageCondition {
                image_path: "extreme.png".to_string(),
                tolerance: 20,
                timeout_ms: 1500,
            },
            [300.0, 150.0],
            Language::Fr,
        );

        let out_pin = PinId {
            node_id: start_id,
            is_output: true,
            pin_index: 0,
        };
        let in_pin = PinId {
            node_id: cond_id,
            is_output: false,
            pin_index: 0,
        };
        graph.connect(out_pin, in_pin);

        let json = graph.to_json().expect("Serialization failed");
        let loaded = BlueprintGraph::from_json(&json).expect("Deserialization failed");

        assert_eq!(loaded.nodes.len(), 2);
        assert_eq!(loaded.connections.len(), 1);
        assert_eq!(loaded.connections[0].from, out_pin);
        assert_eq!(loaded.connections[0].to, in_pin);
    }

    #[test]
    fn test_blueprint_node_min_dimensions_prevent_overflow() {
        let node_types = [
            BlueprintNodeType::Start,
            BlueprintNodeType::Macro {
                name: "Test".to_string(),
                actions: vec![],
            },
            BlueprintNodeType::ImageCondition {
                image_path: "test.png".to_string(),
                tolerance: 25,
                timeout_ms: 3000,
            },
            BlueprintNodeType::WaitImage {
                image_path: "test.png".to_string(),
                timeout_ms: 5000,
                tolerance: 25,
            },
            BlueprintNodeType::ClickImage {
                use_last_detected: true,
                image_path: "".to_string(),
                tolerance: 25,
                timeout_ms: 3000,
                click_type: BlueprintClickType::Left,
                offset_x: 0,
                offset_y: 0,
            },
            BlueprintNodeType::ClickImage {
                use_last_detected: false,
                image_path: "btn.png".to_string(),
                tolerance: 25,
                timeout_ms: 3000,
                click_type: BlueprintClickType::Right,
                offset_x: 10,
                offset_y: -5,
            },
            BlueprintNodeType::ClickCoordinate {
                x: 500,
                y: 300,
                click_type: BlueprintClickType::DoubleLeft,
                delay_after_ms: 50,
            },
            BlueprintNodeType::MouseMove {
                x: 100,
                y: 200,
                relative: false,
            },
            BlueprintNodeType::KeyPress {
                key_name: "Enter".to_string(),
                vk_code: 13,
                is_extended: false,
                hold_ms: 30,
            },
            BlueprintNodeType::RandomDelay {
                min_ms: 100,
                max_ms: 500,
            },
            BlueprintNodeType::MouseScroll { steps: 3 },
            BlueprintNodeType::Delay { delay_ms: 1000 },
            BlueprintNodeType::Loop { count: 3 },
            BlueprintNodeType::Stop,
        ];

        for nt in &node_types {
            let (min_w, min_h) = nt.min_dimensions();
            let node = BlueprintNode::new(NodeId(1), nt.clone(), [0.0, 0.0], Language::Fr);
            assert!(node.width >= min_w, "Node width must be >= min_w");
            assert!(node.height >= min_h, "Node height must be >= min_h");
            if matches!(
                nt,
                BlueprintNodeType::ImageCondition { .. }
                    | BlueprintNodeType::WaitImage { .. }
                    | BlueprintNodeType::ClickImage { .. }
                    | BlueprintNodeType::ClickCoordinate { .. }
            ) {
                assert!(node.width >= 260.0);
                assert!(node.height >= 150.0);
            }
        }
    }

    #[test]
    fn test_blueprint_click_nodes_and_pins() {
        let click_img = BlueprintNodeType::ClickImage {
            use_last_detected: true,
            image_path: "".to_string(),
            tolerance: 25,
            timeout_ms: 1000,
            click_type: BlueprintClickType::Left,
            offset_x: 0,
            offset_y: 0,
        };
        assert_eq!(click_img.input_pins(Language::Fr).len(), 1);
        let out_fr = click_img.output_pins(Language::Fr);
        assert_eq!(out_fr.len(), 2);
        assert!(out_fr[0].contains("Effectué"));
        assert!(out_fr[1].contains("Échec"));

        let click_coord = BlueprintNodeType::ClickCoordinate {
            x: 100,
            y: 200,
            click_type: BlueprintClickType::Right,
            delay_after_ms: 50,
        };
        assert_eq!(click_coord.input_pins(Language::Fr).len(), 1);
        assert_eq!(click_coord.output_pins(Language::Fr).len(), 1);

        assert_eq!(
            BlueprintClickType::Left.label(Language::Fr),
            "Clic Gauche"
        );
        assert_eq!(
            BlueprintClickType::DoubleLeft.label(Language::En),
            "Double Click"
        );
    }
}
