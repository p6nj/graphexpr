use core::f32;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use egui::ImageSource;
use egui_dialogs::Dialogs;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;
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
            expr: "a % b == 0".to_string(),
            points: 150,
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
            ui.heading("GraphExpr");
            ui.collapsing("About the tool", |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label("GraphExpr is a tool to draw graphs from expressions.\nThe graphs generated are made of a custom amount of points all evenly scattered on an invisible circle. One point ");
                    ui.monospace("a");
                    ui.label(" is linked to the other ");
                    ui.monospace("b");
                    ui.label(" if the expression given is true for them. For example, given ");
                    ui.monospace("a % b == 0");
                    ui.label(", point 15 will be linked to point 5 because 15 is a multiple of 5 and so ");
                    ui.monospace("15 % 5 == 0");
                    ui.label(" is evaluated to be true.\nBecause the expression actually returns a real number, any expression which evaluates to a non-zero value is considered as true. For example, ");
                    ui.monospace("7");
                    ui.label(", ");
                    ui.monospace("a / 0");
                    ui.label(" or ");
                    ui.monospace("a");
                    ui.label(" will always be true (the first point is '1').\n\nThis app uses the ");
                    ui.monospace("fasteval");
                    ui.spacing_mut().item_spacing.x = 0f32;
                    ui.label("library. To know which symbols your expression can contain, check out the documentation ");
                    ui.hyperlink_to("here", "https://docs.rs/fasteval/0.2");
                    ui.label(".");
                });
            });

            ui.add_space(12.0);

            ui.horizontal(|ui| {
                let label = ui.label("Expression: ").id;
                ui.add(
                    egui::TextEdit::multiline(&mut self.expr)
                        .code_editor()
                        .desired_rows(1)
                        .desired_width(f32::INFINITY),
                )
                .labelled_by(label);
            });
            ui.collapsing("Options", |ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut self.points)
                            .speed(1f32)
                            .range(1f32..=f32::MAX)
                            .fixed_decimals(0),
                    )
                    .labelled_by(ui.label("Number of points").id)
                    .on_hover_text("Number of points on the invisible circle. Make it huge and watch your computer burn!");
                });
                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut self.stroke)
                            .speed(0.001)
                            .range(0f32..=20f32),
                    )
                    .labelled_by(ui.label("Stroke width").id)
                    .on_hover_text("Width of each line. You want this proportional to the number of points so it's not filling everything but you can still see the graph.");
                });
            });

            ui.add_space(12.0);

            ui.horizontal(|ui| {
                if ui.button("Preview").clicked() {
                    match super::path::graph(&self.expr, self.points) {
                        Ok(path_data) => {
                            self.svg_path = self
                                .svg_path
                                .clone()
                                .set("stroke-width", self.stroke)
                                .set("d", path_data);
                            self.reload_image = true;
                        }
                        Err(e) => self.dialogs.error(
                            "Parsing error :O",
                            format!("Your expression doesn't look right: {e}"),
                        ),
                    };
                }
                if ui.button("Save").clicked() {
                    match super::path::graph(&self.expr, self.points) {
                        Ok(path_data) => {
                            let document = svg::Document::new()
                                .set("viewBox", (0, 0, 1000, 1000))
                                .set(
                                    "style",
                                    match ctx.theme() {
                                        egui::Theme::Dark => "background-color: black",
                                        egui::Theme::Light => "background-color: white",
                                    },
                                )
                                .add(
                                    self.svg_path
                                        .clone()
                                        .set("stroke-width", self.stroke)
                                        .set("d", path_data),
                                );
                            #[cfg(target_arch = "wasm32")]
                            download(document.to_string().into_bytes());
                            #[cfg(not(target_arch = "wasm32"))]
                            if let Some(path) =
                                FileDialog::new().set_file_name("graph.svg").save_file()
                            {
                                if let Err(e) = svg::save(path.clone(), &document) {
                                    self.dialogs.error(
                                        "I/O Error :(",
                                        format!(
                                            "Here's what your computer has to say about it: {:?}",
                                            e
                                        ),
                                    );
                                }
                                self.last_save_path = Some(path);
                            }
                        }
                        Err(e) => self.dialogs.error(
                            "Parsing error :O",
                            format!("Your expression doesn't look right: {e}"),
                        ),
                    };
                }
            });
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
