use crate::core::{
    covers::{cover_needs_download, process_cover},
    responses::SubSonicSong,
    server::Server,
    songs::{Action, song_needs_download, strip_mp3_artwork},
    utils::{download_file, sanitize_filename},
};
use rayon::prelude::*;
use std::{
    format, fs,
    path::{Path, PathBuf},
    vec,
};

pub struct SongResult {
    path_elements: Vec<PathBuf>,
    song: SubSonicSong,
    actions: Vec<Action>,
    audio_path: PathBuf,
}
pub struct SongResults {
    pub paths: Vec<PathBuf>,
    pub audio_paths: Vec<PathBuf>,
}

/*
* does the heavy lifting, dowloading songs if missing or upgrade needed,
* cover downloading, etc
*/
pub fn process_songs<F>(
    songs: &Vec<SubSonicSong>,
    library_dir: &Path,
    upgrade_covers: bool,
    cover_size: u16,
    upgrade_songs: bool,
    mp3: Option<u16>,
    srv: &Server,
    threads: usize,
    log_status: F,
) -> SongResults
where
    F: Fn(SubSonicSong, bool, bool, bool) + Sync,
{
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("could not create thread pool");
    let mut new_paths: Vec<(Vec<PathBuf>, PathBuf)> = vec![];
    pool.install(|| {
        new_paths = songs
            .par_iter()
            .map(|song| -> anyhow::Result<SongResult> {
                let mut known_paths = vec![];
                let mut actions = vec![];
                let mut album_dir = library_dir.to_path_buf();
                album_dir.push(sanitize_filename(song.artist.clone().into()));
                known_paths.push(album_dir.clone());
                album_dir.push(sanitize_filename(song.album.clone().into()));
                known_paths.push(album_dir.clone());

                if !fs::exists(&album_dir)? {
                    fs::create_dir_all(&album_dir)?;
                }

                let mut cover_path = album_dir.clone();
                cover_path.push("cover.jpeg");
                known_paths.push(cover_path.clone());

                if cover_needs_download(&cover_path, cover_size as u32, upgrade_covers)? {
                    let cover_resp = srv.get_cover_art(&song.id, cover_size)?;
                    if let Some(cover_action) = process_cover(&cover_path, &cover_resp.bytes()?)? {
                        actions.push(cover_action);
                    }
                }

                let mut song_path = album_dir.clone();
                song_path.push(format!(
                    "{:0>3} {}.{}",
                    song.track.unwrap_or(0),
                    crate::core::utils::sanitize_filename(song.title.clone().into())
                        .to_str()
                        .unwrap(),
                    if mp3.is_some() { "mp3" } else { &song.suffix }
                ));
                known_paths.push(song_path.clone());

                // there is currently no way to get the bitrate the server has.
                // if the local bitrate is insufficient, the insufficient file gets downloaded again
                // as it may have been updated with a higher bitrate one, but we don't know
                if song_needs_download(&song_path, mp3, upgrade_songs)? {
                    let mut song_stream = srv.get_song(&song.id, mp3)?;
                    download_file(&mut song_stream, &song_path)?;

                    strip_mp3_artwork(&song_path)?;

                    actions.push(Action::SongDownloaded);
                }
                Ok(SongResult {
                    path_elements: known_paths,
                    song: song.clone(),
                    actions,
                    audio_path: song_path,
                })
            })
            .filter_map(|elem| {
                let Ok(result) = elem else {
                    return None;
                };
                let song_downloaded = result.actions.contains(&Action::SongDownloaded);
                let cov_downloaded = result.actions.contains(&Action::CoverDownloaded);
                let cov_error = result.actions.contains(&Action::CoverError);
                log_status(result.song, song_downloaded, cov_downloaded, cov_error);
                Some((result.path_elements, result.audio_path))
            })
            .collect();
    });

    let mut all_paths = vec![];
    let mut audio_paths = vec![];
    new_paths.into_iter().for_each(|mut np| {
        all_paths.append(&mut np.0);
        audio_paths.push(np.1)
    });
    SongResults {
        paths: all_paths,
        audio_paths,
    }
}
