use std::{
    format,
    fs::{self, File},
    path::{Path, PathBuf},
};

pub fn create_playlist(
    name: &str,
    audio_paths: &[PathBuf],
    library_dir: &Path,
) -> anyhow::Result<()> {
    let playlist_entries: Vec<m3u::Entry> = audio_paths
        .iter()
        .map(|ap| {
            let mut audio_path = PathBuf::from("../");
            audio_path.push(ap);
            m3u::path_entry(audio_path)
        })
        .collect();

    let mut playlist_dir: PathBuf = library_dir.into();
    playlist_dir.pop(); // go up one directory, so to step out of the music dir
    playlist_dir.push("Playlists"); // append this to point to a sibling dir on the same level

    if !fs::exists(&playlist_dir)? {
        fs::create_dir(&playlist_dir)?;
    }

    let mut playlist_path = playlist_dir;
    playlist_path.push(format!("{}.m3u", name)); // points now to the playlist file

    let mut file = File::create(playlist_path)?;
    let mut writer = m3u::Writer::new(&mut file);

    for entry in &playlist_entries {
        writer.write_entry(entry)?;
    }
    Ok(())
}
