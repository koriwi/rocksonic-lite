pub mod state;
pub mod ui;

use crate::state::{RockSonicLite, SyncButtonState};
use crate::ui::{panel_bottom, panel_central, panel_top};

use eframe::egui::{self, vec2};
use rocksonic_lite::{SyncEvent, sync};
use std::{sync::mpsc::channel, thread};

impl eframe::App for RockSonicLite {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        panel_top::render(ui, self);
        panel_bottom::render(ui, self);
        panel_central::render(ui, self);
    }
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
    let sbs = rs.sync_button_state.clone();
    let thread_log_text = rs.log_text.clone();
    thread::spawn(move || {
        loop {
            if let Ok((config_path, ctx)) = rx.recv() {
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
                        let Ok(mut sbs) = sbs.write() else {
                            return;
                        };
                        if current == total {
                            *sbs = SyncButtonState::IdleDone;
                        } else {
                            *sbs = SyncButtonState::InProgress((current, total));
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
                        let mut log_text = thread_log_text.write().unwrap();
                        log_text.push_str(&format!(
                            "\n{} {} {} / {} / {}",
                            count_str, status_str, artist, album, title,
                        ));
                        ctx.request_repaint();
                    };
                })
                .expect("oof");
            }
        }
    });
    eframe::run_native("RockSonic Lite", options, Box::new(|_cc| Ok(Box::new(rs))))
}
