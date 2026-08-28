use std::{ffi::OsString, fs::File, io, path::PathBuf};

use anyhow::Result;
use reqwest::blocking::Response;

const SANITIZE_OPTIONS: sanitize_filename::Options = sanitize_filename::Options {
    truncate: true,
    windows: true,
    replacement: "",
};

pub fn sanitize_filename(file_name: OsString) -> OsString {
    sanitize_filename::sanitize_with_options(file_name.to_str().unwrap(), SANITIZE_OPTIONS).into()
}

pub fn download_file(req_res: &mut Response, file_path: &PathBuf) -> Result<()> {
    let mut file = File::create(file_path)?;
    io::copy(req_res, &mut file)?;
    Ok(())
}

// returns the percentage
pub fn number_good_enough(num_a: u32, num_b: u32, max_diff: f32) -> bool {
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
