use std::{format, time::Duration};

use crate::core::responses::{
    SubSonicAlbumResponse, SubSonicErrorResponse, SubSonicPlaylistResponse, SubSonicSong,
    SubSonicStarredResponse,
};
use anyhow::{Result, anyhow};
use reqwest::blocking::Response;

pub struct Server<'a> {
    client: reqwest::blocking::Client,
    host: &'a str,
    username: &'a str,
    password: &'a str,
}

impl<'a> Server<'a> {
    fn get(&self, endpoint: &str, params: &[(&str, String)]) -> Result<Response> {
        let url = format!("{}/rest/{endpoint}", self.host.trim_end_matches("/"));

        let res = self
            .client
            .get(url)
            .query(&[
                ("v", "1.16.1"),
                ("c", "rocksonic-lite"),
                ("u", self.username),
                ("p", self.password),
            ])
            .query(params)
            .send()?
            .error_for_status()?;
        Ok(res)
    }

    fn test_connection(&self) -> Result<()> {
        let response = self.get("ping", &[])?;
        let status = response.status();
        let text = response.text()?;
        let xml = serde_xml_rs::from_str::<SubSonicErrorResponse>(&text)
            .map_err(|_e| anyhow!(format!("status {}\n{}", status, text)))?;
        if xml.status != "ok" {
            return match xml.error {
                Some(error) => Result::Err(anyhow!(error.message)),
                None => Result::Err(anyhow!(status)),
            };
        }
        Ok(())
    }

    pub fn get_cover_art(&self, id: &str, size: u16) -> Result<Response> {
        let response = self.get(
            "getCoverArt",
            &[("id", id.to_string()), ("size", size.to_string())],
        )?;

        if let Some(content_type) = response.headers().get("Content-Type")
            && content_type == "text/xml"
        {
            let xml = serde_xml_rs::from_str::<SubSonicErrorResponse>(&response.text()?)?;
            let error_message = xml.error.ok_or(anyhow!("unknown error"))?.message;
            return Result::Err(anyhow!(error_message));
        };
        Ok(response)
    }

    pub fn get_song(&self, id: &str, mp3: Option<u16>) -> Result<Response> {
        let (endpoint, params) = song_request(id, mp3);
        let response = self.get(endpoint, &params)?;

        if let Some(content_type) = response.headers().get("Content-Type")
            && content_type == "text/xml"
        {
            let xml = serde_xml_rs::from_str::<SubSonicErrorResponse>(&response.text()?)?;
            let error_message = xml.error.ok_or(anyhow!("unknown error"))?.message;
            return Result::Err(anyhow!(error_message));
        };
        Ok(response)
    }

    pub fn get_playlist(&self, playlist_id: &str) -> Result<SubSonicPlaylistResponse> {
        let response = self.get("getPlaylist", &[("id", playlist_id.to_string())])?;
        let xml = serde_xml_rs::from_str::<SubSonicPlaylistResponse>(&response.text()?)?;
        Ok(xml)
    }
    pub fn get_album(&self, album_id: &str) -> Result<SubSonicAlbumResponse> {
        let response = self.get("getAlbum", &[("id", album_id.to_string())])?;
        let xml = serde_xml_rs::from_str::<SubSonicAlbumResponse>(&response.text()?)?;
        Ok(xml)
    }
    pub fn get_favs(&self) -> Result<Vec<SubSonicSong>> {
        let response = self.get("getStarred", &[])?;
        let xml = serde_xml_rs::from_str::<SubSonicStarredResponse>(&response.text()?)?;
        Ok(xml.starred.songs)
    }

    pub fn connect(host: &'a str, username: &'a str, password: &'a str) -> Result<Self> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(300))
            .http1_only()
            .connect_timeout(Duration::from_secs(15))
            .build()?;
        let server = Server {
            client,
            host,
            username,
            password,
        };
        server.test_connection().map(|()| server)
    }
}

fn song_request(id: &str, mp3: Option<u16>) -> (&'static str, Vec<(&str, String)>) {
    let mut params = vec![("id", id.to_owned())];
    match mp3 {
        Some(bitrate) => {
            params.push(("maxBitRate", bitrate.to_string()));
            params.push(("format", "mp3".to_owned()));
            ("stream", params)
        }
        None => ("download", params),
    }
}
