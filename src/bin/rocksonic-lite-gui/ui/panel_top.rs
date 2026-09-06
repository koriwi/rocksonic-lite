use std::{fs, io};

use eframe::egui::{self, Align, Ui, vec2};
use rocksonic_lite::config::Config;

use crate::state::{ConfigStruct, RockSonicLite};

fn load_config(state: &mut RockSonicLite) {
    let fd = rfd::FileDialog::new();
    let new_config_path = fd
        .set_title("Select the config file yaml")
        .add_filter("RockSonicLite config file", &["yaml", "yml"])
        .pick_file();
    let Some(config_path) = new_config_path.as_ref() else {
        return;
    };
    let io::Result::Ok(config_text) = fs::read_to_string(config_path) else {
        return;
    };
    let Ok(config) = Config::from_path(config_path) else {
        return;
    };
    state.config = Some(ConfigStruct {
        config,
        path: config_path.clone(),
        text: config_text.clone(),
        text_changed: config_text,
        save_needed: false,
    });
}

pub fn render(ui: &mut Ui, state: &mut RockSonicLite) {
    egui::Panel::top("header").exact_size(50.0).show(ui, |ui| {
        ui.columns(3, |cols| {
            cols[0].horizontal_centered(|ui| {
                ui.heading("RockSonic Lite");
            });
            cols[1].centered_and_justified(|ui| {
                ui.label(
                    state
                        .config
                        .as_ref()
                        .map_or("no config file loaded", |config| {
                            config.path.to_str().as_ref().unwrap()
                        }),
                );
            });
            cols[2].with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                // ui.horizontal_centered(|ui| {
                if ui
                    .add_sized(vec2(ui.min_size().x, 25.0), egui::Button::new("open"))
                    .clicked()
                {
                    load_config(state);
                };
                ui.label("Config file");
                // });
            });
        });
    });
}
