#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use egui::ImageSource;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct GraphExpr {
    expr: String,
    #[serde(skip)]
    svg_path: svg::node::element::Path,
    #[cfg(not(target_arch = "wasm32"))]
    last_save_path: Option<PathBuf>,
    #[serde(skip)]
    funny_bool: bool,
}

impl Default for GraphExpr {
    fn default() -> Self {
        GraphExpr {
            svg_path: svg::node::element::Path::new()
                .set("fill", "none")
                .set("stroke-width", 3),
            expr: Default::default(),
            #[cfg(not(target_arch = "wasm32"))]
            last_save_path: Default::default(),
            funny_bool: Default::default(),
        }
    }
}

impl GraphExpr {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

fn theme_to_stroke_color(theme: egui::Theme) -> &'static str {
    match theme {
        egui::Theme::Dark => "white",
        egui::Theme::Light => "black",
    }
}

impl eframe::App for GraphExpr {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui_extras::install_image_loaders(ctx);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                ui.menu_button("File", |ui| {
                    if !is_web {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        ui.add_space(16.0);
                    }
                });
                egui::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Expression: ");
                ui.text_edit_singleline(&mut self.expr);
                if ui.button("Go!").clicked() {
                    self.svg_path = self.svg_path.clone().set(
                        "d",
                        svg::node::element::path::Data::new()
                            .move_to((10, 10))
                            .line_by((0, 50))
                            .line_by((50, 0))
                            .line_by((0, -50))
                            .close(),
                    );

                    self.funny_bool = !self.funny_bool;
                }
            });
            ui.add(egui::Image::new(ImageSource::from((
                match (ctx.theme(), self.funny_bool) {
                    (egui::Theme::Dark, false) => "bytes://graph-dark.svg",
                    (egui::Theme::Light, false) => "bytes://graph.svg",
                    (egui::Theme::Dark, true) => "bytes://plot-dark.svg",
                    (egui::Theme::Light, true) => "bytes://plot.svg",
                },
                {
                    let d = svg::Document::new().set("viewBox", (0, 0, 70, 70)).add(
                        self.svg_path
                            .clone()
                            .set("stroke", theme_to_stroke_color(ctx.theme())),
                    );
                    println!("{}", d);
                    d.to_string().into_bytes()
                },
            ))));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
