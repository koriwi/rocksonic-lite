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

/// tells you the percentage difference from the larger number to the smaller number
/// ```
/// use rocksonic_lite::core::utils::percentage_diff;
///
/// let diff = percentage_diff(10, 9);
/// approx::assert_relative_eq!(diff, 0.1);
///
/// let diff = percentage_diff(9, 10);
/// approx::assert_relative_eq!(diff, 0.1);
/// ```
pub fn percentage_diff(num_a: u32, num_b: u32) -> f32 {
    if num_a == num_b {
        return 0.0;
    }
    if num_a < num_b {
        1.0 - (num_a as f32) / (num_b as f32)
    } else {
        1.0 - (num_b as f32) / (num_a as f32)
    }
}
