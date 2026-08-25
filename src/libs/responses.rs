use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct SubSonicError {
    #[serde(rename = "@code")]
    pub code: u16,
    #[serde(rename = "@message")]
    pub message: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename = "subsonic-response")]
pub struct SubSonicErrorResponse {
    #[serde(rename = "@status")]
    pub status: String,
    pub error: Option<SubSonicError>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct SubSonicSong {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@title")]
    pub title: String,
    #[serde(rename = "@track")]
    pub track: Option<u16>,
    #[serde(rename = "@album")]
    pub album: String,
    #[serde(rename = "@artist")]
    pub artist: String,
    #[serde(rename = "@suffix")]
    pub suffix: String,
    #[serde(rename = "@size")]
    pub size: u64,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct SubSonicPlaylist {
    #[serde(rename = "entry")]
    pub songs: Vec<SubSonicSong>,
    #[serde(rename = "@name")]
    pub name: String,
}
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename = "subsonic-response")]
pub struct SubSonicPlaylistResponse {
    pub playlist: SubSonicPlaylist,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct SubSonicAlbum {
    #[serde(rename = "song")]
    pub songs: Vec<SubSonicSong>,
    #[serde(rename = "@name")]
    pub name: String,
}
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename = "subsonic-response")]
pub struct SubSonicAlbumResponse {
    pub album: SubSonicAlbum,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct SubSonicStarred {
    #[serde(rename = "song")]
    pub songs: Vec<SubSonicSong>,
}
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename = "subsonic-response")]
pub struct SubSonicStarredResponse {
    pub starred: SubSonicStarred,
}
