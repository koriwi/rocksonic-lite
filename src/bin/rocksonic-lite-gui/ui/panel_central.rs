use eframe::egui::{self, Label, ScrollArea, TextEdit, Ui};

use crate::state::{ActiveTab, RockSonicLite};

fn tab_log(ui: &mut Ui, state: &mut RockSonicLite) {
    ui.group(|ui| {
        ui.add_enabled_ui(true, |ui| {
            ScrollArea::both()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    let text = state.log_text.read().unwrap();
                    let mut text = text.as_str();
                    ui.add_sized(ui.available_size(), TextEdit::multiline(&mut text))
                });
        });
    });
}

fn tab_editor(ui: &mut Ui, state: &mut RockSonicLite) {
    let Some(config) = state.config.as_mut() else {
        ui.group(|ui| {
            ui.add_sized(ui.available_size(), Label::new("No config file loaded"));
        });
        return;
    };
    let editor = TextEdit::multiline(&mut config.text_changed)
        .code_editor()
        .desired_width(f32::INFINITY);
    // TODO: remove add_enabled_ui, not needed anymore
    let editor = ui
        .add_enabled_ui(true, |ui| {
            ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| ui.add_sized(ui.available_size(), editor))
                .inner
        })
        .inner;

    let Some(config) = state.config.as_mut() else {
        return;
    };
    if editor.changed() {
        config.save_needed = config.text != config.text_changed;
    };
}

pub fn render(ui: &mut Ui, state: &mut RockSonicLite) {
    egui::CentralPanel::default().show(ui, |ui| {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(state.tab_active == ActiveTab::Editor, "Config")
                    .clicked()
                {
                    state.tab_active = ActiveTab::Editor;
                };
                if ui
                    .selectable_label(state.tab_active == ActiveTab::Log, "Log")
                    .clicked()
                {
                    state.tab_active = ActiveTab::Log;
                }
            });
        });
        if state.tab_active == ActiveTab::Editor {
            tab_editor(ui, state);
        } else if state.tab_active == ActiveTab::Log {
            tab_log(ui, state);
        }
    });
}
