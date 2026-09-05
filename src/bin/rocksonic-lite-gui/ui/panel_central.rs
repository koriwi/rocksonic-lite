use eframe::egui::{
    self, Button, Checkbox, DragValue, Label, RichText, ScrollArea, TextEdit, Ui, vec2,
};

use crate::state::{ActiveTab, RockSonicLite};

// fn key_value_row(ui: &mut Ui) {
//
// }

fn render_form(ui: &mut Ui, state: &mut RockSonicLite) {
    let Some(config) = state.config.as_mut() else {
        return;
    };

    ui.add(Label::new(
        RichText::new("THIS DOESN'T WORK YET, USE EDITOR INSTEAD")
            .color(eframe::egui::Color32::RED),
    ));

    ui.label("Connection");
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.add_sized(vec2(250.0, 25.0), Label::new("Server URL"));
                ui.add_sized(vec2(250.0, 25.0), Label::new("Username"));
                ui.add_sized(vec2(250.0, 25.0), Label::new("Password"));
            });

            ui.vertical(|ui| {
                ui.add_sized(
                    vec2(ui.available_width(), 25.0),
                    TextEdit::singleline(&mut config.config.server_url),
                );
                ui.add_sized(
                    vec2(ui.available_width(), 25.0),
                    TextEdit::singleline(&mut config.config.user),
                );
                ui.horizontal(|ui| {
                    let button = Button::new(
                        RichText::new(if state.password_hidden {
                            "show"
                        } else {
                            "hide"
                        })
                        .monospace(),
                    );
                    if ui.add_sized(vec2(60.0, 25.0), button).clicked() {
                        state.password_hidden ^= true;
                    }
                    ui.add_sized(
                        vec2(ui.available_width(), 25.0),
                        TextEdit::singleline(&mut config.config.password)
                            .password(state.password_hidden),
                    );
                });
            });
        });
    });

    ui.label("Format");
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.add_sized(vec2(250.0, 25.0), Label::new("MP3 conversion"));
                ui.add_sized(vec2(250.0, 25.0), Label::new("MP3 bitrate"));

                ui.add_sized(vec2(250.0, 25.0), Label::new("Cover size"));
            });
            ui.vertical(|ui| {
                match config.config.mp3 {
                    Some(mut mp3) => {
                        if ui
                            .add_sized(
                                vec2(ui.available_width(), 25.0),
                                Checkbox::new(&mut config.config.mp3.is_some(), "enabled"),
                            )
                            .clicked()
                        {
                            config.config.mp3 = None;
                        }
                        ui.add_sized(vec2(ui.available_width(), 25.0), DragValue::new(&mut mp3));
                    }
                    None => {
                        if ui
                            .add_sized(
                                vec2(ui.available_width(), 25.0),
                                Checkbox::new(&mut config.config.mp3.is_some(), "enabled"),
                            )
                            .clicked()
                        {
                            config.config.mp3 = Some(256);
                        }
                        ui.add_sized(
                            vec2(ui.available_width(), 25.0),
                            Label::new("enable conversion to set this"),
                        );
                    }
                };

                ui.add_sized(
                    vec2(ui.available_width(), 25.0),
                    DragValue::new(&mut config.config.cover_size),
                );
            });
        });
    });

    ui.label("Format");
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.add_sized(vec2(250.0, 25.0), Label::new("Create playlists"));
                ui.horizontal(|ui| {
                    ui.add_sized(vec2(220.0, 25.0), Label::new("Entities to sync"));
                    if ui.add_sized(vec2(25.0, 25.0), Button::new("+")).clicked() {
                        config.config.sync.push(String::from(""));
                    };
                });
            });
            ui.vertical(|ui| {
                if ui
                    .add_sized(
                        vec2(ui.available_width(), 25.0),
                        Checkbox::new(&mut config.config.mp3.is_some(), "enabled"),
                    )
                    .clicked()
                {
                    config.config.mp3 = None;
                }
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        for i in 0..config.config.sync.len() {
                            if ui.add_sized(vec2(25.0, 25.0), Button::new("-")).clicked() {
                                config.config.sync.remove(i);
                            };
                        }
                    });
                    ui.vertical(|ui| {
                        for sync in config.config.sync.iter_mut() {
                            ui.add_sized(
                                vec2(ui.available_width(), 25.0),
                                TextEdit::singleline(sync),
                            );
                        }
                    });
                });
            });
        });
    });
}

fn tab_form(ui: &mut Ui, state: &mut RockSonicLite) {
    ui.group(|ui| {
        ui.add_enabled_ui(true, |ui| {
            ScrollArea::both()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    // let text = state.log_text.read().unwrap();
                    // let mut text = text.as_str();
                    // ui.add_sized(ui.available_size(), TextEdit::multiline(&mut text))
                    render_form(ui, state);
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

pub fn render(ui: &mut Ui, state: &mut RockSonicLite) {
    egui::CentralPanel::default().show(ui, |ui| {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(state.tab_active == ActiveTab::Form, "Form")
                    .clicked()
                {
                    state.tab_active = ActiveTab::Form;
                };
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
        match state.tab_active {
            ActiveTab::Form => tab_form(ui, state),
            ActiveTab::Editor => tab_editor(ui, state),
            ActiveTab::Log => tab_log(ui, state),
        };
    });
}
