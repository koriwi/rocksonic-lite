use std::{
    path::PathBuf,
    sync::{Arc, RwLock, mpsc::Sender},
};

use eframe::egui;

#[derive(Default, Debug, PartialEq)]
pub enum ActiveTab {
    #[default]
    Editor,
    Log,
}

#[derive(Default, Debug, PartialEq)]
pub enum SyncButtonState {
    #[default]
    Idle,
    IdleDone,
    IdleError,
    InProgress(Option<(usize, usize)>),
}

#[derive(Debug, Default)]
pub struct RockSonicLite {
    pub tab_active: ActiveTab,
    pub sync_button_state: Arc<RwLock<SyncButtonState>>,
    pub config_path: Option<PathBuf>,
    pub config_text: Option<String>,
    pub config_text_changed: Option<String>,
    pub config_save_needed: bool,
    pub log_text: Arc<RwLock<String>>,
    pub tx: Option<Sender<(PathBuf, egui::Context)>>,
}
