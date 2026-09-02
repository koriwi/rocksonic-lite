use std::fs;

use eframe::egui::{self, Color32, RichText, Ui, vec2};

use crate::state::{ActiveTab, RockSonicLite};

fn save_button(ui: &mut Ui, state: &mut RockSonicLite) {
    let save_button = ui
        .add_enabled_ui(state.config_save_needed, |ui| {
            ui.add_sized(
                vec2(ui.min_size().x, 25.0),
                egui::Button::new("save config"),
            )
        })
        .inner;
    if save_button.clicked() {
        fs::write(
            state.config_path.as_ref().unwrap(),
            state.config_text_changed.as_ref().unwrap(),
        )
        .expect("couldnt save config, sad");
        state.config_text = state.config_text_changed.clone();
        state.config_save_needed = false;
    }
}

fn sync_button(ui: &mut Ui, state: &mut RockSonicLite) {
    let sync_in_progress = state.sync_progress.lock().unwrap().is_some();
    let sync_button = if sync_in_progress {
        let progress = state.sync_progress.lock().unwrap();
        let (current, total) = progress.unwrap();
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_enabled_ui(false, |ui| {
                ui.add_sized(
                    vec2(ui.min_size().x, 25.0),
                    egui::Button::new(format!("{}/{}", current, total)),
                )
            })
            .inner
        })
        .inner
    } else {
        let sync_enabled = !state.config_save_needed && state.config_path.is_some();
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_enabled_ui(sync_enabled, |ui| {
                ui.add_sized(vec2(ui.min_size().x, 25.0), egui::Button::new("SYNC"))
            })
            .inner
        })
        .inner
    };

    if sync_button.clicked()
        && let Some(config_path) = state.config_path.clone()
    {
        state.tab_active = ActiveTab::Log;
        if let Err(e) = state
            .tx
            .as_ref()
            .unwrap()
            .send((config_path, ui.ctx().clone()))
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
                let label = if state.config_save_needed {
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
