use eframe::{
    egui::{self, Id, TextEdit},
    epi,
};

use crate::highlighter::highlight;

pub struct Converter {
    json: String,
    toml: String,
    yaml: String,
}

impl Converter {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            json: String::new(),
            toml: String::new(),
            yaml: String::new(),
        }
    }
}

impl Default for Converter {
    fn default() -> Self {
        Self::new()
    }
}

fn code_input(ui: &mut egui::Ui, code: &mut String, lang: &str) {
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
    });
}

impl epi::App for Converter {
    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        let Self { json, toml, yaml } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(3, |ui| {
                code_input(&mut ui[0], json, "json");
                code_input(&mut ui[1], toml, "toml");
                code_input(&mut ui[2], yaml, "yaml");
            });

            if ui.memory().has_focus(Id::new("json")) {
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(json) {
                    match toml::to_string_pretty(&json_value) {
                        Ok(v) => {
                            *toml = v;
                        }
                        // Err(toml::ser::Error::ValueAfterTable) => {}
                        // see https://github.com/alexcrichton/toml-rs/issues/336#issuecomment-1076728961
                        Err(err) => *toml = err.to_string(),
                    }
                    *yaml = serde_yaml::to_string(&json_value)
                        .map_or_else(|err| err.to_string(), |v| v);
                }
            }

            if ui.memory().has_focus(Id::new("toml")) {
                if let Ok(toml_value) = toml::from_str::<toml::Value>(toml) {
                    *json = serde_json::to_string_pretty(&toml_value)
                        .map_or_else(|err| err.to_string(), |v| v);
                    *yaml = serde_yaml::to_string(&toml_value)
                        .map_or_else(|err| err.to_string(), |v| v);
                }
            }

            if ui.memory().has_focus(Id::new("yaml")) {
                if let Ok(yaml_value) = serde_yaml::from_str::<serde_yaml::Value>(yaml) {
                    match toml::to_string_pretty(&yaml_value) {
                        Ok(v) => {
                            *toml = v;
                        }
                        // Err(toml::ser::Error::ValueAfterTable) => {}
                        // see https://github.com/alexcrichton/toml-rs/issues/336#issuecomment-1076728961
                        Err(err) => *toml = err.to_string(),
                    }
                    *json = serde_json::to_string_pretty(&yaml_value)
                        .map_or_else(|err| err.to_string(), |v| v);
                }
            }
        });
    }
}
