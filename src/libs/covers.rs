use crate::libs::{songs::Action, utils::number_good_enough};
use anyhow::{Result, anyhow};
use image::{ImageFormat, codecs::jpeg::JpegEncoder};
use std::{
    fs::{self, File},
    io::BufReader,
    path::Path,
    println,
};
use zune_jpeg::{JpegDecoder, zune_core::bytestream::ZCursor};

pub fn cover_needs_download(
    cover_path: &Path,
    cover_size: u32,
    upgrade_wanted: bool,
) -> anyhow::Result<bool> {
    let cover_exists = fs::exists(cover_path)?;
    if !cover_exists {
        return Ok(true);
    }
    if upgrade_wanted {
        let cover_file = File::open(cover_path)?;
        let cover_reader = BufReader::new(cover_file);
        let mut cover_decoder = JpegDecoder::new(cover_reader);
        cover_decoder.decode_headers()?;
        let cover_info = cover_decoder
            .info()
            .ok_or_else(|| anyhow!("JPEG: Malformed header info"))?;
        return Ok(!number_good_enough(
            cover_info.width as u32,
            cover_size,
            0.1,
        ));
    }
    Ok(false)
}

pub fn process_cover(path: &Path, data: &[u8]) -> Result<Option<Action>> {
    match image::guess_format(data)? {
        ImageFormat::Jpeg => {
            let cover_data_cursor = ZCursor::new(data);
            let mut cover_decoder = JpegDecoder::new(cover_data_cursor);
            cover_decoder.decode_headers()?;
            let cover_info = cover_decoder
                .info()
                .ok_or_else(|| anyhow!("JPEG: Malformed header info"))?;

            if !cover_info.sof.is_sequential_dct() {
                println!("JPEG is not baseline, converting...");
                let cover_rgb8 = image::load_from_memory(data)?.to_rgb8();
                let mut cover_baseline = Vec::new();
                JpegEncoder::new_with_quality(&mut cover_baseline, 90).encode(
                    cover_rgb8.as_raw(),
                    cover_rgb8.width(),
                    cover_rgb8.height(),
                    image::ExtendedColorType::Rgb8,
                )?;
                fs::write(path, &cover_baseline)?;
            } else {
                fs::write(path, data)?;
            }
            Ok(Some(Action::CoverDownloaded))
        }
        _ => Ok(Some(Action::CoverError)),
    }
}
