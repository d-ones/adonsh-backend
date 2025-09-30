use crate::transit_authorities::SupportedTransitAuthorities;
use anyhow::Result;
use reqwest;
use std::fs;
use std::io::{self, Cursor};
use std::path::Path;
use zip::ZipArchive;

pub fn download_and_extract(ta: SupportedTransitAuthorities) -> Result<()> {
    let output_path = Path::new(ta.get_static_download_dir());
    fs::create_dir_all(&output_path)?;

    let mut response = reqwest::blocking::get(ta.download_url())?;
    let mut buffer = Vec::new();
    response.copy_to(&mut buffer)?;
    let cursor = Cursor::new(buffer);
    let mut archive = ZipArchive::new(cursor)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => output_path.join(path),
            None => continue,
        };

        if file.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

pub fn cleanup_gtfs_files(ta: SupportedTransitAuthorities) -> Result<()> {
    Ok(fs::remove_dir_all(ta.get_static_download_dir())?)
}
