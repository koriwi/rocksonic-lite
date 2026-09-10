// TODO: get rid of magic numbers in this file
use crate::state::{ActiveTab, ConfigStruct, RockSonicLite};
use eframe::egui::{
    self, Align, Button, Checkbox, DragValue, Label, Layout, Response, RichText, ScrollArea,
    TextEdit, Ui, Widget, vec2,
};
use rocksonic_lite::config;
use std::fs;
#[derive(Default)]
struct Form {
    changed: bool,
}

impl Form {
    fn field(&mut self, ui: &mut Ui, label: &str, value: impl Widget) {
        self.changed |= key_value_row(ui, Label::new(label), value).1.changed();
    }

    fn mark_changed(&mut self) {
        self.changed = true;
    }
}

fn key_value_row(ui: &mut Ui, key: impl Widget, value: impl Widget) -> (Response, Response) {
    ui.horizontal(|ui| {
        (
            ui.add_sized(vec2(120.0, 25.0), key),
            ui.add_sized(vec2(ui.available_width(), 25.0), value),
        )
    })
    .inner
}

fn new_config_button(ui: &mut Ui, state: &mut RockSonicLite) {
    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() / 2.0 - 40.0);
        if ui.button("Create new config").clicked() {
            let fd = rfd::FileDialog::new();
            let new_config_path = fd
                .set_title("Select the new config file location")
                .add_filter("RockSonicLite config file", &["yaml", "yml"])
                .save_file();
            let Some(config_path) = new_config_path.as_ref() else {
                return;
            };
            let config = config::Config::default();
            let Ok(config_string) = yaml_serde::to_string(&config) else {
                return;
            };
            // TODO: get rid of this unwrap
            fs::write(config_path, config_string).unwrap();
            let config_text = yaml_serde::to_string(&config).unwrap();
            state.config = Some(ConfigStruct {
                config,
                path: config_path.clone(),
                text: config_text.clone(),
                text_changed: config_text.clone(),
                save_needed: false,
            });
        }
        ui.label("or open existing config");
        ui.add_space(ui.available_height());
    });
}

// TODO: split this form render to make it more structured/readable
fn render_form(ui: &mut Ui, state: &mut RockSonicLite) {
    let mut form = Form::default();
    // this can be safely unwrapped
    let config = state.config.as_mut().unwrap();

    ui.label("Connection");
    ui.group(|ui| {
        // Server URL
        form.field(
            ui,
            "Server URL",
            TextEdit::singleline(&mut config.config.server_url),
        );

        // Username
        form.field(
            ui,
            "Username",
            TextEdit::singleline(&mut config.config.user),
        );

        // Password
        form.field(ui, "Password", |ui: &mut Ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
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
                )
            })
            .inner
        });
    });
    ui.add_space(25.0);

    ui.label("Format(song/cover)");
    ui.columns(2, |col| {
        col[0].group(|ui| {
            let cnv_btn = match config.config.mp3.as_mut() {
                Some(mp3) => {
                    // MP3 conversion
                    let (_, cnv_btn) =
                        key_value_row(ui, Label::new("MP3 conversion"), Button::new("enabled"));
                    // MP3 bitrate
                    form.field(ui, "MP3 bitrate", DragValue::new(mp3));
                    cnv_btn
                }
                None => {
                    // MP3 Conversion
                    let (_, cnv_btn) =
                        key_value_row(ui, Label::new("MP3 conversion"), Button::new("disabled"));
                    // MP3 Bitrate
                    key_value_row(
                        ui,
                        Label::new("MP3 bitrate"),
                        Label::new("enable conversion to set this"),
                    );
                    cnv_btn
                }
            };
            if cnv_btn.clicked() {
                form.mark_changed();
                config.config.mp3 = match config.config.mp3 {
                    Some(_) => None,
                    None => Some(256),
                };
            }

            // Upgrade Song
            form.field(
                ui,
                "Upgrade songs",
                Checkbox::new(&mut config.config.upgrade_songs, "upgrade"),
            );
        });
        col[1].group(|ui| {
            // Cover size
            form.field(
                ui,
                "Cover size",
                DragValue::new(&mut config.config.cover_size),
            );
            // Upgrade Song
            form.field(
                ui,
                "Upgrade covers",
                Checkbox::new(&mut config.config.upgrade_covers, "upgrade"),
            );
            ui.add_space(25.0);
            // key_value_row(ui, egui, value)
        });
    });
    ui.add_space(25.0);

    ui.label("Sync");
    ui.group(|ui| {
        // sync threads
        form.field(
            ui,
            "Threads",
            DragValue::new(&mut config.config.threads).range(1..=128),
        );

        // create playlists
        form.field(
            ui,
            "Create playlists",
            Checkbox::new(&mut config.config.create_playlist, "enabled"),
        );

        // Sync entities
        if key_value_row(ui, Label::new("Entities to sync"), Button::new("+"))
            .1
            .clicked()
        {
            form.mark_changed();
            config.config.sync.push(String::from(""));
        }

        let mut to_remove = vec![];
        for i in 0..config.config.sync.len() {
            let (minus_button, sync_string) = key_value_row(
                ui,
                Button::new("-"),
                TextEdit::singleline(&mut config.config.sync[i]),
            );

            if minus_button.clicked() {
                form.mark_changed();
                to_remove.push(i);
            }

            if sync_string.changed() {
                form.mark_changed();
            }
        }
        to_remove.into_iter().for_each(|ri| {
            config.config.sync.remove(ri);
        });
    });
    if form.changed {
        config.text_changed = yaml_serde::to_string(&config.config).unwrap();
        config.save_needed = true;
    }
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
    ui.group(|ui| {
        // this can be safely unwrapped
        let config = state.config.as_mut().unwrap();

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
    });
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
                    && let Some(config) = state.config.as_mut()
                {
                    let des_result = yaml_serde::from_str(&config.text_changed);
                    match des_result {
                        Ok(conf) => {
                            config.config = conf;
                            state.tab_active = ActiveTab::Form;
                        }
                        Err(e) => {
                            state.error = Some(e.to_string());
                        }
                    }
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
        if state.error.is_some() {
            let modal = eframe::egui::Modal::new("config_des_err_modal".into());
            if modal
                .show(ui.ctx(), |ui| {
                    ui.heading("Oops");
                    ui.separator();
                    // TODO: i am tired, so here is an unwrap :S
                    ui.text_edit_multiline(&mut state.error.as_ref().unwrap().as_str());
                    ui.separator();
                    if ui.button("OK").clicked() {
                        state.error = None;
                    }
                })
                .should_close()
            {
                state.error = None;
            }
        }

        if state.config.is_none() {
            ui.group(|ui| {
                new_config_button(ui, state);
            });
            return;
        };
        match state.tab_active {
            ActiveTab::Form => tab_form(ui, state),
            ActiveTab::Editor => tab_editor(ui, state),
            ActiveTab::Log => tab_log(ui, state),
        };
    });
}
