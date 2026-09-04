use anyhow::Result;
use serde::Deserialize;
use std::{
    fs::{self},
    path::Path,
};

fn default_upgrade_songs() -> bool {
    false
}
fn default_mp3() -> Option<u16> {
    None
}
fn default_upgrade_covers() -> bool {
    false
}
fn default_cover_size() -> u16 {
    300
}
fn default_create_playlist() -> bool {
    true
}
fn default_threads() -> u16 {
    4
}

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    pub server_url: String,
    pub user: String,
    pub password: String,
    #[serde(default = "default_upgrade_songs")]
    pub upgrade_songs: bool,
    #[serde(default = "default_mp3")]
    pub mp3: Option<u16>,
    #[serde(default = "default_upgrade_covers")]
    pub upgrade_covers: bool,
    #[serde(default = "default_cover_size")]
    pub cover_size: u16,
    pub sync: Vec<String>,
    #[serde(default = "default_create_playlist")]
    pub create_playlist: bool,
    #[serde(default = "default_threads")]
    pub threads: u16,
}
impl Config {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let yaml = fs::read_to_string(path)?;
        Ok(yaml_serde::from_str(&yaml)?)
    }
}
