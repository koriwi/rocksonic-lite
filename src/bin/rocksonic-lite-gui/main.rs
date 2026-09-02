pub mod state;
pub mod ui;

use crate::state::{RockSonicLite, SyncButtonState};
use crate::ui::{panel_bottom, panel_central, panel_top};

use eframe::egui::{self, vec2};
use rocksonic_lite::sync::SongFinishedInfo;
use rocksonic_lite::{SyncEvent, sync};
use std::{sync::mpsc::channel, thread};

impl eframe::App for RockSonicLite {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        panel_top::render(ui, self);
        panel_bottom::render(ui, self);
        panel_central::render(ui, self);
    }
}

fn on_song_finished(
    info: SongFinishedInfo,
    sync_button_state: &mut SyncButtonState,
    log_text: &mut String,
) {
    *sync_button_state = SyncButtonState::InProgress(Some((info.current, info.total)));

    let pad_count = info.total.to_string().len();
    let count_str = format!(
        "[{:>width$}/{}]",
        info.current,
        info.total,
        width = pad_count
    );
    let mut status_str = String::from("");

    status_str += if info.song_downloaded {
        "🎵⌛"
    } else {
        "🎵✔️"
    };
    status_str += if info.cover_downloaded {
        " 📷⌛"
    } else if info.cover_error {
        " 📷⚠️"
    } else {
        " 📷✔️"
    };

    log_text.push_str(&format!(
        "\n{} {} {} / {} / {}",
        count_str, status_str, info.artist, info.album, info.title,
    ));
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
    let sync_button_state = rs.sync_button_state.clone();
    let thread_log_text = rs.log_text.clone();
    thread::spawn(move || {
        loop {
            if let Ok((config_path, ctx)) = rx.recv()
                && let Err(e) = sync::run_sync(&config_path, |event| {
                    let mut sbs = sync_button_state.write().unwrap();
                    let mut tlg = thread_log_text.write().unwrap();
                    match event {
                        SyncEvent::SongFinished(info) => {
                            on_song_finished(info, &mut sbs, &mut tlg);
                        }
                        SyncEvent::Started => {
                            *sbs = SyncButtonState::InProgress(None);
                        }
                        SyncEvent::FileDeleted(path) => {
                            tlg.push_str(&format!(
                                "Deleting stale file {}",
                                path.to_str().unwrap()
                            ));
                        }
                        SyncEvent::Done => {
                            *sbs = SyncButtonState::IdleDone;
                        }
                        _ => todo!("uff"),
                    };
                    ctx.request_repaint();
                })
            {
                let mut tlg = thread_log_text.write().unwrap();
                let mut sbs = sync_button_state.write().unwrap();
                tlg.push_str(&format!("\nError: {:?}", e));
                *sbs = SyncButtonState::IdleError;
                ctx.request_repaint();
            };
        }
    });
    eframe::run_native("RockSonic Lite", options, Box::new(|_cc| Ok(Box::new(rs))))
}
