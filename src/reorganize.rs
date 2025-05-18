use anyhow::{anyhow, Context};
use image::ImageReader;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::str::from_utf8;
use thiserror::Error;
use time::macros::format_description;
use time::{OffsetDateTime, PrimitiveDateTime, UtcDateTime};

#[derive(Error, Debug)]
pub enum OrganizeError {
    #[error("These files are not supported, unable to parse EXIF data: {0:?}")]
    ExifNotSupported(Vec<PathBuf>),
    #[error("File {0} is missing an EXIF DateTimeOriginal field")]
    ExifNoDateTime(PathBuf),
}

pub fn reorganize(dir: &Path, dry_run: bool) -> anyhow::Result<()> {
    let renames = get_renames(dir);

    // Either do the renames, or if dry-run print what the names would be

    Ok(())
}

/// Returns a vec of tuples of all the renames that need to happen in a directory
fn get_renames(dir: &Path) -> anyhow::Result<Vec<(PathBuf, PathBuf)>> {
    let mut renames: Vec<(PathBuf, PathBuf)> = Vec::new();

    // Run through all the images and figure out new names for them
    for entry in dir.read_dir()? {
        let entry = entry?;

        // Only bother with image files, because we return an error if we fail to read EXIF
        let is_image: bool = ImageReader::open(entry.path())?
            .with_guessed_format()?
            .format()
            .is_some();

        let is_cover: bool = entry
            .path()
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with("cover"));

        if is_image && !is_cover {
            // TODO: Should we just skip over images with no EXIF data? Find datetime some other
            // way?
            let Ok(dt) = get_exif_datetime(entry.path()) else {
                log::warn!(
                    "Unable to read datetime from EXIF for {}",
                    entry.path().display()
                );
                continue;
            };
            let ext = entry
                .path()
                .extension()
                .ok_or(anyhow!(
                    "{} is missing an extension",
                    entry.path().display()
                ))?
                .to_string_lossy()
                .to_string();

            let new_filename_base = dt.format(format_description!(
                "[year][month][day]_[hour][minute][second]"
            ))?;
            let new_path = entry
                .path()
                .with_file_name(new_filename_base)
                .with_extension(ext);

            renames.push((entry.path(), new_path.clone()));

            // Check for files associated with this image and set them up to be renamed too, like
            // description files that end with .txt or .md
            for ext in ["txt", "md"] {
                let side_file_path = entry.path().with_extension(ext);
                if side_file_path.exists() {
                    let new_side_file_path = new_path.with_extension(ext);
                    renames.push((side_file_path, new_side_file_path));
                }
            }
        }
    }
    Ok(renames)
}

/// Tries to figure out the datetime that the image was created from EXIF metadata
fn get_exif_datetime(path: PathBuf) -> anyhow::Result<UtcDateTime> {
    let format_with_offset = format_description!(
        "[year]:[month]:[day] [hour]:[minute]:[second][offset_hour]:[offset_minute]"
    );
    let format_without_offset =
        format_description!(version = 2, "[year]:[month]:[day] [hour]:[minute]:[second]");

    let file = File::open(&path).with_context(|| format!("Couldn't open {}", path.display()))?;
    let mut bufreader = BufReader::new(file);
    // TODO: Return a better error if EXIF is not supported
    let exif = exif::Reader::new()
        .read_from_container(&mut bufreader)
        .with_context(|| format!("Couldn't read EXIF data from {}", path.display()))?;
    let field = exif
        .get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY)
        .ok_or(OrganizeError::ExifNoDateTime(path.clone()))?;

    let dt: UtcDateTime = match &field.value {
        exif::Value::Ascii(v) => {
            let s = from_utf8(&v[0])?;
            log::debug!("Date string from file: {s}");

            match OffsetDateTime::parse(&s, format_with_offset) {
                Ok(v) => v.to_utc(),
                Err(_) => PrimitiveDateTime::parse(&s, format_without_offset)?.as_utc(),
            }
        }
        // TODO: return some error
        _ => todo!(),
    };

    Ok(dt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{init, make_test_album};
    use time::{Date, Month, Time};

    #[test]
    /// Make sure we can get the datetime from one of our test photos
    fn basic_datetime_read() {
        init();
        let dt = get_exif_datetime("resources/test_album/moon.jpg".into()).unwrap();
        log::info!("Got dt: {dt}");
        assert_eq!(
            dt,
            UtcDateTime::new(
                Date::from_calendar_date(1970, Month::January, 1).unwrap(),
                Time::from_hms(13, 37, 0).unwrap(),
            )
        )
    }

    #[test]
    fn exif_datetime_missing() {
        init();
        let result = get_exif_datetime("resources/test_album/mountains.jpg".into());
        assert!(result.is_err());
        //result.unwrap();
    }

    #[test]
    fn basic_renames() {
        init();
        let tmp_album_dir = make_test_album();
        let dir = tmp_album_dir.join("with_description");

        log::debug!("Getting renames for {}", dir.display());
        let renames = get_renames(&dir).unwrap();

        assert_eq!(
            renames,
            vec![
                (dir.join("mountains.jpg"), dir.join("19700103_133700.jpg")),
                (dir.join("moon.jpg"), dir.join("19700102_133700.jpg")),
                (dir.join("moon.txt"), dir.join("19700102_133700.txt")),
            ]
        );
    }
}
