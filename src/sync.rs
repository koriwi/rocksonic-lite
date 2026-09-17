use crate::{
    config::Config,
    core::{
        playlists::create_playlist, process::process_songs, responses::SubSonicSong, server,
        songs::get_song_lists,
    },
};
use anyhow::{Result, anyhow};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

#[derive(Debug)]
pub struct SongFinishedInfo {
    pub current: usize,
    pub total: usize,
    pub artist: String,
    pub album: String,
    pub title: String,
    pub song_downloaded: bool,
    pub cover_downloaded: bool,
    pub cover_error: bool,
}

#[derive(Debug)]
pub enum SyncEvent {
    Started,
    SongFinished(SongFinishedInfo),
    FileDeleted(PathBuf),
}

/// creates the library dir and returns the path to the newly create dir
fn create_library_dir(config_path: &Path) -> Result<PathBuf> {
    let config_file_dir = config_path.with_file_name("");
    let config_file_name = config_path
        .file_stem()
        .ok_or_else(|| anyhow!("config file name is too funky"))?;

    let mut library_dir = config_file_dir.clone();
    library_dir.push(config_file_name);
    if !fs::exists(&library_dir)? {
        fs::create_dir(&library_dir)?;
    }
    Ok(library_dir)
}

pub fn run_sync<F>(config_path: &Path, emit: F) -> Result<()>
where
    F: Fn(SyncEvent) + Sync,
{
    emit(SyncEvent::Started);
    let config = Config::from_path(config_path)?;
    let srv = server::Server::connect(&config.server_url, &config.user, &config.password)?;

    let library_dir = create_library_dir(config_path)?;

    let song_lists = get_song_lists(&config, &srv);

    // to keep track of all the files that we manage
    let mut known_paths: HashSet<PathBuf> = HashSet::new();

    let song_count: usize = song_lists
        .iter()
        .filter_map(|sl| Some(sl.as_ref().ok()?.songs.len()))
        .sum();
    let current_song_index = AtomicU64::new(1);
    let mut success_lists = vec![];
    let mut error_lists = vec![];

    for song_list_result in song_lists {
        match song_list_result {
            Ok(song_list) => success_lists.push(song_list),
            Err(e) => error_lists.push(e),
        };
    }

    for song_list in success_lists {
        let song_results = process_songs(
            &song_list.songs,
            &library_dir,
            config.upgrade_covers,
            config.cover_size,
            config.upgrade_songs,
            config.mp3,
            &srv,
            config.threads as usize,
            |song: SubSonicSong, song_dl: bool, cover_dl: bool, cover_err: bool| {
                emit(SyncEvent::SongFinished(SongFinishedInfo {
                    current: current_song_index.fetch_add(1, Ordering::AcqRel) as usize,
                    total: song_count,
                    artist: song.artist,
                    album: song.album,
                    title: song.title,
                    song_downloaded: song_dl,
                    cover_downloaded: cover_dl,
                    cover_error: cover_err,
                }))
            },
        );
        if config.create_playlist
            && let Some(playlist_name) = song_list.name
        {
            create_playlist(&playlist_name, &song_results.audio_paths, &library_dir)?;
        }

        known_paths.extend(song_results.paths);
    }

    if !error_lists.is_empty() {
        let details = error_lists
            .iter()
            .map(|e| format!(" - {:#}", e))
            .collect::<Vec<String>>()
            .join("\n");
        return Err(anyhow!(
            "Skipping cleanup. Elements failed to sync:\n{details}"
        ));
    }

    // don't forget to add the root dir, so we dont delete everything.
    // ask me how I know
    known_paths.insert(library_dir.clone());

    // checks every file in the library if it is wanted, if not -> rm
    // TODO: put this into a nice little function maybe
    let walker_paths = walkdir::WalkDir::new(&library_dir).contents_first(true);
    for path in walker_paths {
        let Ok(path_entry) = path else { continue };

        let found = known_paths.contains(&path_entry.path().to_path_buf());

        if !found {
            if path_entry.path().is_file() {
                fs::remove_file(path_entry.path())?;
            } else {
                fs::remove_dir(path_entry.path())?;
            }
            emit(SyncEvent::FileDeleted(path_entry.path().to_path_buf()));
        }
    }
    Ok(())
}
