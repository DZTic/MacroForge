use crate::macro_core::{self, ImageTestResult};
use crate::ui::i18n::Language;
use crate::ui::theme::{self, colors};
use crate::ui::widgets::{ButtonVariant, GlassButton};
use eframe::egui::{self, Frame, Margin, Rounding, Stroke, Vec2};

/// État du test de recherche d'image
#[derive(Debug, Clone)]
pub enum ImageTestState {
    /// Aucun test lancé
    Idle,
    /// Test en cours (compteur de tentatives)
    Scanning(u32),
    /// Dernier résultat
    Done(Box<ImageTestResult>),
    /// Erreur du worker (ex : thread impossible à lancer)
    Failed(String),
}

impl PartialEq for ImageTestState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ImageTestState::Idle, ImageTestState::Idle) => true,
            (ImageTestState::Scanning(a), ImageTestState::Scanning(b)) => a == b,
            (ImageTestState::Failed(a), ImageTestState::Failed(b)) => a == b,
            (ImageTestState::Done(a), ImageTestState::Done(b)) => {
                // Comparaison superficielle : verdict + position, sans les buffers
                a.found == b.found && a.pos == b.pos && a.error == b.error
            }
            _ => false,
        }
    }
}

impl Eq for ImageTestState {}

impl ImageTestState {
    /// Référence au dernier résultat si l'état est Done.
    fn result_ref(&self) -> Option<&ImageTestResult> {
        match self {
            ImageTestState::Done(res) => Some(res),
            _ => None,
        }
    }
}

pub struct ImageTestModal {
    pub is_open: bool,
    /// Image testée
    pub image_path: String,
    pub tolerance: u8,
    pub state: ImageTestState,
    /// Nombre de tests réussis / total pour le bilan
    pub stats_ok: u32,
    pub stats_total: u32,
    /// Réceptacle du résultat du worker : Some tant que le scan est en cours
    scan_receiver: Option<std::sync::mpsc::Receiver<ImageTestResult>>,
    /// Aperçu de la capture mise en texture (construit une fois par résultat)
    preview: Option<(egui::TextureHandle, [f32; 2])>,
}

impl Default for ImageTestModal {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageTestModal {
    pub fn new() -> Self {
        Self {
            is_open: false,
            image_path: String::new(),
            tolerance: 25,
            state: ImageTestState::Idle,
            stats_ok: 0,
            stats_total: 0,
            scan_receiver: None,
            preview: None,
        }
    }

    /// Ouvre le modal pré-rempli pour un nœud ImageCondition / WaitImage.
    pub fn open_for(&mut self, image_path: &str, tolerance: u8) {
        self.image_path = image_path.to_string();
        self.tolerance = tolerance;
        self.state = ImageTestState::Idle;
        self.preview = None;
        self.is_open = true;
    }

    /// Lance un test asynchrone : remonte le jeu au premier plan (comme en
    /// exécution), attend un court instant pour laisser le rendu se stabiliser,
    /// capture puis recherche le template. Le résultat est livré via le canal ;
    /// l'UI reste réactive et affiche l'état "Analyse...".
    fn run_test(&mut self) {
        if self.scan_receiver.is_some() {
            return; // Un scan est déjà en cours
        }
        let path = self.image_path.clone();
        let tolerance = self.tolerance;
        self.state = ImageTestState::Scanning(0);

        let (tx, rx) = std::sync::mpsc::channel();
        self.scan_receiver = Some(rx);

        std::thread::spawn(move || {
            // Même séquence que le runner Blueprint : jeu au premier plan,
            // stabilisation de l'affichage, puis capture + recherche.
            macro_core::bring_game_to_foreground();
            std::thread::sleep(std::time::Duration::from_millis(200));
            let res = macro_core::test_image_search(&path, tolerance);
            let _ = tx.send(res);
        });
    }

    /// Récupère le résultat du worker si disponible.
    fn poll_scan_result(&mut self) {
        let Some(rx) = self.scan_receiver.take() else {
            return;
        };
        match rx.try_recv() {
            Ok(res) => {
                self.stats_total += 1;
                if res.found {
                    self.stats_ok += 1;
                }
                self.preview = None; // nouvel aperçu à reconstruire
                self.state = ImageTestState::Done(Box::new(res));
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                // Toujours en cours : reposer le réceptacle et re-vérifier au prochain frame
                self.scan_receiver = Some(rx);
                if let ImageTestState::Scanning(n) = self.state {
                    self.state = ImageTestState::Scanning(n + 1);
                } else {
                    self.state = ImageTestState::Scanning(0);
                }
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.stats_total += 1;
                self.state = ImageTestState::Failed(
                    "Le thread de test d'image s'est terminé sans résultat.".to_string(),
                );
            }
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, lang: Language) -> bool {
        if !self.is_open {
            return false;
        }

        // Récupérer le résultat du worker dès qu'il est prêt
        self.poll_scan_result();
        if self.scan_receiver.is_some() {
            // Scan en cours : redessiner continuellement pour la spinner
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }

        // Construire l'aperçu de la capture une seule fois par nouveau
        // résultat (la conversion BGRA->texture est coûteuse : plusieurs Mo)
        if matches!(self.state, ImageTestState::Done(_)) && self.preview.is_none() {
            if let Some(res) = self.state.result_ref() {
                let capture_bgra: &[u8] = res
                    .capture_small
                    .as_deref()
                    .unwrap_or(res.capture.as_slice());
                let (cw, ch) = if res.capture_small.is_some() {
                    (res.capture_width / 2, res.capture_height / 2)
                } else {
                    (res.capture_width, res.capture_height)
                };
                self.preview =
                    macro_core::bgra_capture_to_egui_texture(ctx, capture_bgra, cw, ch, 560);
            }
        }

        let mut request_close = false;

        egui::Window::new(lang.image_test_modal_title())
            .frame(theme::modal_frame())
            .collapsible(false)
            .resizable(false)
            .default_size(Vec2::new(720.0, 560.0))
            .anchor(egui::Align2::CENTER_CENTER, Vec2::new(0.0, 0.0))
            .show(ctx, |ui| {
                ui.add_space(2.0);

                // Informations sur l'image testée
                Frame::none()
                    .fill(colors::BG_CARD)
                    .stroke(Stroke::new(1.0_f32, colors::BORDER_CARD))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(lang.image_test_image_label())
                                    .color(colors::TEXT_PRIMARY)
                                    .size(13.0),
                            );
                            let name = std::path::Path::new(&self.image_path)
                                .file_name()
                                .and_then(|f| f.to_str())
                                .unwrap_or(&self.image_path);
                            ui.label(
                                egui::RichText::new(name)
                                    .color(colors::TEXT_SECONDARY)
                                    .size(12.5),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(lang.image_test_tolerance_label())
                                    .color(colors::TEXT_PRIMARY)
                                    .size(13.0),
                            );
                            ui.add(
                                egui::DragValue::new(&mut self.tolerance)
                                    .range(0..=100)
                                    .speed(1.0),
                            );

                            let test_btn = GlassButton::new(lang.image_test_run_btn())
                                .icon("🔍")
                                .variant(ButtonVariant::Primary);
                            if ui
                                .add(test_btn)
                                .on_hover_text(lang.image_test_run_tooltip())
                                .clicked()
                            {
                                self.run_test();
                            }

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let verdict_label = format!(
                                        "{} {}/{}",
                                        lang.image_test_stats_label(),
                                        self.stats_ok,
                                        self.stats_total
                                    );
                                    ui.label(
                                        egui::RichText::new(verdict_label)
                                            .color(colors::TEXT_MUTED)
                                            .size(12.0),
                                    );
                                },
                            );
                        });
                    });

                ui.add_space(8.0);

                // Zone de résultat
                Frame::none()
                    .fill(colors::BG_CARD)
                    .stroke(Stroke::new(1.0_f32, colors::BORDER_CARD))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(Margin::same(12.0))
                    .show(ui, |ui| match &self.state {
                        ImageTestState::Idle => {
                            ui.vertical_centered(|ui| {
                                ui.add_space(30.0);
                                ui.label(
                                    egui::RichText::new(lang.image_test_idle_hint())
                                        .color(colors::TEXT_MUTED)
                                        .size(13.0),
                                );
                                ui.add_space(30.0);
                            });
                        }
                        ImageTestState::Scanning(_) => {
                            ui.vertical_centered(|ui| {
                                ui.add_space(30.0);
                                ui.label(
                                    egui::RichText::new(lang.image_test_scanning())
                                        .color(colors::TEXT_SECONDARY)
                                        .size(14.0),
                                );
                                ui.add_space(30.0);
                            });
                        }
                        ImageTestState::Failed(msg) => {
                            ui.label(
                                egui::RichText::new(format!("❌ {}", msg))
                                    .color(colors::ACCENT_DANGER)
                                    .size(13.5),
                            );
                        }
                        ImageTestState::Done(res) => {
                            self.render_result(ui, res, lang);
                        }
                    });

                ui.add_space(10.0);

                // Bouton fermer
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let close_btn =
                            GlassButton::new(lang.modal_cancel()).variant(ButtonVariant::Ghost);
                        if ui.add(close_btn).clicked() {
                            request_close = true;
                        }
                    });
                });
            });

        if request_close {
            self.is_open = false;
        }

        false
    }

    fn render_result(&self, ui: &mut egui::Ui, res: &ImageTestResult, lang: Language) {
        if let Some(err) = &res.error {
            ui.label(
                egui::RichText::new(format!("❌ {}", err))
                    .color(colors::ACCENT_DANGER)
                    .size(13.5),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(lang.image_test_error_hint())
                    .color(colors::TEXT_MUTED)
                    .size(12.0),
            );
            return;
        }

        // Verdict principal
        let (verdict, verdict_color) = if res.found {
            (lang.image_test_found(), colors::ACCENT_SUCCESS)
        } else {
            (lang.image_test_not_found(), colors::ACCENT_DANGER)
        };

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(verdict)
                    .strong()
                    .size(15.0)
                    .color(verdict_color),
            );

            if res.found {
                if let Some((x, y)) = res.pos {
                    ui.label(
                        egui::RichText::new(format!("({}, {})", x, y))
                            .monospace()
                            .color(colors::TEXT_SECONDARY)
                            .size(12.5),
                    );
                }
            }

            // Origine de la capture
            if let Some(title) = &res.capture_origin {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let lbl = format!(
                        "{} {}",
                        lang.image_test_capture_zone(),
                        truncate_chars(title, 24)
                    );
                    ui.label(
                        egui::RichText::new(lbl)
                            .color(colors::TEXT_MUTED)
                            .size(11.5),
                    );
                });
            }
        });

        if res.found {
            ui.label(
                egui::RichText::new(lang.image_test_found_advice())
                    .color(colors::TEXT_SECONDARY)
                    .size(12.0),
            );
        } else {
            ui.label(
                egui::RichText::new(lang.image_test_miss_advice())
                    .color(colors::TEXT_SECONDARY)
                    .size(12.0),
            );
        }

        ui.add_space(6.0);

        // Tailles comparées
        let mut size_line = String::new();
        if let Some((tw, th)) = res.template_size {
            size_line.push_str(&format!(
                "{} {}×{}px",
                lang.image_test_template_size(),
                tw,
                th
            ));
        }
        if !size_line.is_empty() {
            size_line.push_str(" — ");
        }
        size_line.push_str(&format!(
            "{} {}×{}px",
            lang.image_test_capture_size(),
            res.capture_width,
            res.capture_height
        ));
        ui.label(
            egui::RichText::new(size_line)
                .color(colors::TEXT_MUTED)
                .size(11.5),
        );

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(6.0);

        // Aperçu de la capture (texture construite une fois dans show) avec
        // cadre vert autour de la zone trouvée
        if let Some((texture, size)) = &self.preview {
            let avail = ui.available_size();
            let scale = (avail.x / size[0]).min(1.0).min(avail.y / size[1]);
            let img_size = egui::vec2(size[0] * scale, size[1] * scale);

            let (rect, _resp) = ui.allocate_exact_size(img_size, egui::Sense::hover());
            ui.painter().rect_filled(
                rect,
                Rounding::same(4.0),
                egui::Color32::from_rgb(18, 24, 38),
            );
            ui.painter().image(
                texture.id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );

            // Cadre vert autour de la zone détectée
            if res.found {
                if let Some((fx, fy)) = res.pos {
                    // Dimensions de la source d'aperçu (capture complète ou
                    // réduite de moitié), pour convertir les coordonnées
                    let (cw, ch) = if res.capture_small.is_some() {
                        (res.capture_width / 2, res.capture_height / 2)
                    } else {
                        (res.capture_width, res.capture_height)
                    };
                    let rel_x = (fx - res.capture_x) as f32 * (img_size.x / cw as f32);
                    let rel_y = (fy - res.capture_y) as f32 * (img_size.y / ch as f32);
                    let (tw, th) = res.template_size.unwrap_or((0, 0));
                    let box_w = tw as f32 * (img_size.x / cw as f32);
                    let box_h = th as f32 * (img_size.y / ch as f32);
                    let box_rect = egui::Rect::from_min_size(
                        rect.min + egui::vec2(rel_x, rel_y),
                        egui::vec2(box_w, box_h),
                    );
                    ui.painter().rect_stroke(
                        box_rect,
                        Rounding::same(2.0),
                        Stroke::new(3.0_f32, colors::ACCENT_SUCCESS),
                    );
                }
            }
        } else {
            ui.label(
                egui::RichText::new(lang.image_test_preview_unavailable())
                    .color(colors::TEXT_MUTED)
                    .size(12.0),
            );
        }
    }
}

fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        let truncated: String = s.chars().take(max - 1).collect();
        format!("{}…", truncated)
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_test_modal_state_machine() {
        let mut modal = ImageTestModal::new();
        assert!(!modal.is_open);
        assert_eq!(modal.state, ImageTestState::Idle);

        modal.open_for("test.png", 30);
        assert!(modal.is_open);
        assert_eq!(modal.image_path, "test.png");
        assert_eq!(modal.tolerance, 30);
        assert_eq!(modal.state, ImageTestState::Idle);
        assert_eq!(modal.stats_ok, 0);
        assert_eq!(modal.stats_total, 0);
    }

    #[test]
    fn test_truncate_chars() {
        assert_eq!(truncate_chars("court", 10), "court");
        assert_eq!(truncate_chars("tres-long-titre-ici", 9), "tres-lon…");
    }
}
