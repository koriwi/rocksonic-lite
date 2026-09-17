use anyhow::{Result, anyhow};
use std::{
    ffi::OsString,
    format,
    fs::{self, File},
    path::{Path, PathBuf},
};

use crate::core::utils::sanitize_filename;

fn create_playlist_dir(library_dir: &Path) -> Result<PathBuf> {
    let playlist_dir: PathBuf = [library_dir.to_owned(), "../".into(), "Playlists".into()]
        .iter()
        .collect();

    if !fs::exists(&playlist_dir)? {
        fs::create_dir(&playlist_dir)?;
    }
    Ok(playlist_dir)
}

fn create_playlist_path(library_dir: &Path, playlist_name: &str) -> Result<PathBuf> {
    let playlist_dir = create_playlist_dir(library_dir)?;
    Ok([
        playlist_dir.to_path_buf(),
        format!(
            "{}.m3u",
            sanitize_filename(OsString::from(playlist_name.to_owned()))
                .to_str()
                .ok_or(anyhow!("could not convert OsString to str"))?
        )
        .into(),
    ]
    .iter()
    .collect())
}

pub fn create_playlist(
    name: &str,
    audio_paths: &[PathBuf],
    library_dir: &Path,
) -> anyhow::Result<()> {
    let playlist_entries: Vec<m3u::Entry> = audio_paths
        .iter()
        .map(|audio_path| {
            m3u::path_entry(
                ["../".into(), audio_path.to_owned()]
                    .iter()
                    .collect::<PathBuf>(),
            )
        })
        .collect();

    let playlist_path = create_playlist_path(library_dir, name)?;

    let mut file = File::create(playlist_path)?;
    let mut writer = m3u::Writer::new(&mut file);

    for entry in &playlist_entries {
        writer.write_entry(entry)?;
    }
    Ok(())
}
