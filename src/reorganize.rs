use anyhow::Context;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::str::from_utf8;
use thiserror::Error;
use time::macros::format_description;
use time::{OffsetDateTime, PrimitiveDateTime};

#[derive(Error, Debug)]
pub enum OrganizeError {
    #[error("These files are not supported, unable to parse EXIF data: {0:?}")]
    ExifNotSupported(Vec<PathBuf>),
    #[error("File {0} is missing an EXIF DateTimeOriginal field")]
    ExifNoDateTime(PathBuf),
}

pub fn reorganize(dir: &Path, dry_run: bool) -> anyhow::Result<()> {
    // Run through all the images and figure out new names for them
    for entry in dir.read_dir()? {
        let entry = entry?;
        if entry.path().is_file() {
            let dt = get_exif_datetime(entry.path())?;
            todo!();
        }
    }

    // Either do the renames, or if dry-run print what the names would be

    Ok(())
}

/// Tries to figure out the datetime that t
fn get_exif_datetime(path: PathBuf) -> anyhow::Result<()> {
    let DT_WITH_OFFSET = format_description!(
        "[year]:[month]:[day] [hour]:[minute]:[second][offset_hour]:[offset_minute]"
    );
    let DT_WITHOUT_OFFSET =
        format_description!(version = 2, "[year]:[month]:[day] [hour]:[minute]:[second]");

    let file = File::open(&path).with_context(|| format!("Couldn't open {}", path.display()))?;
    let mut bufreader = BufReader::new(file);
    // TODO: Return a better error if EXIF is not supported
    let exif = exif::Reader::new().read_from_container(&mut bufreader)?;
    let field = exif
        .get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY)
        .ok_or(OrganizeError::ExifNoDateTime(path.clone()))?;

    let dt = match &field.value {
        exif::Value::Ascii(v) => {
            let s = from_utf8(&v[0])?;
            log::debug!("Date string: {s}");
            log::debug!("{DT_WITH_OFFSET:?}");
            match OffsetDateTime::parse(&s, DT_WITH_OFFSET) {
                Ok(v) => v,
                Err(_) => {
                    log::debug!("Unable to parse {s} with offset");
                    PrimitiveDateTime::parse(&s, DT_WITHOUT_OFFSET)?
                }
            }
        }
        _ => todo!(),
    };
    println!("{dt:?}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    /// Make sure we can get the datetime from one of our test photos
    fn basic_datetime_read() {
        init();
        let dt = get_exif_datetime("resources/test_album/moon.jpg".into()).unwrap();
        todo!();
    }

    #[test]
    fn exif_datetime_missing() {
        init();
        let result = get_exif_datetime("resources/test_album/mountains.jpg".into());
        assert!(result.is_err());
        //result.unwrap();
    }
}
