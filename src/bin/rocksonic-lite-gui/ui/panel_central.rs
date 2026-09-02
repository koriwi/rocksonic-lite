use eframe::egui::{self, ScrollArea, TextEdit, Ui};

use crate::state::{ActiveTab, RockSonicLite};

fn tab_log(ui: &mut Ui, state: &mut RockSonicLite) {
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
}

fn tab_editor(ui: &mut Ui, state: &mut RockSonicLite) {
    let config_text = if let Some(config_text) = state.config_text_changed.as_mut() {
        config_text
    } else {
        &mut "".to_string()
    };

    let editor = TextEdit::multiline(config_text)
        .code_editor()
        .desired_width(f32::INFINITY);
    let editor = ui
        .add_enabled_ui(state.config_path.is_some(), |ui| {
            ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| ui.add_sized(ui.available_size(), editor))
                .inner
        })
        .inner;

    if editor.changed()
        && let Some(config_text) = state.config_text.as_ref()
        && let Some(config_text_changed) = state.config_text_changed.as_ref()
    {
        state.config_save_needed = config_text != config_text_changed;
    };
}

pub fn render(ui: &mut Ui, state: &mut RockSonicLite) {
    egui::CentralPanel::default().show(ui, |ui| {
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
        if state.tab_active == ActiveTab::Editor {
            tab_editor(ui, state);
        } else if state.tab_active == ActiveTab::Log {
            tab_log(ui, state);
        }
    });
}
