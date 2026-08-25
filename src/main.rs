pub mod libs;
use libs::server;

use std::{fs, path::Path, println, process::exit};

use anyhow::{Result, anyhow};
use clap::Parser;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Config<'a> {
    server_url: &'a str,
    user: &'a str,
    password: &'a str,
    mp3: Option<u16>,
    cover_size: Option<u32>,
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

    let config_path = Path::new(args.config.as_str());
    let config_exists = fs::exists(config_path)?;
    if !config_exists {
        println!("Could not find the config file {}", args.config);
        exit(1)
    }
    let config_file = fs::read_to_string(config_path)?;
    let config: Config = yaml_serde::from_str(&config_file)?;
    let srv = server::Server::connect(config.server_url, config.user, config.password)?;

    let config_file_path = config_path.with_file_name("");
    let config_file_name = config_path
        .file_stem()
        .ok_or_else(|| anyhow!("config file name is too funky"))?;
    let mut library_dir = config_file_path.clone();
    library_dir.push(config_file_name);
    println!("root config path {}", library_dir.to_str().unwrap(),);

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
        if let Some(songs) = songs {
            // println!("songs for {elem_type} {elem_id} {:?}", songs);
        }
    }

    Ok(())
}
