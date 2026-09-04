use std::{
    path::PathBuf,
    sync::{Arc, RwLock, mpsc::Sender},
};

use eframe::egui;
use rocksonic_lite::config::Config;

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
pub struct ConfigStruct {
    pub config: Config,
    pub path: PathBuf,
    pub text: String,
    pub text_changed: String,
    pub save_needed: bool,
}

#[derive(Debug, Default)]
pub struct RockSonicLite {
    pub tab_active: ActiveTab,
    pub sync_button_state: Arc<RwLock<SyncButtonState>>,
    pub config: Option<ConfigStruct>,
    pub log_text: Arc<RwLock<String>>,
    pub tx: Option<Sender<(PathBuf, egui::Context)>>,
}
