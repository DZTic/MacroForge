use crate::blueprint::graph::{BlueprintGraph, BlueprintNode, BlueprintNodeType, NodeId, PinId};
use crate::blueprint::runner::BlueprintRunnerState;
use crate::macro_core::MacroAction;
use crate::ui::i18n::Language;
use crate::ui::theme::colors;
use crate::ui::widgets::{ButtonVariant, GlassButton};
use eframe::egui::{self, epaint::CubicBezierShape, Color32, Pos2, Rect, Rounding, Stroke, Vec2};

pub struct BlueprintCanvas {
    pub pan: [f32; 2],
    pub zoom: f32,
    pub dragged_node: Option<(NodeId, Vec2)>,
    pub dragged_cable_from: Option<PinId>,
    pub selected_node: Option<NodeId>,
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
        let template_items: Vec<(&'static str, &'static str, BlueprintNodeType)> = vec![
            ("🟢", "Départ", BlueprintNodeType::Start),
            (
                "🟣",
                "Condition Image",
                BlueprintNodeType::ImageCondition {
                    image_path: "embedded://extreme.png".to_string(),
                    tolerance: 25,
                    timeout_ms: 3000,
                },
            ),
            (
                "🔷",
                "Attente Image",
                BlueprintNodeType::WaitImage {
                    image_path: "embedded://extreme.png".to_string(),
                    timeout_ms: 5000,
                    tolerance: 25,
                },
            ),
            (
                "🟠",
                "Pause / Délai",
                BlueprintNodeType::Delay { delay_ms: 1000 },
            ),
            ("🟡", "Boucle", BlueprintNodeType::Loop { count: 3 }),
            ("🔴", "Arrêt", BlueprintNodeType::Stop),
        ];

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (icon, label, node_type) in template_items {
                    let display_label = format!("{} {}", icon, label);
                    let item_btn = GlassButton::new(&display_label).variant(ButtonVariant::Ghost);

                    if ui
                        .add(item_btn)
                        .on_hover_text("Cliquer pour insérer ce nœud sur le canevas")
                        .clicked()
                    {
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
        let screen_rect = Rect::from_min_size(
            self.to_screen(node.position, origin),
            Vec2::new(node.width * self.zoom, node.height * self.zoom),
        );

        let header_h = 28.0 * self.zoom;
        let content_y = screen_rect.min.y + header_h;
        let pin_y_offset = (20.0 + (pin.pin_index as f32 * 22.0)) * self.zoom;
        let y = content_y + pin_y_offset;

        if pin.is_output {
            Pos2::new(screen_rect.max.x, y)
        } else {
            Pos2::new(screen_rect.min.x, y)
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

        // Broches d'entrée (à gauche)
        let inputs = node.node_type.input_pins(lang);
        let pin_radius = 5.5 * self.zoom;

        for (idx, pin_name) in inputs.iter().enumerate() {
            let pin_id = PinId {
                node_id: node.id,
                is_output: false,
                pin_index: idx,
            };
            let pin_center = Pos2::new(
                rect.min.x,
                header_rect.max.y + (20.0 + idx as f32 * 22.0) * self.zoom,
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
                    pin_radius * 1.3
                } else {
                    pin_radius
                },
                pin_color,
            );
            painter.circle_stroke(pin_center, pin_radius, Stroke::new(1.0_f32, Color32::WHITE));

            painter.text(
                Pos2::new(pin_center.x + 10.0 * self.zoom, pin_center.y),
                egui::Align2::LEFT_CENTER,
                pin_name,
                egui::FontId::proportional(11.0 * self.zoom),
                colors::TEXT_SECONDARY,
            );
        }

        // Broches de sortie (à droite)
        let outputs = node.node_type.output_pins(lang);
        for (idx, pin_name) in outputs.iter().enumerate() {
            let pin_id = PinId {
                node_id: node.id,
                is_output: true,
                pin_index: idx,
            };
            let pin_center = Pos2::new(
                rect.max.x,
                header_rect.max.y + (20.0 + idx as f32 * 22.0) * self.zoom,
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
                    pin_radius * 1.3
                } else {
                    pin_radius
                },
                pin_color,
            );
            painter.circle_stroke(pin_center, pin_radius, Stroke::new(1.0_f32, Color32::WHITE));

            painter.text(
                Pos2::new(pin_center.x - 10.0 * self.zoom, pin_center.y),
                egui::Align2::RIGHT_CENTER,
                pin_name,
                egui::FontId::proportional(11.0 * self.zoom),
                colors::TEXT_SECONDARY,
            );
        }

        // Corps du nœud : paramètres éditables
        let body_rect = Rect::from_min_max(
            Pos2::new(
                rect.min.x + 8.0 * self.zoom,
                header_rect.max.y + 40.0 * self.zoom,
            ),
            Pos2::new(rect.max.x - 8.0 * self.zoom, rect.max.y - 6.0 * self.zoom),
        );

        if self.zoom >= 0.7 && body_rect.height() > 15.0 {
            ui.allocate_new_ui(
                egui::UiBuilder::new().max_rect(body_rect),
                |ui| match &mut node.node_type {
                    BlueprintNodeType::Macro { actions, .. } => {
                        ui.label(
                            egui::RichText::new(format!(
                                "⚡ {} actions enregistrées",
                                actions.len()
                            ))
                            .size(11.0 * self.zoom)
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
                            let short_path = std::path::Path::new(image_path)
                                .file_name()
                                .and_then(|f| f.to_str())
                                .unwrap_or("...");
                            if ui
                                .button(egui::RichText::new(short_path).size(10.5 * self.zoom))
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
                        tolerance: _,
                    } => {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Img:")
                                    .size(10.5 * self.zoom)
                                    .color(colors::TEXT_MUTED),
                            );
                            let short_path = std::path::Path::new(image_path)
                                .file_name()
                                .and_then(|f| f.to_str())
                                .unwrap_or("...");
                            if ui
                                .button(egui::RichText::new(short_path).size(10.5 * self.zoom))
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
                                egui::RichText::new("Max ms:")
                                    .size(10.5 * self.zoom)
                                    .color(colors::TEXT_MUTED),
                            );
                            ui.add(
                                egui::DragValue::new(timeout_ms)
                                    .range(100..=120000)
                                    .speed(100),
                            );
                        });
                    }
                    BlueprintNodeType::Delay { delay_ms } => {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Délai (ms):")
                                    .size(11.0 * self.zoom)
                                    .color(colors::TEXT_MUTED),
                            );
                            ui.add(egui::DragValue::new(delay_ms).range(10..=60000).speed(50));
                        });
                    }
                    BlueprintNodeType::Loop { count } => {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Nb (0=inf):")
                                    .size(11.0 * self.zoom)
                                    .color(colors::TEXT_MUTED),
                            );
                            ui.add(egui::DragValue::new(count).range(0..=1000).speed(1));
                        });
                    }
                    _ => {}
                },
            );
        }

        delete_clicked
    }
}
