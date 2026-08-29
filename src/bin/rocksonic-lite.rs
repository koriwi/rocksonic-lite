use anyhow::{Result, anyhow};
use clap::Parser;
use rocksonic_lite::{SyncEvent, sync};
use std::{
    format,
    fs::{self},
    path::Path,
    println,
};

fn on_event(event: SyncEvent) {
    let string = match event {
        SyncEvent::Started { total } => format!("Found {} songs. Processing them now.", total),
        SyncEvent::FileDeleted(p) => format!("Deleting outdated file {}", p.to_str().unwrap()),
        SyncEvent::SongFinished {
            current,
            total,
            artist,
            album,
            title,
            song_downloaded,
            cover_downloaded,
            cover_error,
        } => {
            let pad_count = total.to_string().len();
            let count_str = format!("[{:>width$}/{}]", current, total, width = pad_count);
            let mut status_str = String::from("");

            status_str += if song_downloaded {
                "🎵⌛"
            } else {
                "🎵✔️"
            };
            status_str += if cover_downloaded {
                " 📷⌛"
            } else if cover_error {
                " 📷⚠️"
            } else {
                " 📷✔️"
            };

            format!(
                "{} {} {} / {} / {}",
                count_str, status_str, artist, album, title,
            )
        }
        _ => String::from("unknown event, skipping..."),
    };
    println!("{}", string);
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
    sync::run_sync(config_path, on_event)?;
    Ok(())
}
