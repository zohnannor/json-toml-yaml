use std::convert::identity;

use crate::highlighter::highlight;
use eframe::{
    egui::{self, Id, RichText, TextEdit},
    emath::Align2,
    epaint::Color32,
    epi,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
pub struct Converter {
    json: String,
    toml: String,
    yaml: String,
    limitations_window_open: bool,
}

impl Converter {
    #[must_use]
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            let mut slf: Self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default();
            slf.limitations_window_open = false;
            return slf;
        }
        Self {
            json: String::new(),
            toml: String::new(),
            yaml: String::new(),
            limitations_window_open: false,
        }
    }
}

fn code_input<'a>(code: &'a mut String, lang: &'a str) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| {
        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job = highlight(ui.ctx(), string, lang);
            layout_job.wrap_width = wrap_width;
            ui.fonts().layout_job(layout_job)
        };

        ui.vertical_centered_justified(|ui| {
            ui.label(lang.to_uppercase());
        });
        ui.add_sized(ui.available_size(), |ui: &mut egui::Ui| {
            egui::ScrollArea::vertical()
                .id_source(lang)
                .show(ui, |ui| {
                    ui.add(
                        TextEdit::multiline(code)
                            .code_editor()
                            .hint_text(lang)
                            .id(Id::new(lang))
                            .layouter(&mut layouter),
                    )
                })
                .inner
        })
    }
}

impl epi::App for Converter {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn persist_egui_memory(&self) -> bool {
        false
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        let Self {
            json,
            toml,
            yaml,
            limitations_window_open,
        } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            if ui
                .button(RichText::new("ℹ Limitations").color(Color32::YELLOW))
                .clicked()
            {
                *limitations_window_open = true;
            }

            if *limitations_window_open {
                limitations_window(ctx, limitations_window_open);
            }

            ui.columns(3, |ui| {
                ui[0].add(code_input(json, "json"));
                ui[1].add(code_input(toml, "toml"));
                ui[2].add(code_input(yaml, "yaml"));
            });

            match ui.memory().focus() {
                Some(id) if id == Id::new("json") => {
                    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(json) {
                        match toml::to_string_pretty(&json_value) {
                            Ok(v) => *toml = v,
                            // Err(toml::ser::Error::ValueAfterTable) => {}
                            // see https://github.com/alexcrichton/toml-rs/issues/336#issuecomment-1076728961
                            Err(err) => *toml = err.to_string(),
                        }
                        *yaml = serde_yaml::to_string(&json_value)
                            .map_or_else(|err| err.to_string(), identity);
                    }
                }
                Some(id) if id == Id::new("toml") => {
                    if let Ok(toml_value) = toml::from_str::<toml::Value>(toml) {
                        *json = serde_json::to_string_pretty(&toml_value)
                            .map_or_else(|err| err.to_string(), identity);
                        *yaml = serde_yaml::to_string(&toml_value)
                            .map_or_else(|err| err.to_string(), identity);
                    }
                }
                Some(id) if id == Id::new("yaml") => {
                    if let Ok(yaml_value) = serde_yaml::from_str::<serde_yaml::Value>(yaml) {
                        match toml::to_string_pretty(&yaml_value) {
                            Ok(v) => *toml = v,
                            // Err(toml::ser::Error::ValueAfterTable) => {}
                            // see https://github.com/alexcrichton/toml-rs/issues/336#issuecomment-1076728961
                            Err(err) => *toml = err.to_string(),
                        }
                        *json = serde_json::to_string_pretty(&yaml_value)
                            .map_or_else(|err| err.to_string(), identity);
                    }
                }
                Some(_) => unreachable!(),
                None => {}
            }
        });
    }
}

fn limitations_window(ctx: &egui::Context, limitations_window_open: &mut bool) {
    egui::Window::new(RichText::new("ℹ Limitations").color(Color32::YELLOW))
        .collapsible(false)
        .resizable(false)
        .default_width(100.0)
        .anchor(Align2::CENTER_CENTER, (0., 0.))
        .scroll2([false; 2])
        .show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("- You should be careful with JSON: if you put an array value before others, TOML serializer would fail.");
                ui.label("See");
                ui.hyperlink_to(
                    "alexcrichton/toml-rs#336 (comment)",
                    "https://github.com/alexcrichton/toml-rs/issues/336#issuecomment-1076728961",
                );
            });
            ui.label("- The memory of the app is saved once every 30 seconds on the web.");
            ui.vertical_centered(|ui| {
                if ui.button("Ok").clicked() {
                    *limitations_window_open = false;
                }
            });
        });
}
