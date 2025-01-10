use core::f32;
#[cfg(target_arch = "wasm32")]
use std::collections::BTreeMap;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
use std::str::FromStr;

#[cfg(target_arch = "wasm32")]
use cached::proc_macro::cached;
use egui::ImageSource;
use egui_dialogs::Dialogs;
#[cfg(target_arch = "wasm32")]
use fasteval::{Compiler, Evaler};
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;
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
    #[serde(skip)]
    locale: Locale,
}

#[derive(Default)]
enum Locale {
    French,
    #[default]
    English,
}

impl FromStr for Locale {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.contains("fr")
            .then_some(Locale::French)
            .or_else(|| s.contains("en").then_some(Locale::English))
            .ok_or(())
    }
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
            locale: get_locale()
                .and_then(|s| s.parse().ok())
                .unwrap_or_default(),
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
                ui.horizontal_wrapped(|ui| match self.locale {
                    Locale::French => {
                        ui.label(include_str!("locale/fr/info1.txt"));
                        ui.monospace("a");
                        ui.label(include_str!("locale/fr/info2.txt"));
                        ui.monospace("b");
                        ui.label(include_str!("locale/fr/info3.txt"));
                        ui.monospace("a % b == 0");
                        ui.label(include_str!("locale/fr/info4.txt"));
                        ui.monospace("15 % 5 == 0");
                        ui.label(include_str!("locale/fr/info5.txt"));
                        ui.monospace("7");
                        ui.label(", ");
                        ui.monospace("a / 0");
                        ui.label(include_str!("locale/fr/info6.txt"));
                        ui.monospace("a");
                        ui.label(include_str!("locale/fr/info7.txt"));
                        ui.monospace("fasteval");
                        ui.label(include_str!("locale/fr/info8.txt"));
                        ui.hyperlink_to(
                            include_str!("locale/fr/info9.txt"),
                            "https://docs.rs/fasteval/0.2",
                        );
                        ui.label(".");
                    }
                    Locale::English => {
                        ui.label(include_str!("locale/en/info1.txt"));
                        ui.monospace("a");
                        ui.label(include_str!("locale/en/info2.txt"));
                        ui.monospace("b");
                        ui.label(include_str!("locale/en/info3.txt"));
                        ui.monospace("a % b == 0");
                        ui.label(include_str!("locale/en/info4.txt"));
                        ui.monospace("15 % 5 == 0");
                        ui.label(include_str!("locale/en/info5.txt"));
                        ui.monospace("7");
                        ui.label(", ");
                        ui.monospace("a / 0");
                        ui.label(include_str!("locale/en/info6.txt"));
                        ui.monospace("a");
                        ui.label(include_str!("locale/en/info7.txt"));
                        ui.monospace("fasteval");
                        ui.label(include_str!("locale/en/info8.txt"));
                        ui.hyperlink_to(
                            include_str!("locale/en/info9.txt"),
                            "https://docs.rs/fasteval/0.2",
                        );
                        ui.label(".");
                    }
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
                    .labelled_by(
                        ui.label(match self.locale {
                            Locale::English => include_str!("locale/en/options1.txt"),
                            Locale::French => include_str!("locale/fr/options1.txt"),
                        })
                        .id,
                    )
                    .on_hover_text(match self.locale {
                        Locale::English => include_str!("locale/en/options1h.txt"),
                        Locale::French => include_str!("locale/fr/options1h.txt"),
                    });
                });
                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut self.stroke)
                            .speed(0.001)
                            .range(0f32..=20f32),
                    )
                    .labelled_by(
                        ui.label(match self.locale {
                            Locale::English => include_str!("locale/en/options2.txt"),
                            Locale::French => include_str!("locale/fr/options2.txt"),
                        })
                        .id,
                    )
                    .on_hover_text(match self.locale {
                        Locale::English => include_str!("locale/en/options2h.txt"),
                        Locale::French => include_str!("locale/fr/options2h.txt"),
                    });
                });
            });

            ui.add_space(12.0);

            ui.horizontal(|ui| {
                if ui
                    .button(match self.locale {
                        Locale::English => include_str!("locale/en/action1.txt"),
                        Locale::French => include_str!("locale/fr/action1.txt"),
                    })
                    .on_hover_text(match self.locale {
                        Locale::English => include_str!("locale/en/action1h.txt"),
                        Locale::French => include_str!("locale/fr/action1h.txt"),
                    })
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
                            match self.locale {
                                Locale::English => include_str!("locale/en/error1t.txt"),
                                Locale::French => include_str!("locale/fr/error1t.txt"),
                            },
                            format!(
                                "{}: {}",
                                match self.locale {
                                    Locale::English => include_str!("locale/en/error1.txt"),
                                    Locale::French => include_str!("locale/fr/error1.txt"),
                                },
                                e
                            ),
                        ),
                    };
                }
                if ui
                    .button(match self.locale {
                        Locale::English => include_str!("locale/en/action2.txt"),
                        Locale::French => include_str!("locale/fr/action2.txt"),
                    })
                    .on_hover_text(match self.locale {
                        Locale::English => include_str!("locale/en/action2h.txt"),
                        Locale::French => include_str!("locale/fr/action2h.txt"),
                    })
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
                                        match self.locale {
                                            Locale::English => {
                                                include_str!("locale/en/error2t.txt")
                                            }
                                            Locale::French => include_str!("locale/fr/error2t.txt"),
                                        },
                                        format!(
                                            "{}: {:?}",
                                            match self.locale {
                                                Locale::English =>
                                                    include_str!("locale/en/error2.txt"),
                                                Locale::French =>
                                                    include_str!("locale/fr/error2.txt"),
                                            },
                                            e
                                        ),
                                    );
                                }
                            }
                        }
                        Err(e) => self.dialogs.error(
                            match self.locale {
                                Locale::English => include_str!("locale/en/error1t.txt"),
                                Locale::French => include_str!("locale/fr/error1t.txt"),
                            },
                            format!(
                                "{}: {}",
                                match self.locale {
                                    Locale::English => include_str!("locale/en/error1.txt"),
                                    Locale::French => include_str!("locale/fr/error1.txt"),
                                },
                                e
                            ),
                        ),
                    };
                }
                #[cfg(target_arch = "wasm32")]
                ui.label(format!(
                    "{} {}",
                    match self.locale {
                        Locale::English => include_str!("locale/en/exprstat.txt"),
                        Locale::French => include_str!("locale/fr/exprstat.txt"),
                    },
                    match expression_status(self.expr.clone()) {
                        Some(_) => "valid",
                        None => "invalid",
                    }
                ));
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
                powered_by_egui_and_eframe(ui, &self.locale);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

#[cfg(target_arch = "wasm32")]
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

fn powered_by_egui_and_eframe(ui: &mut egui::Ui, locale: &Locale) {
    match locale {
        Locale::English => {
            #[cfg(target_arch = "wasm32")]
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label(include_str!("locale/en/github1.txt"));
                ui.hyperlink_to(
                    "GitHub",
                    "https://github.com/p6nj/graphexpr/releases/latest",
                );
                ui.label(include_str!("locale/en/github2.txt"));
            });
            ui.label(include_str!("locale/en/note.txt"));
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label(include_str!("locale/en/libs1.txt"));
                ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                ui.label(include_str!("locale/en/libs2.txt"));
                ui.hyperlink_to(
                    "eframe",
                    "https://github.com/emilk/egui/tree/master/crates/eframe",
                );
                ui.label(".");
            });
        }
        Locale::French => {
            #[cfg(target_arch = "wasm32")]
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label(include_str!("locale/fr/github1.txt"));
                ui.hyperlink_to(
                    "GitHub",
                    "https://github.com/p6nj/graphexpr/releases/latest",
                );
                ui.label(include_str!("locale/fr/github2.txt"));
            });
            ui.label(include_str!("locale/fr/note.txt"));
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label(include_str!("locale/fr/libs1.txt"));
                ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                ui.label(include_str!("locale/fr/libs2.txt"));
                ui.hyperlink_to(
                    "eframe",
                    "https://github.com/emilk/egui/tree/master/crates/eframe",
                );
                ui.label(".");
            });
        }
    }
}
