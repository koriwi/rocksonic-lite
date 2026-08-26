use crate::{
    Config,
    libs::{
        covers::process_cover,
        responses::SubSonicSong,
        server::Server,
        utils::{check_number_size_diff, download_file, sanitize_filename},
    },
};
use anyhow::{Result, anyhow};
use clap::Parser;
use id3::{Tag, TagLike, no_tag_ok};
use lofty::{config::ParseOptions, file::AudioFile, mpeg::MpegFile};
use rayon::prelude::*;
use std::{
    fmt, format,
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
    println, vec, write,
};
use zune_jpeg::JpegDecoder;

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

fn strip_mp3_artwork(path: &Path) -> Result<bool> {
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

pub fn get_song_lists(config: &Config, srv: &Server) -> Vec<(Option<String>, Vec<SubSonicSong>)> {
    config
        .sync
        .clone()
        .into_iter()
        .filter_map(|element| -> Option<(Option<String>, Vec<SubSonicSong>)> {
            let (elem_type, elem_id) = element.split_once(".")?;
            println!("element {} {}", elem_type, elem_id);
            match elem_type {
                "playlist" => {
                    let resp = srv.get_playlist(elem_id).ok()?;
                    Some((Some(resp.playlist.name), resp.playlist.songs))
                }
                "album" => {
                    let resp = srv.get_album(elem_id).ok()?;
                    Some((None, resp.album.songs))
                }
                _ => {
                    println!("ignoring unknown type {}", elem_type);
                    None
                }
            }
        })
        .collect()
}

pub fn process_songs<F>(
    songs: &Vec<SubSonicSong>,
    library_dir: &Path,
    cover_size: u16,
    mp3: Option<u16>,
    srv: &Server,
    log_status: F,
) -> (Vec<PathBuf>, Vec<PathBuf>)
where
    F: Fn(SubSonicSong, bool, bool, bool) + Sync,
{
    let new_paths: Vec<(Vec<PathBuf>, PathBuf)> = songs
        .par_iter()
        .map(
            |song| -> anyhow::Result<(Vec<PathBuf>, SubSonicSong, Vec<Action>, PathBuf)> {
                let mut known_paths = vec![];
                let mut actions = vec![];
                let mut album_dir = library_dir.to_path_buf();
                album_dir.push(sanitize_filename(song.artist.clone().into()));
                known_paths.push(album_dir.clone());
                album_dir.push(sanitize_filename(song.album.clone().into()));
                known_paths.push(album_dir.clone());

                if !fs::exists(&album_dir)? {
                    fs::create_dir_all(&album_dir)?;
                }

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
                    if !check_number_size_diff(cover_info.width as u32, cover_size as u32, 0.1) {
                        let cover_resp = srv.get_cover_art(&song.id, cover_size)?;
                        if let Some(cover_action) =
                            process_cover(&cover_path, &cover_resp.bytes()?)?
                        {
                            actions.push(cover_action);
                        }
                    }
                } else {
                    let cover_resp = srv.get_cover_art(&song.id, cover_size)?;
                    if let Some(cover_action) = process_cover(&cover_path, &cover_resp.bytes()?)? {
                        actions.push(cover_action);
                    }
                }

                let mut song_path = album_dir.clone();
                song_path.push(format!(
                    "{:0>3} {}.{}",
                    song.track.unwrap_or(0),
                    sanitize_filename(song.title.clone().into())
                        .to_str()
                        .unwrap(),
                    if mp3.is_some() { "mp3" } else { &song.suffix }
                ));
                known_paths.push(song_path.clone());
                let audio_path = song_path.clone();
                // check if existing song has the correct bitrate within a percentage
                if fs::exists(&song_path)? {
                    if let Some(bitrate) = mp3 {
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
                            return Ok((known_paths, song.clone(), actions, audio_path));
                        };
                    } else {
                        return Ok((known_paths, song.clone(), actions, audio_path));
                    };
                }
                let mut song_stream = srv.get_song(&song.id, mp3)?;
                download_file(&mut song_stream, &song_path)?;
                strip_mp3_artwork(&song_path)?;
                actions.push(Action::SongDownloaded);
                Ok((known_paths, song.clone(), actions, audio_path))
            },
        )
        .filter_map(|elem| {
            let Ok(result) = elem else {
                return None;
            };
            let song = result.1;
            let song_downloaded = result.2.contains(&Action::SongDownloaded);
            let cov_downloaded = result.2.contains(&Action::CoverDownloaded);
            let cov_error = result.2.contains(&Action::CoverError);
            log_status(song, song_downloaded, cov_downloaded, cov_error);
            Some((result.0, result.3))
        })
        .collect();

    let mut all_paths = vec![];
    let mut audio_paths = vec![];
    new_paths.into_iter().for_each(|mut np| {
        all_paths.append(&mut np.0);
        audio_paths.push(np.1)
    });
    (all_paths, audio_paths)
}
