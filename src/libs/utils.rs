use std::{fs::File, io, path::PathBuf};

use anyhow::Result;
use reqwest::blocking::Response;
const SANITIZE_OPTIONS: sanitize_filename::Options = sanitize_filename::Options {
    truncate: true,
    windows: false,
    replacement: "",
};

pub fn sanitize_filename(file_name: &str) -> String {
    sanitize_filename::sanitize_with_options(file_name, SANITIZE_OPTIONS)
}

pub fn download_file(req_res: &mut Response, file_path: &PathBuf) -> Result<()> {
    let mut file = File::create(file_path)?;
    io::copy(req_res, &mut file)?;
    Ok(())
}
