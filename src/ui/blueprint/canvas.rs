use crate::blueprint::graph::{
    BlueprintClickType, BlueprintGraph, BlueprintNode, BlueprintNodeType, NodeId, PinId,
};
use crate::blueprint::runner::BlueprintRunnerState;
use crate::macro_core::MacroAction;
use crate::ui::i18n::Language;
use crate::ui::theme::colors;
use crate::ui::widgets::{ButtonVariant, GlassButton};
use eframe::egui::{self, epaint::CubicBezierShape, Color32, Pos2, Rect, Rounding, Stroke, Vec2};

fn capture_cursor() -> Option<(i32, i32)> {
    #[cfg(windows)]
    {
        use winapi::shared::windef::POINT;
        use winapi::um::winuser::GetCursorPos;
        let mut pt = POINT { x: 0, y: 0 };
        unsafe {
            if GetCursorPos(&mut pt) != 0 {
                return Some((pt.x, pt.y));
            }
        }
    }
    None
}

pub struct BlueprintCanvas {
    pub pan: [f32; 2],
    pub zoom: f32,
    pub dragged_node: Option<(NodeId, Vec2)>,
    pub dragged_cable_from: Option<PinId>,
    pub selected_node: Option<NodeId>,
    pub editing_node_id: Option<NodeId>,
    pub status_notification: Option<(String, std::time::Instant)>,
}

impl Default for BlueprintCanvas {
    fn default() -> Self {
        Self::new()
    }
}

impl BlueprintCanvas {
    pub fn new() -> Self {
        Self {
            pan: [200.0, 150.0],
            zoom: 1.0,
            dragged_node: None,
            dragged_cable_from: None,
            selected_node: None,
            editing_node_id: None,
            status_notification: None,
        }
    }

    pub fn to_screen(&self, pos: [f32; 2], origin: Pos2) -> Pos2 {
        Pos2::new(
            origin.x + self.pan[0] + pos[0] * self.zoom,
            origin.y + self.pan[1] + pos[1] * self.zoom,
        )
    }

    pub fn to_graph(&self, screen_pos: Pos2, origin: Pos2) -> [f32; 2] {
        [
            (screen_pos.x - origin.x - self.pan[0]) / self.zoom,
            (screen_pos.y - origin.y - self.pan[1]) / self.zoom,
        ]
    }

    pub fn reset_view(&mut self) {
        self.pan = [200.0, 150.0];
        self.zoom = 1.0;
    }

    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        graph: &mut BlueprintGraph,
        runner: &BlueprintRunnerState,
        current_actions: &[MacroAction],
        lang: Language,
    ) {
        // 1. Barre d'outils supérieure du Blueprint
        self.render_top_bar(ui, graph, runner, lang);

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // 2. Zone principale : Palette latérale à gauche + Canvas interactif au centre/droite
        let available_size = ui.available_size();
        let palette_width = 240.0;

        ui.horizontal(|ui| {
            // Volet Palette Latérale
            ui.allocate_ui_with_layout(
                egui::vec2(palette_width, available_size.y),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    self.render_palette(ui, graph, current_actions, lang);
                },
            );

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // Volet Canvas
            let canvas_size = ui.available_size();
            self.render_canvas_area(ui, canvas_size, graph, runner, lang);
        });

        // 3. Modale d'édition avancée de nœud suite à un double-clic
        self.render_node_edit_modal(ui.ctx(), graph, lang);
    }

    fn render_top_bar(
        &mut self,
        ui: &mut egui::Ui,
        graph: &mut BlueprintGraph,
        runner: &BlueprintRunnerState,
        lang: Language,
    ) {
        ui.horizontal(|ui| {
            // Bouton Exécuter / Arrêter
            let is_running = runner.is_active();
            if !is_running {
                let run_btn = GlassButton::new(lang.blueprint_run())
                    .icon("▶")
                    .variant(ButtonVariant::Success);
                if ui
                    .add(run_btn)
                    .on_hover_text("Exécuter le graphe de Blueprint (F7)")
                    .clicked()
                {
                    runner.run_graph(graph.clone(), lang);
                }
            } else {
                let stop_btn = GlassButton::new(lang.blueprint_stop())
                    .icon("⏹")
                    .variant(ButtonVariant::Warning);
                if ui
                    .add(stop_btn)
                    .on_hover_text("Interrompre l'exécution du Blueprint (F4)")
                    .clicked()
                {
                    runner.stop();
                }
            }

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // Sauvegarder Graphe
            let save_btn = GlassButton::new(lang.blueprint_save())
                .icon("💾")
                .compact(true)
                .variant(ButtonVariant::Secondary);
            if ui
                .add(save_btn)
                .on_hover_text("Sauvegarder le Blueprint dans un fichier .json")
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("MacroForge Blueprint", &["mfg", "json"])
                    .save_file()
                {
                    if let Some(path_str) = path.to_str() {
                        if let Err(e) = graph.save_to_file(path_str) {
                            self.status_notification = Some((
                                format!("❌ Erreur sauvegarde: {}", e),
                                std::time::Instant::now(),
                            ));
                        } else {
                            self.status_notification = Some((
                                "✅ Blueprint sauvegardé avec succès!".to_string(),
                                std::time::Instant::now(),
                            ));
                        }
                    }
                }
            }

            // Ouvrir Graphe
            let load_btn = GlassButton::new(lang.blueprint_load())
                .icon("📂")
                .compact(true)
                .variant(ButtonVariant::Secondary);
            if ui
                .add(load_btn)
                .on_hover_text("Ouvrir un Blueprint existant (.mfg / .json)")
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("MacroForge Blueprint", &["mfg", "json"])
                    .pick_file()
                {
                    if let Some(path_str) = path.to_str() {
                        match BlueprintGraph::load_from_file(path_str) {
                            Ok(loaded_graph) => {
                                *graph = loaded_graph;
                                self.status_notification = Some((
                                    "✅ Blueprint chargé avec succès!".to_string(),
                                    std::time::Instant::now(),
                                ));
                            }
                            Err(e) => {
                                self.status_notification = Some((
                                    format!("❌ Erreur chargement: {}", e),
                                    std::time::Instant::now(),
                                ));
                            }
                        }
                    }
                }
            }

            // Vider Graphe
            let clear_btn = GlassButton::new(lang.blueprint_clear())
                .icon("🧹")
                .compact(true)
                .variant(ButtonVariant::Ghost);
            if ui
                .add(clear_btn)
                .on_hover_text("Effacer tous les nœuds du Blueprint")
                .clicked()
            {
                *graph = BlueprintGraph::new();
                self.status_notification = Some((
                    "Graphe réinitialisé.".to_string(),
                    std::time::Instant::now(),
                ));
            }

            // Centrer la vue
            let center_btn = GlassButton::new(lang.blueprint_center())
                .icon("🎯")
                .compact(true)
                .variant(ButtonVariant::Ghost);
            if ui.add(center_btn).clicked() {
                self.reset_view();
            }

            // Statut en direct
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let status_text = if let Some((msg, time)) = &self.status_notification {
                    if time.elapsed() < std::time::Duration::from_secs(4) {
                        msg.clone()
                    } else {
                        runner.get_status()
                    }
                } else {
                    runner.get_status()
                };

                let dot_color = if runner.is_active() {
                    colors::ACCENT_SUCCESS
                } else {
                    colors::ACCENT_PRIMARY
                };

                ui.label(
                    egui::RichText::new(&status_text)
                        .color(colors::TEXT_SECONDARY)
                        .size(12.0),
                );
                ui.label(egui::RichText::new("●").color(dot_color).size(10.0));
            });
        });
    }

    fn render_palette(
        &mut self,
        ui: &mut egui::Ui,
        graph: &mut BlueprintGraph,
        current_actions: &[MacroAction],
        lang: Language,
    ) {
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new(lang.blueprint_node_library())
                .strong()
                .color(colors::TEXT_PRIMARY)
                .size(14.0),
        );

        ui.add_space(6.0);

        // Bouton vedette : Créer un nœud à partir de la macro actuelle
        let actions_count = current_actions.len();
        let btn_label = format!("{} ({})", lang.blueprint_create_from_macro(), actions_count);
        let make_macro_node_btn =
            GlassButton::new(&btn_label)
                .icon("⚡")
                .variant(if actions_count > 0 {
                    ButtonVariant::Primary
                } else {
                    ButtonVariant::Secondary
                });

        if ui
            .add(make_macro_node_btn)
            .on_hover_text(
                "Génère un nouveau nœud Macro autonome contenant les actions actuellement enregistrées",
            )
            .clicked()
        {
            let name = format!("Macro #{}", graph.nodes.len() + 1);
            let target_pos = [
                -self.pan[0] + 350.0 + (graph.nodes.len() as f32 * 25.0),
                -self.pan[1] + 200.0 + (graph.nodes.len() as f32 * 20.0),
            ];
            let new_id = graph.create_macro_node(&name, current_actions.to_vec(), target_pos, lang);
            self.selected_node = Some(new_id);
            self.status_notification = Some((
                format!("✅ Nœud Macro créé avec {} actions.", actions_count),
                std::time::Instant::now(),
            ));
        }

        ui.add_space(4.0);

        // Bouton Importer une macro .mforge en nœud
        let import_macro_btn = GlassButton::new(lang.blueprint_import_macro())
            .icon("📂")
            .variant(ButtonVariant::Secondary);
        if ui
            .add(import_macro_btn)
            .on_hover_text("Importer un fichier profil .mforge pour en faire un nœud de macro")
            .clicked()
        {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("MacroForge Profile", &["mforge", "json"])
                .pick_file()
            {
                if let Some(path_str) = path.to_str() {
                    match std::fs::read_to_string(path_str) {
                        Ok(content) => {
                            if let Ok(actions) = serde_json::from_str::<Vec<MacroAction>>(&content)
                            {
                                let file_name = path
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("Macro Importée");
                                let target_pos = [-self.pan[0] + 350.0, -self.pan[1] + 200.0];
                                let new_id = graph.create_macro_node(
                                    file_name,
                                    actions.clone(),
                                    target_pos,
                                    lang,
                                );
                                self.selected_node = Some(new_id);
                                self.status_notification = Some((
                                    format!(
                                        "✅ Nœud '{}' créé ({} actions).",
                                        file_name,
                                        actions.len()
                                    ),
                                    std::time::Instant::now(),
                                ));
                            } else {
                                self.status_notification = Some((
                                    "❌ Fichier .mforge invalide.".to_string(),
                                    std::time::Instant::now(),
                                ));
                            }
                        }
                        Err(e) => {
                            self.status_notification = Some((
                                format!("❌ Erreur lecture: {}", e),
                                std::time::Instant::now(),
                            ));
                        }
                    }
                }
            }
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Blocs prédéfinis de la bibliothèque
        let template_items: Vec<(&'static str, BlueprintNodeType)> = vec![
            ("🟢", BlueprintNodeType::Start),
            (
                "🟣",
                BlueprintNodeType::ImageCondition {
                    image_path: "embedded://extreme.png".to_string(),
                    tolerance: 25,
                    timeout_ms: 3000,
                },
            ),
            (
                "🔷",
                BlueprintNodeType::WaitImage {
                    image_path: "embedded://extreme.png".to_string(),
                    timeout_ms: 5000,
                    tolerance: 25,
                    delay_after_ms: 0,
                },
            ),
            (
                "🎯",
                BlueprintNodeType::ClickImage {
                    use_last_detected: true,
                    image_path: "embedded://extreme.png".to_string(),
                    tolerance: 25,
                    timeout_ms: 3000,
                    click_type: BlueprintClickType::Left,
                    offset_x: 0,
                    offset_y: 0,
                },
            ),
            (
                "📍",
                BlueprintNodeType::ClickCoordinate {
                    x: 500,
                    y: 500,
                    click_type: BlueprintClickType::Left,
                    delay_after_ms: 50,
                },
            ),
            (
                "🖱️",
                BlueprintNodeType::MouseMove {
                    x: 500,
                    y: 500,
                    relative: false,
                },
            ),
            (
                "⌨️",
                BlueprintNodeType::KeyPress {
                    key_name: "Space".to_string(),
                    vk_code: 32,
                    is_extended: false,
                    hold_ms: 30,
                },
            ),
            (
                "🎲",
                BlueprintNodeType::RandomDelay {
                    min_ms: 200,
                    max_ms: 800,
                },
            ),
            ("📜", BlueprintNodeType::MouseScroll { steps: -3 }),
            ("🟠", BlueprintNodeType::Delay { delay_ms: 1000 }),
            ("🟡", BlueprintNodeType::Loop { count: 3 }),
            ("🔴", BlueprintNodeType::Stop),
        ];

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (icon, node_type) in template_items {
                    let title = node_type.default_title(lang);
                    let display_label = format!("{} {}", icon, title);
                    let item_btn = GlassButton::new(&display_label).variant(ButtonVariant::Ghost);

                    let hover_tip = match lang {
                        Language::Fr => "Cliquer pour insérer ce nœud sur le canevas",
                        Language::En => "Click to insert this node onto the canvas",
                    };

                    if ui.add(item_btn).on_hover_text(hover_tip).clicked() {
                        let target_pos = [
                            -self.pan[0] + 300.0 + (graph.nodes.len() as f32 * 20.0),
                            -self.pan[1] + 180.0 + (graph.nodes.len() as f32 * 20.0),
                        ];
                        let id = graph.add_node(node_type, target_pos, lang);
                        self.selected_node = Some(id);
                    }
                    ui.add_space(3.0);
                }

                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new(lang.blueprint_empty_hint())
                        .size(11.0)
                        .color(colors::TEXT_MUTED),
                );
            });
    }

    fn render_canvas_area(
        &mut self,
        ui: &mut egui::Ui,
        size: Vec2,
        graph: &mut BlueprintGraph,
        runner: &BlueprintRunnerState,
        lang: Language,
    ) {
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
        let painter = ui.painter_at(rect);
        let origin = rect.min;

        // Gestion du Pan (déplacement du canevas)
        if response.dragged() && self.dragged_node.is_none() && self.dragged_cable_from.is_none() {
            let delta = response.drag_delta();
            self.pan[0] += delta.x;
            self.pan[1] += delta.y;
        }

        // Gestion du Zoom avec molette souris
        let scroll = ui.input(|i| i.raw_scroll_delta.y);
        if scroll != 0.0 && response.hovered() {
            let zoom_factor = if scroll > 0.0 { 1.1 } else { 0.9 };
            let new_zoom = (self.zoom * zoom_factor).clamp(0.4, 2.2);

            if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                // Zoomer vers la position du curseur
                let mouse_graph_x = (mouse_pos.x - origin.x - self.pan[0]) / self.zoom;
                let mouse_graph_y = (mouse_pos.y - origin.y - self.pan[1]) / self.zoom;

                self.zoom = new_zoom;
                self.pan[0] = mouse_pos.x - origin.x - mouse_graph_x * self.zoom;
                self.pan[1] = mouse_pos.y - origin.y - mouse_graph_y * self.zoom;
            } else {
                self.zoom = new_zoom;
            }
        }

        // 1. Dessiner le fond de grille
        self.draw_grid(&painter, rect);

        // Récupérer le nœud actif en cours d'exécution
        let active_node_id = runner.get_active_node();

        // 2. Dessiner les connexions existantes (Câbles Bézier)
        self.draw_connections(&painter, graph, origin, active_node_id);

        // 3. Dessiner le câble en cours de création par l'utilisateur
        if let Some(from_pin) = self.dragged_cable_from {
            if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                if let Some(from_node) = graph.find_node(from_pin.node_id) {
                    let from_pos = self.get_pin_screen_pos(from_node, from_pin, origin);
                    self.draw_bezier_cable(
                        &painter,
                        from_pos,
                        mouse_pos,
                        colors::ACCENT_CYAN,
                        true,
                    );
                }
            }
        }

        // 4. Dessiner les nœuds
        let mut node_to_delete: Option<NodeId> = None;
        let mut pin_under_mouse: Option<PinId> = None;

        for node in &mut graph.nodes {
            let (min_w, min_h) = node.node_type.min_dimensions();
            if node.width < min_w {
                node.width = min_w;
            }
            if node.height < min_h {
                node.height = min_h;
            }
            let screen_pos = self.to_screen(node.position, origin);
            let node_rect = Rect::from_min_size(
                screen_pos,
                Vec2::new(node.width * self.zoom, node.height * self.zoom),
            );

            // Interaction déplacement de nœud
            let node_id = node.id;
            let node_resp = ui.interact(
                node_rect,
                ui.make_persistent_id(node_id),
                egui::Sense::click_and_drag(),
            );

            if node_resp.clicked() {
                self.selected_node = Some(node_id);
            }

            if node_resp.double_clicked() {
                self.selected_node = Some(node_id);
                self.editing_node_id = Some(node_id);
            }

            if node_resp.drag_started() && self.dragged_cable_from.is_none() {
                self.dragged_node = Some((node_id, Vec2::ZERO));
                self.selected_node = Some(node_id);
            }

            if let Some((d_id, _)) = self.dragged_node {
                if d_id == node_id && node_resp.dragged() {
                    let delta = node_resp.drag_delta();
                    node.position[0] += delta.x / self.zoom;
                    node.position[1] += delta.y / self.zoom;
                }
            }

            let is_active = active_node_id == Some(node_id);
            let is_selected = self.selected_node == Some(node_id);

            // Dessiner la carte du nœud
            let deleted = self.draw_node_card(
                ui,
                &painter,
                node,
                node_rect,
                is_active,
                is_selected,
                &mut pin_under_mouse,
                lang,
            );

            if deleted {
                node_to_delete = Some(node_id);
            }
        }

        // Relâchement du drag de nœud
        if ui.input(|i| i.pointer.any_released()) {
            self.dragged_node = None;

            // Relâchement du câble sur une broche cible
            if let Some(from_pin) = self.dragged_cable_from {
                if let Some(to_pin) = pin_under_mouse {
                    if !to_pin.is_output && from_pin.is_output && to_pin.node_id != from_pin.node_id
                    {
                        graph.connect(from_pin, to_pin);
                    }
                }
                self.dragged_cable_from = None;
            }
        }

        // Clic droit sur le canvas : annuler le câble ou désélectionner
        if ui.input(|i| i.pointer.secondary_clicked()) {
            if self.dragged_cable_from.is_some() {
                self.dragged_cable_from = None;
            } else if let Some(pin) = pin_under_mouse {
                graph.disconnect_pin(pin);
            }
        }

        // Suppression de nœud si demandée
        if let Some(id) = node_to_delete {
            graph.remove_node(id);
            if self.selected_node == Some(id) {
                self.selected_node = None;
            }
        }
    }

    fn draw_grid(&self, painter: &egui::Painter, rect: Rect) {
        let grid_size = 40.0 * self.zoom;
        let subtle_color = Color32::from_rgba_unmultiplied(35, 45, 65, 80);
        let major_color = Color32::from_rgba_unmultiplied(50, 70, 100, 120);

        let start_x = rect.min.x + (self.pan[0] % grid_size);
        let start_y = rect.min.y + (self.pan[1] % grid_size);

        let mut x = start_x;
        let mut col = 0;
        while x < rect.max.x {
            let stroke = if col % 4 == 0 {
                Stroke::new(1.0_f32, major_color)
            } else {
                Stroke::new(0.5_f32, subtle_color)
            };
            painter.line_segment([Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)], stroke);
            x += grid_size;
            col += 1;
        }

        let mut y = start_y;
        let mut row = 0;
        while y < rect.max.y {
            let stroke = if row % 4 == 0 {
                Stroke::new(1.0_f32, major_color)
            } else {
                Stroke::new(0.5_f32, subtle_color)
            };
            painter.line_segment([Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)], stroke);
            y += grid_size;
            row += 1;
        }
    }

    fn draw_connections(
        &self,
        painter: &egui::Painter,
        graph: &BlueprintGraph,
        origin: Pos2,
        active_node_id: Option<NodeId>,
    ) {
        for conn in &graph.connections {
            if let (Some(from_node), Some(to_node)) = (
                graph.find_node(conn.from.node_id),
                graph.find_node(conn.to.node_id),
            ) {
                let from_pos = self.get_pin_screen_pos(from_node, conn.from, origin);
                let to_pos = self.get_pin_screen_pos(to_node, conn.to, origin);

                let is_active_wire = active_node_id == Some(conn.from.node_id);
                let wire_color = if is_active_wire {
                    colors::ACCENT_SUCCESS
                } else {
                    colors::ACCENT_PRIMARY
                };

                self.draw_bezier_cable(painter, from_pos, to_pos, wire_color, is_active_wire);
            }
        }
    }

    fn draw_bezier_cable(
        &self,
        painter: &egui::Painter,
        from_pos: Pos2,
        to_pos: Pos2,
        color: Color32,
        glowing: bool,
    ) {
        let dx = ((to_pos.x - from_pos.x).abs() * 0.5 + 40.0 * self.zoom).min(250.0 * self.zoom);
        let cp1 = Pos2::new(from_pos.x + dx, from_pos.y);
        let cp2 = Pos2::new(to_pos.x - dx, to_pos.y);

        let stroke_width = if glowing {
            3.5 * self.zoom
        } else {
            2.2 * self.zoom
        };

        if glowing {
            let glow_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 70);
            painter.add(CubicBezierShape::from_points_stroke(
                [from_pos, cp1, cp2, to_pos],
                false,
                Color32::TRANSPARENT,
                Stroke::new(stroke_width + 4.0_f32, glow_color),
            ));
        }

        painter.add(CubicBezierShape::from_points_stroke(
            [from_pos, cp1, cp2, to_pos],
            false,
            Color32::TRANSPARENT,
            Stroke::new(stroke_width, color),
        ));
    }

    fn get_pin_screen_pos(&self, node: &BlueprintNode, pin: PinId, origin: Pos2) -> Pos2 {
        let (min_w, min_h) = node.node_type.min_dimensions();
        let effective_w = node.width.max(min_w);
        let effective_h = node.height.max(min_h);
        let screen_rect = Rect::from_min_size(
            self.to_screen(node.position, origin),
            Vec2::new(effective_w * self.zoom, effective_h * self.zoom),
        );

        let header_h = 28.0 * self.zoom;
        let content_y = screen_rect.min.y + header_h;
        let pin_y_offset = (18.0 + (pin.pin_index as f32 * 22.0)) * self.zoom;
        let y = content_y + pin_y_offset;
        let pin_inset = 9.0 * self.zoom;

        if pin.is_output {
            Pos2::new(screen_rect.max.x - pin_inset, y)
        } else {
            Pos2::new(screen_rect.min.x + pin_inset, y)
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_node_card(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        node: &mut BlueprintNode,
        rect: Rect,
        is_active: bool,
        is_selected: bool,
        pin_under_mouse: &mut Option<PinId>,
        lang: Language,
    ) -> bool {
        let mut delete_clicked = false;
        let header_height = 28.0 * self.zoom;
        let header_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), header_height));

        // Couleur d'en-tête selon le type de nœud
        let header_color = match &node.node_type {
            BlueprintNodeType::Start => colors::ACCENT_SUCCESS,
            BlueprintNodeType::Macro { .. } => colors::ACCENT_PRIMARY,
            BlueprintNodeType::ImageCondition { .. } => colors::ACCENT_PURPLE,
            BlueprintNodeType::WaitImage { .. } => colors::ACCENT_CYAN,
            BlueprintNodeType::ClickImage { .. } => Color32::from_rgb(217, 70, 239), // Rose/Fuchsia
            BlueprintNodeType::ClickCoordinate { .. } => Color32::from_rgb(14, 165, 233), // Bleu ciel
            BlueprintNodeType::MouseMove { .. } => Color32::from_rgb(99, 102, 241),       // Indigo
            BlueprintNodeType::KeyPress { .. } => Color32::from_rgb(168, 85, 247),        // Violet
            BlueprintNodeType::RandomDelay { .. } => Color32::from_rgb(245, 158, 11),     // Ambre
            BlueprintNodeType::MouseScroll { .. } => Color32::from_rgb(20, 184, 166), // Sarcelle (Teal)
            BlueprintNodeType::Delay { .. } => colors::ACCENT_WARNING,
            BlueprintNodeType::Loop { .. } => Color32::from_rgb(234, 179, 8),
            BlueprintNodeType::Stop => colors::ACCENT_DANGER,
        };

        // Bordure & ombre du nœud
        let border_stroke = if is_active {
            Stroke::new(3.0 * self.zoom, colors::ACCENT_SUCCESS_HOVER)
        } else if is_selected {
            Stroke::new(2.0 * self.zoom, colors::ACCENT_PRIMARY_HOVER)
        } else {
            Stroke::new(1.0 * self.zoom, colors::BORDER_CARD)
        };

        // Fond du nœud
        painter.rect(
            rect,
            Rounding::same(8.0 * self.zoom),
            colors::BG_CARD,
            border_stroke,
        );

        // En-tête
        painter.rect_filled(
            header_rect,
            Rounding {
                nw: 8.0 * self.zoom,
                ne: 8.0 * self.zoom,
                sw: 0.0,
                se: 0.0,
            },
            header_color,
        );

        // Titre de l'en-tête
        painter.text(
            Pos2::new(header_rect.min.x + 10.0 * self.zoom, header_rect.center().y),
            egui::Align2::LEFT_CENTER,
            &node.title,
            egui::FontId::proportional(13.0 * self.zoom),
            Color32::WHITE,
        );

        // Bouton suppression "🗑" en haut à droite de l'en-tête
        let del_btn_rect = Rect::from_min_size(
            Pos2::new(
                header_rect.max.x - 22.0 * self.zoom,
                header_rect.min.y + 4.0 * self.zoom,
            ),
            Vec2::splat(18.0 * self.zoom),
        );
        let del_resp = ui.interact(
            del_btn_rect,
            ui.make_persistent_id(("del", node.id)),
            egui::Sense::click(),
        );
        if del_resp.hovered() {
            painter.rect_filled(
                del_btn_rect,
                Rounding::same(4.0 * self.zoom),
                Color32::from_rgba_unmultiplied(200, 30, 30, 180),
            );
        }
        painter.text(
            del_btn_rect.center(),
            egui::Align2::CENTER_CENTER,
            "×",
            egui::FontId::monospace(14.0 * self.zoom),
            Color32::WHITE,
        );
        if del_resp.clicked() {
            delete_clicked = true;
        }

        // Broches d'entrée (à gauche, entièrement dans le nœud)
        let inputs = node.node_type.input_pins(lang);
        let pin_radius = 5.5 * self.zoom;
        let pin_inset = 9.0 * self.zoom;

        for (idx, pin_name) in inputs.iter().enumerate() {
            let pin_id = PinId {
                node_id: node.id,
                is_output: false,
                pin_index: idx,
            };
            let pin_center = Pos2::new(
                rect.min.x + pin_inset,
                header_rect.max.y + (18.0 + idx as f32 * 22.0) * self.zoom,
            );
            let pin_interact_rect =
                Rect::from_center_size(pin_center, Vec2::splat(16.0 * self.zoom));
            let pin_resp = ui.interact(
                pin_interact_rect,
                ui.make_persistent_id(("pin", pin_id)),
                egui::Sense::click(),
            );

            let is_hovered = pin_resp.hovered();
            if is_hovered {
                *pin_under_mouse = Some(pin_id);
            }

            let pin_color = if is_hovered {
                colors::ACCENT_CYAN_HOVER
            } else {
                Color32::from_rgb(180, 200, 230)
            };

            painter.circle_filled(
                pin_center,
                if is_hovered {
                    pin_radius * 1.2
                } else {
                    pin_radius
                },
                pin_color,
            );
            painter.circle_stroke(pin_center, pin_radius, Stroke::new(1.0_f32, Color32::WHITE));

            painter.text(
                Pos2::new(pin_center.x + 9.0 * self.zoom, pin_center.y),
                egui::Align2::LEFT_CENTER,
                pin_name,
                egui::FontId::proportional(11.0 * self.zoom),
                colors::TEXT_SECONDARY,
            );
        }

        // Broches de sortie (à droite, entièrement dans le nœud)
        let outputs = node.node_type.output_pins(lang);
        for (idx, pin_name) in outputs.iter().enumerate() {
            let pin_id = PinId {
                node_id: node.id,
                is_output: true,
                pin_index: idx,
            };
            let pin_center = Pos2::new(
                rect.max.x - pin_inset,
                header_rect.max.y + (18.0 + idx as f32 * 22.0) * self.zoom,
            );
            let pin_interact_rect =
                Rect::from_center_size(pin_center, Vec2::splat(16.0 * self.zoom));
            let pin_resp = ui.interact(
                pin_interact_rect,
                ui.make_persistent_id(("pin", pin_id)),
                egui::Sense::click_and_drag(),
            );

            let is_hovered = pin_resp.hovered();
            if is_hovered {
                *pin_under_mouse = Some(pin_id);
            }

            if pin_resp.drag_started() {
                self.dragged_cable_from = Some(pin_id);
            }

            let pin_color = match idx {
                0 => colors::ACCENT_SUCCESS,
                1 => colors::ACCENT_DANGER,
                _ => colors::ACCENT_PRIMARY,
            };

            painter.circle_filled(
                pin_center,
                if is_hovered {
                    pin_radius * 1.2
                } else {
                    pin_radius
                },
                pin_color,
            );
            painter.circle_stroke(pin_center, pin_radius, Stroke::new(1.0_f32, Color32::WHITE));

            painter.text(
                Pos2::new(pin_center.x - 9.0 * self.zoom, pin_center.y),
                egui::Align2::RIGHT_CENTER,
                pin_name,
                egui::FontId::proportional(11.0 * self.zoom),
                colors::TEXT_SECONDARY,
            );
        }

        // Corps du nœud : paramètres éditables (placés sous les broches, strictement contenus)
        let num_pins = inputs.len().max(outputs.len());
        let pins_height = match num_pins {
            0 => 6.0 * self.zoom,
            1 => 34.0 * self.zoom,
            _ => 56.0 * self.zoom,
        };
        let body_top = header_rect.max.y + pins_height;
        let body_rect = Rect::from_min_max(
            Pos2::new(rect.min.x + 10.0 * self.zoom, body_top),
            Pos2::new(rect.max.x - 10.0 * self.zoom, rect.max.y - 8.0 * self.zoom),
        );

        if self.zoom >= 0.7 && body_rect.height() > 15.0 {
            ui.allocate_new_ui(
                egui::UiBuilder::new()
                    .max_rect(body_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
                |ui| {
                    ui.set_clip_rect(body_rect);
                    ui.set_max_width(body_rect.width());
                    ui.spacing_mut().item_spacing = Vec2::new(4.0 * self.zoom, 4.0 * self.zoom);
                    ui.spacing_mut().button_padding = Vec2::new(5.0 * self.zoom, 2.0 * self.zoom);

                    match &mut node.node_type {
                        BlueprintNodeType::Macro { actions, .. } => {
                            let text = match lang {
                                Language::Fr => format!("⚡ {} action(s)", actions.len()),
                                Language::En => format!("⚡ {} action(s)", actions.len()),
                            };
                            ui.label(
                                egui::RichText::new(text)
                                    .size(10.5 * self.zoom)
                                    .color(colors::TEXT_MUTED),
                            );
                        }
                        BlueprintNodeType::ImageCondition {
                            image_path,
                            tolerance,
                            timeout_ms,
                        } => {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("Img:")
                                        .size(10.5 * self.zoom)
                                        .color(colors::TEXT_MUTED),
                                );
                                let raw_name = std::path::Path::new(image_path)
                                    .file_name()
                                    .and_then(|f| f.to_str())
                                    .unwrap_or("...");
                                let display_name = if raw_name.chars().count() > 16 {
                                    let truncated: String = raw_name.chars().take(14).collect();
                                    format!("{}…", truncated)
                                } else {
                                    raw_name.to_string()
                                };
                                if ui
                                    .add(egui::Button::new(
                                        egui::RichText::new(display_name).size(10.5 * self.zoom),
                                    ))
                                    .on_hover_text(image_path.as_str())
                                    .clicked()
                                {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("Image", &["png", "bmp", "jpg"])
                                        .pick_file()
                                    {
                                        if let Some(p) = path.to_str() {
                                            *image_path = p.to_string();
                                        }
                                    }
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("Tol:")
                                        .size(10.5 * self.zoom)
                                        .color(colors::TEXT_MUTED),
                                );
                                ui.add(egui::DragValue::new(tolerance).range(0..=100).speed(1));
                                ui.label(
                                    egui::RichText::new("ms:")
                                        .size(10.5 * self.zoom)
                                        .color(colors::TEXT_MUTED),
                                );
                                ui.add(egui::DragValue::new(timeout_ms).range(0..=60000).speed(50));
                            });
                        }
                        BlueprintNodeType::WaitImage {
                            image_path,
                            timeout_ms,
                            tolerance,
                            delay_after_ms,
                        } => {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("Img:")
                                        .size(10.5 * self.zoom)
                                        .color(colors::TEXT_MUTED),
                                );
                                let raw_name = std::path::Path::new(image_path)
                                    .file_name()
                                    .and_then(|f| f.to_str())
                                    .unwrap_or("...");
                                let display_name = if raw_name.chars().count() > 16 {
                                    let truncated: String = raw_name.chars().take(14).collect();
                                    format!("{}…", truncated)
                                } else {
                                    raw_name.to_string()
                                };
                                if ui
                                    .add(egui::Button::new(
                                        egui::RichText::new(display_name).size(10.5 * self.zoom),
                                    ))
                                    .on_hover_text(image_path.as_str())
                                    .clicked()
                                {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("Image", &["png", "bmp", "jpg"])
                                        .pick_file()
                                    {
                                        if let Some(p) = path.to_str() {
                                            *image_path = p.to_string();
                                        }
                                    }
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("Tol:")
                                        .size(10.5 * self.zoom)
                                        .color(colors::TEXT_MUTED),
                                );
                                ui.add(egui::DragValue::new(tolerance).range(0..=100).speed(1));
                                ui.label(
                                    egui::RichText::new("ms:")
                                        .size(10.5 * self.zoom)
                                        .color(colors::TEXT_MUTED),
                                );
                                ui.add(
                                    egui::DragValue::new(timeout_ms)
                                        .range(100..=120000)
                                        .speed(100),
                                );
                            });
                            ui.horizontal(|ui| {
                                let pause_label = match lang {
                                    Language::Fr => "Pause après:",
                                    Language::En => "Post-wait:",
                                };
                                ui.label(
                                    egui::RichText::new(pause_label)
                                        .size(10.5 * self.zoom)
                                        .color(colors::TEXT_MUTED),
                                );
                                ui.add(
                                    egui::DragValue::new(delay_after_ms)
                                        .range(0..=60000)
                                        .speed(50)
                                        .suffix("ms"),
                                );
                            });
                        }
                        BlueprintNodeType::Delay { delay_ms } => {
                            ui.horizontal(|ui| {
                                let label = match lang {
                                    Language::Fr => "Délai (ms):",
                                    Language::En => "Delay (ms):",
                                };
                                ui.label(
                                    egui::RichText::new(label)
                                        .size(10.5 * self.zoom)
                                        .color(colors::TEXT_MUTED),
                                );
                                ui.add(egui::DragValue::new(delay_ms).range(10..=60000).speed(50));
                            });
                        }
                        BlueprintNodeType::Loop { count } => {
                            ui.horizontal(|ui| {
                                let label = match lang {
                                    Language::Fr => "Itérations (0=∞):",
                                    Language::En => "Iterations (0=∞):",
                                };
                                ui.label(
                                    egui::RichText::new(label)
                                        .size(10.5 * self.zoom)
                                        .color(colors::TEXT_MUTED),
                                );
                                ui.add(egui::DragValue::new(count).range(0..=1000).speed(1));
                            });
                            let hint_text = match lang {
                                Language::Fr => "↳ Reboucler fin sur entrée",
                                Language::En => "↳ Loop end back to in",
                            };
                            ui.label(
                                egui::RichText::new(hint_text)
                                    .size(9.0 * self.zoom)
                                    .color(Color32::from_rgb(234, 179, 8)),
                            );
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
                            ui.horizontal(|ui| {
                                let chk_label = match lang {
                                    Language::Fr => "Dernière image trouvée",
                                    Language::En => "Last matched image",
                                };
                                ui.checkbox(
                                    use_last_detected,
                                    egui::RichText::new(chk_label).size(10.5 * self.zoom),
                                );
                            });

                            if !*use_last_detected {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("Img:")
                                            .size(10.5 * self.zoom)
                                            .color(colors::TEXT_MUTED),
                                    );
                                    let raw_name = std::path::Path::new(image_path)
                                        .file_name()
                                        .and_then(|f| f.to_str())
                                        .unwrap_or("...");
                                    let display_name = if raw_name.chars().count() > 14 {
                                        let truncated: String = raw_name.chars().take(12).collect();
                                        format!("{}…", truncated)
                                    } else {
                                        raw_name.to_string()
                                    };
                                    if ui
                                        .add(egui::Button::new(
                                            egui::RichText::new(display_name)
                                                .size(10.5 * self.zoom),
                                        ))
                                        .on_hover_text(image_path.as_str())
                                        .clicked()
                                    {
                                        if let Some(path) = rfd::FileDialog::new()
                                            .add_filter("Image", &["png", "bmp", "jpg"])
                                            .pick_file()
                                        {
                                            if let Some(p) = path.to_str() {
                                                *image_path = p.to_string();
                                            }
                                        }
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("Tol:")
                                            .size(10.5 * self.zoom)
                                            .color(colors::TEXT_MUTED),
                                    );
                                    ui.add(egui::DragValue::new(tolerance).range(0..=100).speed(1));
                                    ui.label(
                                        egui::RichText::new("ms:")
                                            .size(10.5 * self.zoom)
                                            .color(colors::TEXT_MUTED),
                                    );
                                    ui.add(
                                        egui::DragValue::new(timeout_ms).range(0..=60000).speed(50),
                                    );
                                });
                            }

                            ui.horizontal(|ui| {
                                let type_label = match lang {
                                    Language::Fr => "Type:",
                                    Language::En => "Type:",
                                };
                                ui.label(
                                    egui::RichText::new(type_label)
                                        .size(10.5 * self.zoom)
                                        .color(colors::TEXT_MUTED),
                                );
                                egui::ComboBox::from_id_salt(
                                    ui.make_persistent_id(("click_img_combo", node.id)),
                                )
                                .selected_text(
                                    egui::RichText::new(click_type.label(lang))
                                        .size(10.0 * self.zoom),
                                )
                                .width(85.0 * self.zoom)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        click_type,
                                        BlueprintClickType::Left,
                                        BlueprintClickType::Left.label(lang),
                                    );
                                    ui.selectable_value(
                                        click_type,
                                        BlueprintClickType::Right,
                                        BlueprintClickType::Right.label(lang),
                                    );
                                    ui.selectable_value(
                                        click_type,
                                        BlueprintClickType::DoubleLeft,
                                        BlueprintClickType::DoubleLeft.label(lang),
                                    );
                                    ui.selectable_value(
                                        click_type,
                                        BlueprintClickType::Middle,
                                        BlueprintClickType::Middle.label(lang),
                                    );
                                });
                            });

                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("ΔX:")
                                        .size(10.5 * self.zoom)
                                        .color(colors::TEXT_MUTED),
                                );
                                ui.add(egui::DragValue::new(offset_x).speed(1));
                                ui.label(
                                    egui::RichText::new("ΔY:")
                                        .size(10.5 * self.zoom)
                                        .color(colors::TEXT_MUTED),
                                );
                                ui.add(egui::DragValue::new(offset_y).speed(1));
                            });
                        }
                        BlueprintNodeType::ClickCoordinate {
                            x,
                            y,
                            click_type,
                            delay_after_ms,
                        } => {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("X:")
                                        .size(10.5 * self.zoom)
                                        .color(colors::TEXT_MUTED),
                                );
                                ui.add(egui::DragValue::new(x).speed(1));
                                ui.label(
                                    egui::RichText::new("Y:")
                                        .size(10.5 * self.zoom)
                                        .color(colors::TEXT_MUTED),
                                );
                                ui.add(egui::DragValue::new(y).speed(1));

                                let cap_tip = match lang {
                                    Language::Fr => "Capturer la position actuelle du curseur",
                                    Language::En => "Capture current cursor position",
                                };
                                if ui
                                    .button(egui::RichText::new("📍").size(11.0 * self.zoom))
                                    .on_hover_text(cap_tip)
                                    .clicked()
                                {
                                    if let Some((cx, cy)) = capture_cursor() {
                                        *x = cx;
                                        *y = cy;
                                    }
                                }
                            });

                            ui.horizontal(|ui| {
                                let type_label = match lang {
                                    Language::Fr => "Type:",
                                    Language::En => "Type:",
                                };
                                ui.label(
                                    egui::RichText::new(type_label)
                                        .size(10.5 * self.zoom)
                                        .color(colors::TEXT_MUTED),
                                );
                                egui::ComboBox::from_id_salt(
                                    ui.make_persistent_id(("click_coord_combo", node.id)),
                                )
                                .selected_text(
                                    egui::RichText::new(click_type.label(lang))
                                        .size(10.0 * self.zoom),
                                )
                                .width(85.0 * self.zoom)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        click_type,
                                        BlueprintClickType::Left,
                                        BlueprintClickType::Left.label(lang),
                                    );
                                    ui.selectable_value(
                                        click_type,
                                        BlueprintClickType::Right,
                                        BlueprintClickType::Right.label(lang),
                                    );
                                    ui.selectable_value(
                                        click_type,
                                        BlueprintClickType::DoubleLeft,
                                        BlueprintClickType::DoubleLeft.label(lang),
                                    );
                                    ui.selectable_value(
                                        click_type,
                                        BlueprintClickType::Middle,
                                        BlueprintClickType::Middle.label(lang),
                                    );
                                });
                            });

                            ui.horizontal(|ui| {
                                let pause_label = match lang {
                                    Language::Fr => "Attente (ms):",
                                    Language::En => "Wait (ms):",
                                };
                                ui.label(
                                    egui::RichText::new(pause_label)
                                        .size(10.5 * self.zoom)
                                        .color(colors::TEXT_MUTED),
                                );
                                ui.add(
                                    egui::DragValue::new(delay_after_ms)
                                        .range(0..=10000)
                                        .speed(25),
                                );
                            });
                        }
                        BlueprintNodeType::MouseMove { x, y, relative } => {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(if *relative { "ΔX:" } else { "X:" })
                                        .size(10.5 * self.zoom)
                                        .color(colors::TEXT_MUTED),
                                );
                                ui.add(egui::DragValue::new(x).speed(1));
                                ui.label(
                                    egui::RichText::new(if *relative { "ΔY:" } else { "Y:" })
                                        .size(10.5 * self.zoom)
                                        .color(colors::TEXT_MUTED),
                                );
                                ui.add(egui::DragValue::new(y).speed(1));

                                if !*relative {
                                    let cap_tip = match lang {
                                        Language::Fr => "Capturer la position actuelle du curseur",
                                        Language::En => "Capture current cursor position",
                                    };
                                    if ui
                                        .button(egui::RichText::new("📍").size(11.0 * self.zoom))
                                        .on_hover_text(cap_tip)
                                        .clicked()
                                    {
                                        if let Some((cx, cy)) = capture_cursor() {
                                            *x = cx;
                                            *y = cy;
                                        }
                                    }
                                }
                            });

                            ui.horizontal(|ui| {
                                let rel_label = match lang {
                                    Language::Fr => "Relatif (Δ)",
                                    Language::En => "Relative (Δ)",
                                };
                                ui.checkbox(
                                    relative,
                                    egui::RichText::new(rel_label).size(10.5 * self.zoom),
                                );
                            });
                        }
                        BlueprintNodeType::KeyPress {
                            key_name,
                            vk_code,
                            is_extended,
                            hold_ms,
                        } => {
                            ui.horizontal(|ui| {
                                let key_label = match lang {
                                    Language::Fr => "Touche:",
                                    Language::En => "Key:",
                                };
                                ui.label(
                                    egui::RichText::new(key_label)
                                        .size(10.5 * self.zoom)
                                        .color(colors::TEXT_MUTED),
                                );

                                egui::ComboBox::from_id_salt(
                                    ui.make_persistent_id(("key_preset", node.id)),
                                )
                                .selected_text(
                                    egui::RichText::new(key_name.as_str()).size(10.5 * self.zoom),
                                )
                                .width(80.0 * self.zoom)
                                .show_ui(ui, |ui| {
                                    let common_keys: &[(&str, u16, bool)] = &[
                                        ("Enter", 13, false),
                                        ("Space", 32, false),
                                        ("Escape", 27, false),
                                        ("Tab", 9, false),
                                        ("Backspace", 8, false),
                                        ("Up", 38, true),
                                        ("Down", 40, true),
                                        ("Left", 37, true),
                                        ("Right", 39, true),
                                        ("F1", 112, false),
                                        ("F2", 113, false),
                                        ("F3", 114, false),
                                        ("F5", 116, false),
                                        ("A", 65, false),
                                        ("E", 69, false),
                                        ("F", 70, false),
                                        ("R", 82, false),
                                    ];
                                    for (name, vk, ext) in common_keys {
                                        if ui.selectable_label(key_name == *name, *name).clicked() {
                                            *key_name = name.to_string();
                                            *vk_code = *vk;
                                            *is_extended = *ext;
                                        }
                                    }
                                });
                            });

                            ui.horizontal(|ui| {
                                let hold_label = match lang {
                                    Language::Fr => "Maintien (ms):",
                                    Language::En => "Hold (ms):",
                                };
                                ui.label(
                                    egui::RichText::new(hold_label)
                                        .size(10.5 * self.zoom)
                                        .color(colors::TEXT_MUTED),
                                );
                                ui.add(egui::DragValue::new(hold_ms).range(0..=5000).speed(10));
                            });
                        }
                        BlueprintNodeType::RandomDelay { min_ms, max_ms } => {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("Min:")
                                        .size(10.5 * self.zoom)
                                        .color(colors::TEXT_MUTED),
                                );
                                ui.add(egui::DragValue::new(min_ms).range(1..=60000).speed(25));
                                ui.label(
                                    egui::RichText::new("Max:")
                                        .size(10.5 * self.zoom)
                                        .color(colors::TEXT_MUTED),
                                );
                                ui.add(egui::DragValue::new(max_ms).range(1..=60000).speed(25));
                            });
                        }
                        BlueprintNodeType::MouseScroll { steps } => {
                            ui.horizontal(|ui| {
                                let scr_label = match lang {
                                    Language::Fr => "Crans (+haut/-bas):",
                                    Language::En => "Steps (+up/-down):",
                                };
                                ui.label(
                                    egui::RichText::new(scr_label)
                                        .size(10.5 * self.zoom)
                                        .color(colors::TEXT_MUTED),
                                );
                                ui.add(egui::DragValue::new(steps).range(-50..=50).speed(1));
                            });
                        }
                        _ => {}
                    }
                },
            );
        }

        delete_clicked
    }

    /// Modale d'édition avancée d'un nœud ouverte lors d'un double-clic
    pub fn render_node_edit_modal(
        &mut self,
        ctx: &egui::Context,
        graph: &mut BlueprintGraph,
        lang: Language,
    ) {
        let editing_id = match self.editing_node_id {
            Some(id) => id,
            None => return,
        };

        let node = match graph.find_node_mut(editing_id) {
            Some(n) => n,
            None => {
                self.editing_node_id = None;
                return;
            }
        };

        let modal_title = match lang {
            Language::Fr => format!("⚙ Paramètres du Nœud : {}", node.title),
            Language::En => format!("⚙ Node Settings: {}", node.title),
        };

        let mut is_open = true;

        egui::Window::new(modal_title)
            .open(&mut is_open)
            .collapsible(false)
            .resizable(true)
            .default_size([460.0, 380.0])
            .min_width(400.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);

                // 1. Titre éditable du nœud
                ui.horizontal(|ui| {
                    let label = match lang {
                        Language::Fr => "Titre du nœud :",
                        Language::En => "Node title:",
                    };
                    ui.label(egui::RichText::new(label).strong().color(colors::TEXT_PRIMARY));
                    ui.text_edit_singleline(&mut node.title);
                });

                ui.separator();

                // 2. Paramètres spécifiques au type de nœud
                match &mut node.node_type {
                    BlueprintNodeType::WaitImage {
                        image_path,
                        timeout_ms,
                        tolerance,
                        delay_after_ms,
                    } => {
                        ui.label(
                            egui::RichText::new(match lang {
                                Language::Fr => "Configuration de l'attente d'apparition d'image :",
                                Language::En => "Image detection wait configuration:",
                            })
                            .strong()
                            .color(colors::ACCENT_CYAN_HOVER),
                        );

                        ui.horizontal(|ui| {
                            ui.label("Image :");
                            let path_label = if image_path.is_empty() {
                                "Parcourir...".to_string()
                            } else {
                                image_path.clone()
                            };
                            if ui
                                .button(egui::RichText::new(format!("📁 {}", path_label)))
                                .on_hover_text(image_path.as_str())
                                .clicked()
                            {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("Image", &["png", "bmp", "jpg"])
                                    .pick_file()
                                {
                                    if let Some(p) = path.to_str() {
                                        *image_path = p.to_string();
                                    }
                                }
                            }
                        });

                        ui.horizontal(|ui| {
                            let tol_label = match lang {
                                Language::Fr => "Tolérance chromatique (0-100) :",
                                Language::En => "Color tolerance (0-100):",
                            };
                            ui.label(tol_label);
                            ui.add(egui::Slider::new(tolerance, 0..=100));
                        });

                        ui.horizontal(|ui| {
                            let timeout_label = match lang {
                                Language::Fr => "Délai max d'attente (timeout ms) :",
                                Language::En => "Max wait timeout (ms):",
                            };
                            ui.label(timeout_label);
                            ui.add(
                                egui::DragValue::new(timeout_ms)
                                    .range(100..=120000)
                                    .speed(100)
                                    .suffix(" ms"),
                            );
                        });

                        ui.horizontal(|ui| {
                            let post_label = match lang {
                                Language::Fr => "Délai après détection (ms) :",
                                Language::En => "Post-detection delay (ms):",
                            };
                            ui.label(egui::RichText::new(post_label).strong().color(colors::ACCENT_SUCCESS));
                            ui.add(
                                egui::DragValue::new(delay_after_ms)
                                    .range(0..=60000)
                                    .speed(50)
                                    .suffix(" ms"),
                            );
                        });

                        let help_text = match lang {
                            Language::Fr => "💡 Le délai après détection permet d'attendre qu'une interface ou une animation se stabilise avant d'exécuter la prochaine action.",
                            Language::En => "💡 The post-detection delay allows animations or UI elements to settle before executing the next action.",
                        };
                        ui.label(egui::RichText::new(help_text).size(11.0).color(colors::TEXT_MUTED));
                    }
                    BlueprintNodeType::Loop { count } => {
                        ui.label(
                            egui::RichText::new(match lang {
                                Language::Fr => "Configuration de la Boucle :",
                                Language::En => "Loop Configuration:",
                            })
                            .strong()
                            .color(Color32::from_rgb(234, 179, 8)),
                        );

                        ui.horizontal(|ui| {
                            let count_label = match lang {
                                Language::Fr => "Nombre d'itérations (0 = infinie) :",
                                Language::En => "Iterations count (0 = infinite):",
                            };
                            ui.label(count_label);
                            ui.add(egui::DragValue::new(count).range(0..=10000).speed(1));
                        });

                        ui.add_space(4.0);
                        let guide_frame = egui::Frame::none()
                            .fill(Color32::from_rgba_unmultiplied(20, 25, 40, 180))
                            .stroke(egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(234, 179, 8, 80)))
                            .rounding(egui::Rounding::same(6.0))
                            .inner_margin(egui::Margin::same(10.0));

                        guide_frame.show(ui, |ui| {
                            let guide = match lang {
                                Language::Fr => "📌 Comment utiliser la boucle :\n1. Connecter l'action précédente à l'entrée [Exec].\n2. Relier la sortie [Corps de boucle] à la 1ère action à répéter.\n3. Raccorder la dernière action de votre séquence sur l'entrée [Exec] (rebouclage multi-fils).\n4. Connecter la sortie [Terminé] à la suite du graphe.",
                                Language::En => "📌 How to use the loop:\n1. Connect the previous action to the [Exec] input.\n2. Wire [Loop Body] to the first action to repeat.\n3. Wire the last action of your loop back to the [Exec] input (multi-wire loopback).\n4. Wire [Completed] to the next sequence after the loop.",
                            };
                            ui.label(egui::RichText::new(guide).size(11.5).color(colors::TEXT_SECONDARY));
                        });
                    }
                    BlueprintNodeType::ClickCoordinate {
                        x,
                        y,
                        click_type,
                        delay_after_ms,
                    } => {
                        ui.label(
                            egui::RichText::new(match lang {
                                Language::Fr => "Paramètres du Clic aux Coordonnées :",
                                Language::En => "Coordinate Click Settings:",
                            })
                            .strong()
                            .color(Color32::from_rgb(14, 165, 233)),
                        );

                        ui.horizontal(|ui| {
                            ui.label("X :");
                            ui.add(egui::DragValue::new(x).speed(1));
                            ui.label("Y :");
                            ui.add(egui::DragValue::new(y).speed(1));

                            let cap_tip = match lang {
                                Language::Fr => "Capturer la position actuelle du curseur",
                                Language::En => "Capture current cursor position",
                            };
                            if ui.button("📍 Capturer Curseur").on_hover_text(cap_tip).clicked() {
                                if let Some((cx, cy)) = capture_cursor() {
                                    *x = cx;
                                    *y = cy;
                                }
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Type de clic :");
                            egui::ComboBox::from_id_salt("modal_click_coord_combo")
                                .selected_text(click_type.label(lang))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(click_type, BlueprintClickType::Left, BlueprintClickType::Left.label(lang));
                                    ui.selectable_value(click_type, BlueprintClickType::Right, BlueprintClickType::Right.label(lang));
                                    ui.selectable_value(click_type, BlueprintClickType::DoubleLeft, BlueprintClickType::DoubleLeft.label(lang));
                                    ui.selectable_value(click_type, BlueprintClickType::Middle, BlueprintClickType::Middle.label(lang));
                                });
                        });

                        ui.horizontal(|ui| {
                            let label = match lang {
                                Language::Fr => "Délai après clic (ms) :",
                                Language::En => "Delay after click (ms):",
                            };
                            ui.label(label);
                            ui.add(egui::DragValue::new(delay_after_ms).range(0..=10000).speed(25).suffix(" ms"));
                        });
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
                        let chk_label = match lang {
                            Language::Fr => "Cliquer sur la dernière image détectée",
                            Language::En => "Click on the last matched image",
                        };
                        ui.checkbox(use_last_detected, chk_label);

                        if !*use_last_detected {
                            ui.horizontal(|ui| {
                                ui.label("Image :");
                                if ui.button(format!("📁 {}", if image_path.is_empty() { "Parcourir..." } else { image_path.as_str() })).clicked() {
                                    if let Some(path) = rfd::FileDialog::new().add_filter("Image", &["png", "bmp", "jpg"]).pick_file() {
                                        if let Some(p) = path.to_str() {
                                            *image_path = p.to_string();
                                        }
                                    }
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Tolérance :");
                                ui.add(egui::Slider::new(tolerance, 0..=100));
                            });
                            ui.horizontal(|ui| {
                                ui.label("Timeout (ms) :");
                                ui.add(egui::DragValue::new(timeout_ms).range(0..=60000).speed(50).suffix(" ms"));
                            });
                        }

                        ui.horizontal(|ui| {
                            ui.label("Type de clic :");
                            egui::ComboBox::from_id_salt("modal_click_img_combo")
                                .selected_text(click_type.label(lang))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(click_type, BlueprintClickType::Left, BlueprintClickType::Left.label(lang));
                                    ui.selectable_value(click_type, BlueprintClickType::Right, BlueprintClickType::Right.label(lang));
                                    ui.selectable_value(click_type, BlueprintClickType::DoubleLeft, BlueprintClickType::DoubleLeft.label(lang));
                                    ui.selectable_value(click_type, BlueprintClickType::Middle, BlueprintClickType::Middle.label(lang));
                                });
                        });

                        ui.horizontal(|ui| {
                            ui.label("Décalage ΔX :");
                            ui.add(egui::DragValue::new(offset_x).speed(1));
                            ui.label("ΔY :");
                            ui.add(egui::DragValue::new(offset_y).speed(1));
                        });
                    }
                    BlueprintNodeType::ImageCondition {
                        image_path,
                        tolerance,
                        timeout_ms,
                    } => {
                        ui.horizontal(|ui| {
                            ui.label("Image :");
                            if ui.button(format!("📁 {}", if image_path.is_empty() { "Parcourir..." } else { image_path.as_str() })).clicked() {
                                if let Some(path) = rfd::FileDialog::new().add_filter("Image", &["png", "bmp", "jpg"]).pick_file() {
                                    if let Some(p) = path.to_str() {
                                        *image_path = p.to_string();
                                    }
                                }
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Tolérance :");
                            ui.add(egui::Slider::new(tolerance, 0..=100));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Timeout (ms) :");
                            ui.add(egui::DragValue::new(timeout_ms).range(0..=60000).speed(50).suffix(" ms"));
                        });
                    }
                    BlueprintNodeType::MouseMove { x, y, relative } => {
                        ui.horizontal(|ui| {
                            ui.label(if *relative { "ΔX :" } else { "X :" });
                            ui.add(egui::DragValue::new(x).speed(1));
                            ui.label(if *relative { "ΔY :" } else { "Y :" });
                            ui.add(egui::DragValue::new(y).speed(1));

                            if !*relative && ui.button("📍 Capturer").clicked() {
                                if let Some((cx, cy)) = capture_cursor() {
                                    *x = cx;
                                    *y = cy;
                                }
                            }
                        });
                        ui.checkbox(relative, match lang {
                            Language::Fr => "Déplacement relatif (Δ)",
                            Language::En => "Relative movement (Δ)",
                        });
                    }
                    BlueprintNodeType::KeyPress {
                        key_name,
                        vk_code,
                        is_extended,
                        hold_ms,
                    } => {
                        ui.horizontal(|ui| {
                            ui.label("Touche :");
                            egui::ComboBox::from_id_salt("modal_key_preset")
                                .selected_text(key_name.as_str())
                                .show_ui(ui, |ui| {
                                    let common_keys: &[(&str, u16, bool)] = &[
                                        ("Enter", 13, false),
                                        ("Space", 32, false),
                                        ("Escape", 27, false),
                                        ("Tab", 9, false),
                                        ("Backspace", 8, false),
                                        ("Up", 38, true),
                                        ("Down", 40, true),
                                        ("Left", 37, true),
                                        ("Right", 39, true),
                                        ("F1", 112, false),
                                        ("F2", 113, false),
                                        ("F3", 114, false),
                                        ("F5", 116, false),
                                        ("A", 65, false),
                                        ("E", 69, false),
                                        ("F", 70, false),
                                        ("R", 82, false),
                                    ];
                                    for (name, vk, ext) in common_keys {
                                        if ui.selectable_label(key_name == *name, *name).clicked() {
                                            *key_name = name.to_string();
                                            *vk_code = *vk;
                                            *is_extended = *ext;
                                        }
                                    }
                                });
                        });

                        ui.horizontal(|ui| {
                            let label = match lang {
                                Language::Fr => "Maintien de la touche (ms) :",
                                Language::En => "Key hold duration (ms):",
                            };
                            ui.label(label);
                            ui.add(egui::DragValue::new(hold_ms).range(0..=5000).speed(10).suffix(" ms"));
                        });
                    }
                    BlueprintNodeType::RandomDelay { min_ms, max_ms } => {
                        ui.horizontal(|ui| {
                            ui.label("Min (ms) :");
                            ui.add(egui::DragValue::new(min_ms).range(1..=60000).speed(25));
                            ui.label("Max (ms) :");
                            ui.add(egui::DragValue::new(max_ms).range(1..=60000).speed(25));
                        });
                    }
                    BlueprintNodeType::MouseScroll { steps } => {
                        ui.horizontal(|ui| {
                            let label = match lang {
                                Language::Fr => "Crans molette (+haut/-bas) :",
                                Language::En => "Wheel steps (+up/-down):",
                            };
                            ui.label(label);
                            ui.add(egui::DragValue::new(steps).range(-50..=50).speed(1));
                        });
                    }
                    BlueprintNodeType::Delay { delay_ms } => {
                        ui.horizontal(|ui| {
                            let label = match lang {
                                Language::Fr => "Durée de la pause (ms) :",
                                Language::En => "Pause duration (ms):",
                            };
                            ui.label(label);
                            ui.add(egui::DragValue::new(delay_ms).range(10..=60000).speed(50).suffix(" ms"));
                        });
                    }
                    BlueprintNodeType::Macro { name, actions } => {
                        ui.label(format!("Macro : {} ({} actions)", name, actions.len()));
                    }
                    _ => {}
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                // 3. Bouton de validation / fermeture
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let close_label = match lang {
                        Language::Fr => "Valider & Fermer",
                        Language::En => "Apply & Close",
                    };
                    let btn = GlassButton::new(close_label).variant(ButtonVariant::Primary);
                    if ui.add(btn).clicked() {
                        self.editing_node_id = None;
                    }
                });
            });

        if !is_open {
            self.editing_node_id = None;
        }
    }
}
