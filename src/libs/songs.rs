use crate::{
    config::Config,
    core::{responses::SubSonicSong, server::Server, utils::number_good_enough},
};
use anyhow::Result;
use clap::Parser;
use id3::{Tag, TagLike, no_tag_ok};
use lofty::{config::ParseOptions, file::AudioFile, mpeg::MpegFile};
use std::{
    fmt,
    fs::{self, File},
    path::Path,
    println, write,
};

#[derive(Parser, Debug, PartialEq, Eq)]
pub enum Action {
    SongDownloaded,
    CoverDownloaded,
    CoverError,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn strip_mp3_artwork(path: &Path) -> Result<bool> {
    let Some(mut tag) = no_tag_ok(Tag::read_from_path(path))? else {
        return Ok(false);
    };

    if tag.pictures().next().is_none() {
        return Ok(false);
    }

    let version = tag.version();

    tag.remove_all_pictures();
    tag.write_to_path(path, version)?;

    Ok(true)
}

pub fn song_needs_download(
    song_path: &Path,
    mp3: Option<u16>,
    upgrade_wanted: bool,
) -> anyhow::Result<bool> {
    // check if existing song has the correct bitrate within a percentage
    if !fs::exists(song_path)? {
        return Ok(true);
    };
    if !upgrade_wanted {
        return Ok(false);
    }
    let Some(bitrate) = mp3 else { return Ok(false) };
    let mut song_file = File::open(song_path)?;
    let mp3 = MpegFile::read_from(
        &mut song_file,
        ParseOptions::new().read_tags(false).read_cover_art(false),
    )?;
    if number_good_enough(bitrate as u32, mp3.properties().audio_bitrate(), 0.1) {
        // return Ok((known_paths, song.clone(), actions, audio_path));
        return Ok(false);
    };
    Ok(true)
}

pub struct SongList {
    pub name: Option<String>,
    pub songs: Vec<SubSonicSong>,
}

pub fn get_song_lists(config: &Config, srv: &Server) -> Vec<SongList> {
    config
        .sync
        .clone()
        .into_iter()
        .filter_map(|element| -> Option<SongList> {
            let (elem_type, elem_id) = element.split_once(".")?;
            println!("element {} {}", elem_type, elem_id);
            match elem_type {
                "playlist" => {
                    let resp = srv.get_playlist(elem_id).ok()?;
                    Some(SongList {
                        name: Some(resp.playlist.name),
                        songs: resp.playlist.songs,
                    })
                }
                "album" => {
                    let resp = srv.get_album(elem_id).ok()?;
                    Some(SongList {
                        name: None,
                        songs: resp.album.songs,
                    })
                }
                _ => {
                    println!("ignoring unknown type {}", elem_type);
                    None
                }
            }
        })
        .collect()
}
