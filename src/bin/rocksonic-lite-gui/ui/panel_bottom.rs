use std::{f32::consts::TAU, fs};

use eframe::egui::{self, Color32, Image, RichText, Ui, include_image, vec2};
use rocksonic_lite::config::Config;

use crate::state::{ActiveTab, RockSonicLite, SyncButtonState};

fn save_button(ui: &mut Ui, state: &mut RockSonicLite) {
    let config_save_needed = state
        .config
        .as_ref()
        .is_some_and(|config| config.save_needed);
    let save_button = ui
        .add_enabled_ui(config_save_needed, |ui| {
            ui.add_sized(
                vec2(ui.min_size().x, 25.0),
                egui::Button::new("save config"),
            )
        })
        .inner;

    let Some(config) = state.config.as_mut() else {
        return;
    };
    if save_button.clicked() {
        fs::write(&config.path, &config.text_changed).expect("couldnt save config, sad");
        config.text = config.text_changed.clone();
        // TODO: change this to be loaded from memory first, if it succeeds, then write to file!
        config.config = Config::from_path(&config.path).unwrap();
        config.save_needed = false;
    }
}

fn sync_button(ui: &mut Ui, state: &mut RockSonicLite) {
    let sync_enabled = state
        .config
        .as_ref()
        .is_some_and(|config| !config.save_needed);
    let sync_button = match *state.sync_button_state.read().unwrap() {
        SyncButtonState::InProgress(status) => {
            let button_text = if let Some((current, total)) = status {
                format!("{current}/{total}")
            } else {
                "in progress".to_owned()
            };
            let time = ui.input(|i| i.time);
            let angle = (time % 1.0) as f32 * TAU;
            let spinner = Image::new(include_image!("../assets/spinner.png"))
                .fit_to_exact_size(vec2(24.0, 24.0))
                .rotate(angle, vec2(0.5, 0.5));
            ui.ctx().request_repaint();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_enabled_ui(false, |ui| {
                    ui.add_sized(
                        vec2(ui.min_size().x, 25.0),
                        egui::Button::image_and_text(spinner, button_text),
                    )
                })
                .inner
            })
            .inner
        }
        SyncButtonState::Idle => {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_enabled_ui(sync_enabled, |ui| {
                    ui.add_sized(vec2(ui.min_size().x, 25.0), egui::Button::new("SYNC"))
                })
                .inner
            })
            .inner
        }
        SyncButtonState::IdleDone => {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_enabled_ui(sync_enabled, |ui| {
                    ui.add_sized(vec2(ui.min_size().x, 25.0), egui::Button::new("✅ SYNC"))
                })
                .inner
            })
            .inner
        }
        SyncButtonState::IdleError => {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_enabled_ui(sync_enabled, |ui| {
                    ui.add_sized(vec2(ui.min_size().x, 25.0), egui::Button::new("❌ SYNC"))
                })
                .inner
            })
            .inner
        }
    };

    let Some(config) = state.config.as_mut() else {
        return;
    };
    if sync_button.clicked() {
        state.tab_active = ActiveTab::Log;
        if let Err(e) = state
            .tx
            .as_ref()
            .unwrap()
            .send((config.path.clone(), ui.ctx().clone()))
        {
            println!("Error: {:?}", e);
        };
    }
}

pub fn render(ui: &mut Ui, state: &mut RockSonicLite) {
    egui::Panel::bottom("footer")
        .exact_size(40.0)
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                let save_needed = state
                    .config
                    .as_ref()
                    .is_some_and(|config| config.save_needed);
                let label = if save_needed {
                    RichText::new("Config was changed").color(Color32::ORANGE)
                } else {
                    RichText::new("All good").color(Color32::LIGHT_GREEN)
                };
                ui.label(label);

                save_button(ui, state);
                sync_button(ui, state);
            });
        });
}
