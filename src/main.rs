pub mod libs;
use crate::libs::{
    responses::SubSonicSong,
    songs::{get_song_lists, process_songs},
    utils::sanitize_filename,
};
use anyhow::{Result, anyhow};
use clap::Parser;
use libs::server;
use serde::Deserialize;
use std::{
    format,
    fs::{self, File},
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
    let config_exists = fs::exists(config_path)?;
    if !config_exists {
        return Err(anyhow!(format!(
            "Could not find the config file {}",
            args.config
        )));
    }
    let config_file = fs::read_to_string(config_path)?;
    let config: Config = yaml_serde::from_str(&config_file)?;
    rayon::ThreadPoolBuilder::new()
        .num_threads(config.threads as usize)
        .build_global()?;

    let srv = server::Server::connect(config.server_url, config.user, config.password)?;

    // build the target library path based on the config file name
    let config_file_path = config_path.with_file_name("");
    let config_file_name = config_path
        .file_stem()
        .ok_or_else(|| anyhow!("config file name is too funky"))?;

    let mut library_dir = config_file_path.clone();
    library_dir.push(config_file_name);

    if !fs::exists(&library_dir)? {
        fs::create_dir(&library_dir)?;
    }

    let song_lists = get_song_lists(&config, &srv);

    let mut known_paths: Vec<PathBuf> = vec![];
    let song_count: usize = song_lists.iter().map(|sl| sl.1.len()).sum();
    let pad_count = song_count.to_string().len();
    let global_counter = AtomicU64::new(1);

    for song_list in song_lists {
        let (playlist_name, songs) = song_list;

        let (mut new_paths, mut audio_paths) = process_songs(
            &songs,
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
            && let Some(name) = playlist_name
        {
            let playlist: Vec<m3u::Entry> = audio_paths
                .iter_mut()
                .map(|ap| {
                    let mut audio_path = PathBuf::from("../");
                    audio_path.push(&ap);
                    m3u::path_entry(audio_path)
                })
                .collect();
            let mut playlist_dir = library_dir.clone();
            playlist_dir.pop();
            playlist_dir.push("Playlists");
            if !fs::exists(&playlist_dir)? {
                fs::create_dir(&playlist_dir)?;
            }
            let mut playlist_path = playlist_dir;
            playlist_path.push(format!("{}.m3u", name));
            let mut file = File::create(playlist_path)?;
            let mut writer = m3u::Writer::new(&mut file);
            for entry in &playlist {
                writer.write_entry(entry)?;
            }
        }
        known_paths.append(&mut new_paths);
    }
    known_paths.push(library_dir.clone());

    let walker = walkdir::WalkDir::new(&library_dir).contents_first(true);
    for entry in walker {
        let Ok(ent) = entry else { continue };
        let sanitized_entry: PathBuf = ent
            .path()
            .iter()
            .map(|part| sanitize_filename(part.into()))
            .collect();
        let found = known_paths.contains(&sanitized_entry);
        if !found {
            if ent.path().is_file() {
                fs::remove_file(ent.path())?;
            } else {
                fs::remove_dir(ent.path())?;
            }
            println!("deleting {}", ent.path().to_str().unwrap())
        }
    }

    Ok(())
}
