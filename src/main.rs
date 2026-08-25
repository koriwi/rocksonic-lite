pub mod libs;
use image::{ImageFormat, codecs::jpeg::JpegEncoder};
use libs::{server, utils};
use zune_jpeg::{JpegDecoder, Marker::SOF, zune_core::bytestream::ZCursor};

use std::{
    fs::{self, read},
    io::Cursor,
    path::Path,
    println,
    process::exit,
};

use anyhow::{Result, anyhow};
use clap::Parser;
use serde::Deserialize;

use crate::libs::utils::sanitize_filename;
// returns the percentage
fn check_cover_size_diff(num_a: u16, num_b: u16, max_diff: f32) -> bool {
    let mut diff = 0;
    let mut abs_max_diff = 0;
    if num_a < num_b {
        diff = num_b - num_a;
        abs_max_diff = (max_diff * (num_b as f32)) as u16;
    }
    if num_a > num_b {
        diff = num_a - num_b;
        abs_max_diff = (max_diff * (num_a as f32)) as u16;
    }
    diff <= abs_max_diff
}

#[derive(Deserialize, Debug)]
struct Config<'a> {
    server_url: &'a str,
    user: &'a str,
    password: &'a str,
    mp3: Option<u16>,
    cover_size: Option<u16>,
    sync: Vec<&'a str>,
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
        println!("Could not find the config file {}", args.config);
        exit(1)
    }
    let config_file = fs::read_to_string(config_path)?;
    let config: Config = yaml_serde::from_str(&config_file)?;

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

    // debug stuff
    for element in config.sync {
        let (elem_type, elem_id) = element.split_once(".").ok_or_else(|| {
            anyhow!("malformed sync element (use playlist.123-abc, album.abc-123)")
        })?;
        println!("element {} {}", elem_type, elem_id);
        let songs = match elem_type {
            "playlist" => {
                let resp = srv.get_playlist(elem_id)?;
                Some(resp.playlist.songs)
            }
            "album" => {
                let resp = srv.get_album(elem_id)?;
                Some(resp.album.songs)
            }
            _ => {
                println!("ignoring unknown type {}", elem_type);
                None
            }
        };
        let mut i = 0;
        if let Some(songs) = songs {
            for song in songs {
                let requested_cover_size = config.cover_size.unwrap_or(300);
                let cover_resp = srv.get_cover_art(&song.id, requested_cover_size)?;

                let mut cover_path = library_dir.clone();
                cover_path.push(sanitize_filename(&song.artist));
                cover_path.push(sanitize_filename(&song.album));

                if !fs::exists(&cover_path)? {
                    fs::create_dir_all(&cover_path)?;
                }

                cover_path.push("cover.jpeg");

                // check if existing cover has the correct size within a percentage
                if fs::exists(&cover_path)? {
                    let cover_bytes = fs::read(&cover_path)?;
                    let cover_cursor = Cursor::new(cover_bytes);
                    let mut cover_decoder = JpegDecoder::new(cover_cursor);
                    cover_decoder.decode_headers()?;
                    let cover_info = cover_decoder
                        .info()
                        .ok_or_else(|| anyhow!("JPEG: Malformed header info"))?;
                    if check_cover_size_diff(cover_info.width, requested_cover_size, 0.1) {
                        continue;
                    }
                    println!(
                        "found cover, but outdated width, should be {}, is {}",
                        requested_cover_size, cover_info.width
                    );
                }

                let cover_bytes = cover_resp.bytes()?;

                match image::guess_format(&cover_bytes)? {
                    ImageFormat::Jpeg => {
                        let cover_data_cursor = ZCursor::new(&cover_bytes);
                        let mut cover_decoder = JpegDecoder::new(cover_data_cursor);
                        cover_decoder.decode_headers()?;
                        let cover_info = cover_decoder
                            .info()
                            .ok_or_else(|| anyhow!("JPEG: Malformed header info"))?;

                        if !cover_info.sof.is_sequential_dct() {
                            println!("JPEG is not baseline, converting...");
                            let cover_rgb8 = image::load_from_memory(&cover_bytes)?.to_rgb8();
                            let mut cover_baseline = Vec::new();
                            JpegEncoder::new_with_quality(&mut cover_baseline, 90).encode(
                                cover_rgb8.as_raw(),
                                cover_rgb8.width(),
                                cover_rgb8.height(),
                                image::ExtendedColorType::Rgb8,
                            )?;
                            fs::write(&cover_path, &cover_baseline)?;
                            return Ok(());
                        } else {
                            fs::write(&cover_path, &cover_bytes)?
                        }
                    }
                    _ => println!("image format not supported yet"),
                };

                i += 1;
                if i == 1000 {
                    break;
                }
            }
            // println!("songs for {elem_type} {elem_id} {:?}", songs);
        }
    }

    Ok(())
}
