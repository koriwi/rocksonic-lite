use std::{
    fs, io,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        mpsc::{Sender, channel},
    },
    thread,
};

use eframe::egui::{self, Color32, Label, RichText, ScrollArea, TextEdit, vec2};
use rocksonic_lite::{SyncEvent, sync};
#[derive(Debug, Default)]
pub struct RockSonicLite {
    config_path: Option<PathBuf>,
    config_text: Option<String>,
    config_text_changed: Option<String>,
    config_save_needed: bool,
    tx: Option<Sender<PathBuf>>,
    sync_progress: Arc<Mutex<Option<(usize, usize)>>>,
}

fn main() -> eframe::Result {
    let (tx, rx) = channel();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(vec2(700f32, 400f32)),

        ..Default::default()
    };
    let rs = RockSonicLite {
        tx: Some(tx),
        ..RockSonicLite::default()
    };
    let thread_rs = rs.sync_progress.clone();
    thread::spawn(move || {
        loop {
            if let Ok(config_path) = rx.recv() {
                sync::run_sync(&config_path, |event| {
                    if let SyncEvent::SongFinished {
                        current,
                        total,
                        artist,
                        album,
                        title,
                        song_downloaded,
                        cover_downloaded,
                        cover_error,
                    } = event
                    {
                        let Ok(mut sp) = thread_rs.lock() else {
                            return;
                        };
                        if current == total {
                            *sp = None;
                        } else {
                            *sp = Some((current, total));
                        }
                        let pad_count = total.to_string().len();
                        let count_str =
                            format!("[{:>width$}/{}]", current, total, width = pad_count);
                        let mut status_str = String::from("");

                        status_str += if song_downloaded {
                            "🎵⌛"
                        } else {
                            "🎵✔️"
                        };
                        status_str += if cover_downloaded {
                            " 📷⌛"
                        } else if cover_error {
                            " 📷⚠️"
                        } else {
                            " 📷✔️"
                        };

                        println!(
                            "{} {} {} / {} / {}",
                            count_str, status_str, artist, album, title,
                        )
                    };
                })
                .expect("oof");
            }
        }
    });
    eframe::run_native("RockSonic Lite", options, Box::new(|cc| Ok(Box::new(rs))))
}

impl eframe::App for RockSonicLite {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::Panel::top("header").exact_size(50.0).show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.heading("RockSonic Lite");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add_sized(vec2(ui.min_size().x, 25.0), egui::Button::new("choose"))
                        .clicked()
                    {
                        let fd = rfd::FileDialog::new();
                        self.config_path = fd
                            .set_title("Select the config file yaml")
                            .add_filter("RockSonicLite config file", &["yaml", "yml"])
                            .pick_file();
                        let Some(config_path) = self.config_path.as_ref() else {
                            return;
                        };
                        let io::Result::Ok(config_text) = fs::read_to_string(config_path) else {
                            return;
                        };
                        self.config_text = Some(config_text);
                        self.config_text_changed = self.config_text.clone();
                    };
                    ui.label("Choose config file:");
                });
            });
        });
        egui::Panel::bottom("footer")
            .exact_size(40.0)
            .show(ui, |ui| {
                let (save_config_button, sync_button) = ui
                    .horizontal_centered(|ui| {
                        let save_enabled = self.config_save_needed;
                        let label = if save_enabled {
                            RichText::new("Config was changed").color(Color32::ORANGE)
                        } else {
                            RichText::new("All good").color(Color32::LIGHT_GREEN)
                        };
                        ui.label(label);
                        let save_button = ui
                            .add_enabled_ui(save_enabled, |ui| {
                                ui.add_sized(
                                    vec2(ui.min_size().x, 25.0),
                                    egui::Button::new("save config"),
                                )
                            })
                            .inner;
                        let sync_in_progress = self.sync_progress.try_lock().unwrap().is_some();
                        let sync_button = if sync_in_progress {
                            let progress = self.sync_progress.lock().unwrap();
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
                            let sync_enabled = !save_enabled && self.config_path.is_some();
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.add_enabled_ui(sync_enabled, |ui| {
                                    ui.add_sized(
                                        vec2(ui.min_size().x, 25.0),
                                        egui::Button::new("SYNC"),
                                    )
                                })
                                .inner
                            })
                            .inner
                        };
                        (save_button, sync_button)
                    })
                    .inner;
                if save_config_button.clicked() {
                    fs::write(
                        self.config_path.as_ref().unwrap(),
                        self.config_text_changed.as_ref().unwrap(),
                    )
                    .expect("couldnt save config, sad");
                    self.config_text = self.config_text_changed.clone();
                    self.config_save_needed = false;
                }
                if sync_button.clicked()
                    && let Some(config_path) = self.config_path.clone()
                {
                    self.tx.as_ref().unwrap().send(config_path);
                }
            });
        egui::CentralPanel::default().show(ui, |ui| {
            let config_text = if let Some(config_text) = self.config_text.as_mut() {
                config_text
            } else {
                &mut "".to_string()
            };

            let editor = TextEdit::multiline(config_text)
                .code_editor()
                .desired_width(f32::INFINITY);
            let editor = ui
                .add_enabled_ui(self.config_path.is_some(), |ui| {
                    ScrollArea::both()
                        .auto_shrink([false, false])
                        .show(ui, |ui| ui.add_sized(ui.available_size(), editor))
                        .inner
                })
                .inner;

            if editor.changed()
                && let Some(config_text) = self.config_text.as_ref()
                && let Some(config_text_changed) = self.config_text_changed.as_ref()
            {
                self.config_save_needed = config_text != config_text_changed;
            };
        });
    }
}
