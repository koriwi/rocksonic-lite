use crate::{
    Config,
    libs::{
        playlists::create_playlist, process::process_songs, responses::SubSonicSong, server,
        songs::get_song_lists,
    },
};
use anyhow::{Result, anyhow};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    vec,
};

#[derive(Debug)]
pub enum SyncEvent {
    Started {
        total: usize,
    },
    SongFinished {
        current: usize,
        total: usize,
        artist: String,
        album: String,
        title: String,
        song_downloaded: bool,
        cover_downloaded: bool,
        cover_error: bool,
    },
    FileDeleted(PathBuf),
    Warning(String),
}

pub fn run_sync<F>(config_path: &Path, emit: F) -> Result<()>
where
    F: Fn(SyncEvent) + Sync,
{
    let config = Config::from_path(config_path)?;
    let srv = server::Server::connect(&config.server_url, &config.user, &config.password)?;

    // build the target library path based on the config file name
    let config_file_dir = config_path.with_file_name("");
    let config_file_name = config_path
        .file_stem()
        .ok_or_else(|| anyhow!("config file name is too funky"))?;

    let mut library_dir = config_file_dir.clone();
    library_dir.push(config_file_name);

    if !fs::exists(&library_dir)? {
        fs::create_dir(&library_dir)?;
    }

    let song_lists = get_song_lists(&config, &srv);

    // this is used for finding outdated files/directories to delete them later
    let mut known_paths: Vec<PathBuf> = vec![];

    // complete song count for progress indicator
    let song_count: usize = song_lists.iter().map(|sl| sl.songs.len()).sum();
    // for counter padding
    let global_counter = AtomicU64::new(1);
    emit(SyncEvent::Started { total: song_count });

    for song_list in song_lists {
        let mut song_results = process_songs(
            &song_list.songs,
            &library_dir,
            config.upgrade_covers,
            config.cover_size,
            config.upgrade_songs,
            config.mp3,
            &srv,
            config.threads as usize,
            |song: SubSonicSong, song_dl: bool, cover_dl: bool, cover_err: bool| {
                emit(SyncEvent::SongFinished {
                    current: global_counter.fetch_add(1, Ordering::AcqRel) as usize,
                    total: song_count,
                    artist: song.artist,
                    album: song.album,
                    title: song.title,
                    song_downloaded: song_dl,
                    cover_downloaded: cover_dl,
                    cover_error: cover_err,
                })
            },
        );
        if config.create_playlist
            && let Some(playlist_name) = song_list.name
        {
            create_playlist(&playlist_name, &song_results.audio_paths, &library_dir)?;
        }

        known_paths.append(&mut song_results.paths);
    }

    // add the root dir, so we dont delete everything
    known_paths.push(library_dir.clone());

    // walks through the library and rm all unknown files
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
            // println!("deleting {}", path_entry.path().to_str().unwrap())
        }
    }
    Ok(())
}
