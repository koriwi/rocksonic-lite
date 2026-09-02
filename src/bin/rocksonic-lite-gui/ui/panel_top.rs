use std::{fs, io};

use eframe::egui::{self, Ui, vec2};

use crate::state::RockSonicLite;

pub fn render(ui: &mut Ui, state: &mut RockSonicLite) {
    egui::Panel::top("header").exact_size(50.0).show(ui, |ui| {
        ui.horizontal_centered(|ui| {
            ui.heading("RockSonic Lite");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add_sized(vec2(ui.min_size().x, 25.0), egui::Button::new("choose"))
                    .clicked()
                {
                    let fd = rfd::FileDialog::new();
                    state.config_path = fd
                        .set_title("Select the config file yaml")
                        .add_filter("RockSonicLite config file", &["yaml", "yml"])
                        .pick_file();
                    let Some(config_path) = state.config_path.as_ref() else {
                        return;
                    };
                    let io::Result::Ok(config_text) = fs::read_to_string(config_path) else {
                        return;
                    };
                    state.config_text = Some(config_text);
                    state.config_text_changed = state.config_text.clone();
                };
                ui.label("Choose config file:");
            });
        });
    });
}
