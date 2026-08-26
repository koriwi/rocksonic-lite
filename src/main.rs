pub mod libs;
use crate::libs::{
    playlists::create_playlist,
    responses::SubSonicSong,
    songs::{get_song_lists, process_songs},
};
use anyhow::{Result, anyhow};
use clap::Parser;
use libs::server;
use serde::Deserialize;
use std::{
    format,
    fs::{self},
    path::{Path, PathBuf},
    println,
    sync::atomic::{AtomicU64, Ordering},
    vec,
};

fn default_mp3() -> Option<u16> {
    None
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

#[derive(Deserialize, Debug)]
pub struct Config<'a> {
    server_url: &'a str,
    user: &'a str,
    password: &'a str,
    #[serde(default = "default_mp3")]
    mp3: Option<u16>,
    #[serde(default = "default_cover_size")]
    cover_size: u16,
    sync: Vec<&'a str>,
    #[serde(default = "default_create_playlist")]
    create_playlist: bool,
    #[serde(default = "default_threads")]
    threads: u16,
}

#[derive(Parser, Debug)]
struct Args {
    // path to the config file
    #[arg(short, long)]
    config: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // create config struct
    let config_path = Path::new(args.config.as_str());
    if !fs::exists(config_path)? {
        return Err(anyhow!(format!(
            "Could not find the config file {}",
            args.config
        )));
    }
    let config = fs::read_to_string(config_path)?;
    let config: Config = yaml_serde::from_str(&config)?;

    rayon::ThreadPoolBuilder::new()
        .num_threads(config.threads as usize)
        .build_global()?;

    let srv = server::Server::connect(config.server_url, config.user, config.password)?;

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

    let mut known_paths: Vec<PathBuf> = vec![];
    let song_count: usize = song_lists.iter().map(|sl| sl.songs.len()).sum();
    let pad_count = song_count.to_string().len();
    let global_counter = AtomicU64::new(1);

    for song_list in song_lists {
        let (mut new_paths, audio_paths) = process_songs(
            &song_list.songs,
            &library_dir,
            config.cover_size,
            config.mp3,
            &srv,
            |song: SubSonicSong, song_dl: bool, cover_dl: bool, cover_err: bool| {
                let count_str = format!(
                    "[{:>width$}/{}]",
                    global_counter.fetch_add(1, Ordering::AcqRel),
                    song_count,
                    width = pad_count
                );
                let mut status_str = String::from("");

                status_str += if song_dl { "🎵⌛" } else { "🎵✔️" };
                status_str += if cover_dl {
                    " 📷⌛"
                } else if cover_err {
                    " 📷⚠️"
                } else {
                    " 📷✔️"
                };

                println!(
                    "{} {} {} / {} / {}",
                    count_str, status_str, song.artist, song.album, song.title,
                );
            },
        );

        if config.create_playlist
            && let Some(playlist_name) = song_list.name
        {
            create_playlist(&playlist_name, &audio_paths, &library_dir)?;
        }

        known_paths.append(&mut new_paths);
    }

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
            println!("deleting {}", path_entry.path().to_str().unwrap())
        }
    }

    Ok(())
}
