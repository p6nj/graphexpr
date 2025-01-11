use core::f32;
use std::collections::BTreeMap;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use cached::proc_macro::cached;
use egui::ImageSource;
use egui_dialogs::Dialogs;
use fasteval::{Compiler, Evaler};
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;
use rust_i18n::t;
use sys_locale::get_locale;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(inline_js = "export function download(bytes) { \
                            window.open(URL.createObjectURL(new Blob([bytes], { type: \
                            'image/svg' })), '_blank').focus(); }")]
extern "C" {
    fn download(bytes: Vec<u8>);
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct GraphExpr<'a> {
    #[serde(skip)]
    dialogs: Dialogs<'a>,
    expr: String,
    points: u16,
    stroke: f32,
    #[serde(skip)]
    svg_path: svg::node::element::Path,
    #[cfg(not(target_arch = "wasm32"))]
    last_save_path: Option<PathBuf>,
    #[serde(skip)]
    dumb_counter: u16,
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

        if let Some(locale) = get_locale() {
            rust_i18n::set_locale(&locale);
        }

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
            ui.collapsing(t!("info.title"), |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(t!("info.1"));
                    ui.monospace("a");
                    ui.label(t!("info.2"));
                    ui.monospace("b");
                    ui.label(t!("info.3"));
                    ui.monospace("a % b == 0");
                    ui.label(t!("info.4"));
                    ui.monospace("15 % 5 == 0");
                    ui.label(t!("info.5"));
                    ui.monospace("7");
                    ui.label(", ");
                    ui.monospace("a / 0");
                    ui.label(t!("info.6"));
                    ui.monospace("a");
                    ui.label(t!("info.7"));
                    ui.monospace("fasteval");
                    ui.label(t!("info.8"));
                    ui.hyperlink_to(t!("info.9"), "https://docs.rs/fasteval/0.2");
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
                    .labelled_by(ui.label(t!("option.points.title")).id)
                    .on_hover_text(t!("option.points.hover"));
                });
                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut self.stroke)
                            .speed(0.001)
                            .range(0f32..=20f32),
                    )
                    .labelled_by(ui.label(t!("option.stroke.title")).id)
                    .on_hover_text(t!("option.stroke.title"));
                });
            });

            ui.add_space(12.0);

            ui.horizontal(|ui| {
                if ui
                    .button(t!("action.preview.title"))
                    .on_hover_text(t!("action.preview.hover"))
                    .clicked()
                {
                    match super::path::graph(self.expr.clone(), self.points) {
                        Ok(path_data) => {
                            self.svg_path = self
                                .svg_path
                                .clone()
                                .set("stroke-width", self.stroke)
                                .set("d", path_data);
                            self.reload_image = true;
                        }
                        Err(e) => self.dialogs.error(
                            t!("error.parsing.title"),
                            t!("error.parsing.body", error = e),
                        ),
                    };
                }
                if ui
                    .button(t!("action.save.title"))
                    .on_hover_text(t!("action.save.hover"))
                    .clicked()
                {
                    match super::path::graph(self.expr.clone(), self.points) {
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
                            if let Some(path) = {
                                let dialog = FileDialog::new().set_file_name("graph.svg");
                                if let Some(ref path) = self.last_save_path {
                                    dialog.set_directory(path)
                                } else {
                                    dialog
                                }
                            }
                            .save_file()
                            {
                                if let Some(folder) = path.parent() {
                                    self.last_save_path = Some(folder.to_path_buf());
                                }
                                if let Err(e) = svg::save(path, &document) {
                                    self.dialogs.error(
                                        t!("error.io.title"),
                                        t!("error.io.body", error = e),
                                    );
                                }
                            }
                        }
                        Err(e) => self.dialogs.error(
                            t!("error.parsing.title"),
                            t!("error.parsing.body", error = e),
                        ),
                    };
                }
                ui.label({
                    let status = match expression_status(self.expr.clone()) {
                        Some(_) => t!("exprstat.valid"),
                        None => t!("exprstat.invalid"),
                    };
                    t!("exprstat.body", status = status)
                });
            });
            if self.reload_image {
                self.dumb_counter = self.dumb_counter.wrapping_add(1);
                self.reload_image = !self.reload_image;
            }
            ui.add({
                let image = egui::Image::new(ImageSource::from((
                    format!("bytes://graph{}.svg", self.dumb_counter),
                    svg::Document::new()
                        .set("viewBox", (0, 0, 1000, 1000))
                        .add(self.svg_path.clone())
                        .to_string()
                        .into_bytes(),
                )));
                if cfg!(target_arch = "wasm32") {
                    image.max_size([4096.0, 4096.0].into())
                } else {
                    image
                }
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

#[cached]
fn expression_status(expr: String) -> Option<()> {
    let mut slab = fasteval::Slab::new();
    let compiled = fasteval::Parser::new()
        .parse(&expr, &mut slab.ps)
        .ok()?
        .from(&slab.ps)
        .compile(&slab.ps, &mut slab.cs);
    let mut map = BTreeMap::from([("a", 1f64), ("b", 2f64)]);
    let _ = compiled.eval(&slab, &mut map).ok()?;
    Some(())
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    #[cfg(target_arch = "wasm32")]
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label(t!("locale/en/github1"));
        ui.hyperlink_to(
            "GitHub",
            "https://github.com/p6nj/graphexpr/releases/latest",
        );
        ui.label(t!("locale/en/github2"));
    });
    ui.label(t!("note"));
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label(t!("libs.1"));
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(t!("libs.2"));
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
