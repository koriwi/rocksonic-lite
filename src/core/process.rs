use crate::core::{
    covers::{cover_needs_download, process_cover},
    responses::SubSonicSong,
    server::Server,
    songs::{Action, song_needs_download, strip_mp3_artwork},
    utils::{download_file, sanitize_filename},
};
use rayon::prelude::*;
use std::{
    collections::HashSet,
    ffi::OsString,
    format, fs,
    path::{Path, PathBuf},
    sync::Mutex,
    vec,
};

pub struct SongResult {
    song: SubSonicSong,
    actions: Vec<Action>,
    audio_path: PathBuf,
}
pub struct SongResults {
    pub paths: HashSet<PathBuf>,
    pub audio_paths: Vec<PathBuf>,
}

struct SongPaths {
    artist: PathBuf,
    album: PathBuf,
    song: PathBuf,
    cover: PathBuf,
}

fn create_paths(library_dir: &Path, sb_song: &SubSonicSong, mp3: Option<u16>) -> SongPaths {
    let path = library_dir.to_path_buf();

    let artist: PathBuf = [
        path,
        PathBuf::from(sanitize_filename(OsString::from(sb_song.artist.to_owned()))),
    ]
    .iter()
    .collect();

    let album: PathBuf = [
        artist.clone(),
        PathBuf::from(sanitize_filename(OsString::from(sb_song.album.to_owned()))),
    ]
    .iter()
    .collect();

    let cover: PathBuf = [album.clone(), PathBuf::from("cover.jpeg")]
        .iter()
        .collect();

    let mut song = album.clone();

    song.push(format!(
        "{:0>3} {}.{}",
        sb_song.track.unwrap_or(0),
        sanitize_filename(sb_song.title.clone().into())
            .to_str()
            .unwrap(),
        if mp3.is_some() {
            "mp3"
        } else {
            &sb_song.suffix
        }
    ));

    SongPaths {
        artist,
        album,
        song,
        cover,
    }
}

/// does the heavy lifting, dowloading songs if missing or upgrade needed,
/// cover downloading, etc
/// TODO: reduce param count
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

    let mut audio_paths: Vec<PathBuf> = vec![];
    let wanted_paths = Mutex::new(HashSet::new());

    pool.install(|| {
        audio_paths = songs
            .par_iter()
            .map(|song| -> anyhow::Result<SongResult> {
                let mut actions = vec![];
                let paths = create_paths(library_dir, song, mp3);
                let handle_cover = {
                    let mut wanted_paths = wanted_paths.lock().unwrap();
                    wanted_paths.insert(paths.artist.clone());
                    wanted_paths.insert(paths.album.clone());
                    wanted_paths.insert(paths.song.clone());
                    wanted_paths.insert(paths.cover.clone())
                };

                if !fs::exists(&paths.album)? {
                    fs::create_dir_all(&paths.album)?;
                }

                if handle_cover
                    && cover_needs_download(&paths.cover, cover_size as u32, upgrade_covers)?
                {
                    let cover_resp = srv.get_cover_art(&song.id, cover_size)?;
                    if let Some(cover_action) = process_cover(&paths.cover, &cover_resp.bytes()?)? {
                        actions.push(cover_action);
                    }
                }

                // there is currently no way to get the bitrate the server has.
                // if the local bitrate is insufficient, the insufficient file gets downloaded again
                // as it may have been updated with a higher bitrate one, but we don't know
                if song_needs_download(&paths.song, mp3, upgrade_songs)? {
                    let mut song_stream = srv.get_song(&song.id, mp3)?;
                    download_file(&mut song_stream, &paths.song)?;

                    strip_mp3_artwork(&paths.song)?;

                    actions.push(Action::SongDownloaded);
                }
                Ok(SongResult {
                    song: song.clone(),
                    actions,
                    audio_path: paths.song,
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
                Some(result.audio_path)
            })
            .collect();
    });

    SongResults {
        paths: wanted_paths.into_inner().unwrap(),
        audio_paths,
    }
}
