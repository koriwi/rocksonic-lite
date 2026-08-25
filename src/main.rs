pub mod libs;
use image::{ImageFormat, codecs::jpeg::JpegEncoder};
use libs::server;
use lofty::{config::ParseOptions, file::AudioFile, mpeg::MpegFile};
use rayon::prelude::*;
use zune_jpeg::{JpegDecoder, zune_core::bytestream::ZCursor};

use std::{
    fmt, format,
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
    print, println,
    sync::atomic::{AtomicU64, Ordering},
    vec, write,
};

use anyhow::{Result, anyhow};
use clap::Parser;
use serde::Deserialize;

use crate::libs::{
    responses::SubSonicSong,
    server::Server,
    utils::{download_file, sanitize_filename},
};
// returns the percentage
fn check_number_size_diff(num_a: u32, num_b: u32, max_diff: f32) -> bool {
    let mut diff = 0;
    let mut abs_max_diff = 0;
    if num_a < num_b {
        diff = num_b - num_a;
        abs_max_diff = (max_diff * (num_b as f32)) as u32;
    }
    if num_a > num_b {
        diff = num_a - num_b;
        abs_max_diff = (max_diff * (num_a as f32)) as u32;
    }
    diff <= abs_max_diff
}

fn process_cover(path: &Path, data: &[u8]) -> Result<()> {
    match image::guess_format(data)? {
        ImageFormat::Jpeg => {
            let cover_data_cursor = ZCursor::new(data);
            let mut cover_decoder = JpegDecoder::new(cover_data_cursor);
            cover_decoder.decode_headers()?;
            let cover_info = cover_decoder
                .info()
                .ok_or_else(|| anyhow!("JPEG: Malformed header info"))?;

            if !cover_info.sof.is_sequential_dct() {
                println!("JPEG is not baseline, converting...");
                let cover_rgb8 = image::load_from_memory(data)?.to_rgb8();
                let mut cover_baseline = Vec::new();
                JpegEncoder::new_with_quality(&mut cover_baseline, 90).encode(
                    cover_rgb8.as_raw(),
                    cover_rgb8.width(),
                    cover_rgb8.height(),
                    image::ExtendedColorType::Rgb8,
                )?;
                fs::write(path, &cover_baseline)?;
                return Ok(());
            } else {
                fs::write(path, data)?
            }
        }
        _ => println!("image format not supported yet"),
    };
    println!("Cover downloaded: {}", path.to_str().unwrap());
    Ok(())
}

fn default_mp3() -> Option<u16> {
    Some(256)
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
struct Config<'a> {
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

#[derive(Parser, Debug, PartialEq, Eq)]
enum Action {
    SongDownloaded,
    CoverDownloaded,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn process_songs(
    songs: Vec<SubSonicSong>,
    library_dir: &Path,
    config: Config,
    srv: Server,
) -> Vec<PathBuf> {
    let global_counter = AtomicU64::new(1);
    let song_count = songs.len();
    let pad_count = song_count.to_string().len();
    let new_paths: Vec<Vec<PathBuf>> = songs
        .par_iter()
        .map(
            |song| -> anyhow::Result<(Vec<PathBuf>, SubSonicSong, Vec<Action>)> {
                let mut known_paths = vec![];
                let mut actions = vec![];
                // println!(
                //     "[{}][{:0>5}/{:0>5}] progress {}%",
                //     ei,
                //     i,
                //     songs.len(),
                //     i as f32 / songs.len() as f32 * 100f32
                // );
                let mut album_dir = library_dir.to_path_buf();
                album_dir.push(sanitize_filename(song.artist.clone().into()));
                known_paths.push(album_dir.clone());
                album_dir.push(sanitize_filename(song.album.clone().into()));
                known_paths.push(album_dir.clone());

                if !fs::exists(&album_dir)? {
                    fs::create_dir_all(&album_dir)?;
                }

                // =========================
                // BEGIN OF COVER PROCESSING
                // =========================
                let mut cover_path = album_dir.clone();
                cover_path.push("cover.jpeg");
                known_paths.push(cover_path.clone());

                // check if existing cover has the correct size within a percentage
                if fs::exists(&cover_path)? {
                    let cover_file = File::open(&cover_path)?;
                    let cover_reader = BufReader::new(cover_file);
                    let mut cover_decoder = JpegDecoder::new(cover_reader);
                    cover_decoder.decode_headers()?;
                    let cover_info = cover_decoder
                        .info()
                        .ok_or_else(|| anyhow!("JPEG: Malformed header info"))?;
                    if !check_number_size_diff(
                        cover_info.width as u32,
                        config.cover_size as u32,
                        0.1,
                    ) {
                        let cover_resp = srv.get_cover_art(&song.id, config.cover_size)?;
                        process_cover(&cover_path, &cover_resp.bytes()?)?;
                        actions.push(Action::CoverDownloaded);
                    }
                } else {
                    let cover_resp = srv.get_cover_art(&song.id, config.cover_size)?;
                    process_cover(&cover_path, &cover_resp.bytes()?)?;
                    actions.push(Action::CoverDownloaded);
                }

                // =======================
                // END OF COVER PROCESSING
                // =======================

                // ========================
                // BEGIN OF SONG PROCESSING
                // ========================
                let mut song_path = album_dir.clone();
                song_path.push(format!(
                    "{:0>3} {}.{}",
                    song.track.unwrap_or(0),
                    sanitize_filename(song.title.clone().into())
                        .to_str()
                        .unwrap(),
                    if config.mp3.is_some() {
                        "mp3"
                    } else {
                        &song.suffix
                    }
                ));

                known_paths.push(song_path.clone());

                // check if existing song has the correct bitrate within a percentage
                if fs::exists(&song_path)? {
                    if let Some(bitrate) = config.mp3 {
                        let mut song_file = File::open(&song_path)?;
                        let mp3 = MpegFile::read_from(
                            &mut song_file,
                            ParseOptions::new().read_tags(false).read_cover_art(false),
                        )?;
                        if check_number_size_diff(
                            bitrate as u32,
                            mp3.properties().audio_bitrate(),
                            0.1,
                        ) {
                            return Ok((known_paths, song.clone(), actions));
                        };
                    } else {
                        return Ok((known_paths, song.clone(), actions));
                    };
                }
                let mut song_stream = srv.get_song(&song.id, config.mp3)?;
                download_file(&mut song_stream, &song_path)?;
                actions.push(Action::SongDownloaded);
                Ok((known_paths, song.clone(), actions))
                // ======================
                // END OF SONG PROCESSING
                // ======================
            },
        )
        .filter_map(|elem| {
            let counter = global_counter.fetch_add(1, Ordering::AcqRel);
            let Ok(result) = elem else {
                return None;
            };
            let song = result.1;
            let song_downloaded = result.2.contains(&Action::SongDownloaded);
            let cov_downloaded = result.2.contains(&Action::CoverDownloaded);
            let count_str = format!(
                "[{:>^width$}/{:>^width$}]",
                counter,
                song_count,
                width = pad_count
            );
            let status_str = match (song_downloaded, cov_downloaded) {
                (true, true) => "Song + Cov :",
                (false, true) => "Cover dwnl.:",
                (true, false) => "Song dwnl. :",
                (false, false) => "All good   :",
            };
            println!(
                "{} {} {} / {} / {}",
                count_str, status_str, song.artist, song.album, song.title,
            );
            Some(result.0)
        })
        .collect();
    new_paths.into_iter().flatten().collect()
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

    let songs = config
        .sync
        .clone()
        .into_iter()
        .filter_map(|element| -> Option<Vec<SubSonicSong>> {
            let (elem_type, elem_id) = element.split_once(".")?;
            println!("element {} {}", elem_type, elem_id);
            match elem_type {
                "playlist" => {
                    let resp = srv.get_playlist(elem_id).ok()?;
                    Some(resp.playlist.songs)
                }
                "album" => {
                    let resp = srv.get_album(elem_id).ok()?;
                    Some(resp.album.songs)
                }
                _ => {
                    println!("ignoring unknown type {}", elem_type);
                    None
                }
            }
        })
        .flatten()
        .collect();

    let mut known_paths = process_songs(songs, &library_dir, config, srv);
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
