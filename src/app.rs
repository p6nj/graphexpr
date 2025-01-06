#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use egui::ImageSource;
use egui_dialogs::Dialogs;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(
    inline_js = "export function download(bytes) { window.open(URL.createObjectURL(new Blob([bytes], { type: 'image/svg' })), '_blank').focus(); }"
)]
extern "C" {
    fn download(bytes: Vec<u8>) -> u32;
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct GraphExpr<'a> {
    #[serde(skip)]
    dialogs: Dialogs<'a>,
    expr: String,
    points: u32,
    stroke: f32,
    #[serde(skip)]
    svg_path: svg::node::element::Path,
    #[cfg(not(target_arch = "wasm32"))]
    last_save_path: Option<PathBuf>,
    #[serde(skip)]
    dumb_counter: u32,
    #[serde(skip)]
    reload_image: bool,
}

impl<'a> Default for GraphExpr<'a> {
    fn default() -> Self {
        GraphExpr {
            dialogs: Default::default(),
            svg_path: svg::node::element::Path::new()
                .set("stroke", "black")
                .set("fill", "none")
                .set("stroke-width", 3),
            expr: "true".to_string(),
            points: 50,
            stroke: 3f32,
            #[cfg(not(target_arch = "wasm32"))]
            last_save_path: None,
            dumb_counter: 0,
            reload_image: false,
        }
    }
}

impl<'a> GraphExpr<'a> {
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

impl<'a> eframe::App for GraphExpr<'a> {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui_extras::install_image_loaders(ctx);
        self.dialogs.show(ctx);

        if self.svg_path.get_attributes()["stroke"]
            != match ctx.theme() {
                egui::Theme::Dark => "white",
                egui::Theme::Light => "black",
            }
        {
            self.svg_path = self.svg_path.clone().set(
                "stroke",
                match ctx.theme() {
                    egui::Theme::Dark => "white",
                    egui::Theme::Light => "black",
                },
            );
            self.reload_image = true;
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                egui::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let label = ui.label("Expression: ");
                ui.text_edit_singleline(&mut self.expr)
                    .labelled_by(label.id);
            });
            ui.horizontal(|ui| {
                ui.label("Number of points: ");
                ui.add(
                    egui::DragValue::new(&mut self.points)
                        .speed(1f32)
                        .range(1f32..=f32::MAX)
                        .fixed_decimals(0),
                );
            });
            ui.horizontal(|ui| {
                ui.label("Stroke width: ");
                ui.add(
                    egui::DragValue::new(&mut self.stroke)
                        .speed(0.001)
                        .range(0f32..=20f32),
                );
            });
            if ui.button("Preview").clicked() {
                self.svg_path = self
                    .svg_path
                    .clone()
                    .set("stroke-width", self.stroke)
                    .set("d", super::path::sample(self.points));
                // self.dialogs.info("OK", "It's ok!!!");
                self.reload_image = true;
            }
            if ui.button("Save").clicked() {
                #[cfg(target_arch = "wasm32")]
                download(
                    svg::Document::new()
                        .set("viewBox", (0, 0, 1000, 1000))
                        .set(
                            "style",
                            format!(
                                "background-color: {}",
                                match ctx.theme() {
                                    egui::Theme::Dark => "black",
                                    egui::Theme::Light => "white",
                                }
                            ),
                        )
                        .add(
                            self.svg_path
                                .clone()
                                .set("stroke-width", self.stroke)
                                .set("d", super::path::sample(self.points)),
                        )
                        .to_string()
                        .into_bytes(),
                );
            }
            if self.reload_image {
                self.dumb_counter = self.dumb_counter.wrapping_add(1);
                self.reload_image = !self.reload_image;
            }
            ui.add(egui::Image::new(ImageSource::from((
                format!("bytes://graph{}.svg", self.dumb_counter),
                svg::Document::new()
                    .set("viewBox", (0, 0, 1000, 1000))
                    .add(self.svg_path.clone())
                    .to_string()
                    .into_bytes(),
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
