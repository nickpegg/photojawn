use anyhow::anyhow;
use serde::Serialize;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize)]
pub struct Image {
    /// Path to the image, relative to the root album
    pub path: PathBuf,
    pub filename: String,

    /// Text description of the image which is displayed below it on the HTML page
    pub description: String,

    pub thumb_filename: String,
    pub thumb_path: PathBuf,
    pub screen_filename: String,
    pub screen_path: PathBuf,
    pub html_filename: String,
    pub html_path: PathBuf,
}
impl Image {
    pub fn new(path: PathBuf, description: String) -> anyhow::Result<Self> {
        let filename = path
            .file_name()
            .ok_or(anyhow!(
                "Image path {} is missing a filename",
                path.display()
            ))?
            .to_str()
            .ok_or(anyhow!("Cannot convert {} to a string", path.display()))?
            .to_string();
        let thumb_filename = Self::slide_filename(&path, "thumb", true)?;
        let thumb_path = Self::slide_path(&path, &thumb_filename);
        let screen_filename = Self::slide_filename(&path, "screen", true)?;
        let screen_path = Self::slide_path(&path, &screen_filename);
        let html_filename = Self::slide_filename(&path, "html", false)?;
        let html_path = Self::slide_path(&path, &html_filename);

        Ok(Image {
            path,
            description,
            filename,
            thumb_filename,
            thumb_path,
            screen_filename,
            screen_path,
            html_filename,
            html_path,
        })
    }

    /// Returns the filename for a given slide type. For example if ext = "thumb" and the current
    /// filename is "blah.jpg" this will return "blah.thumb.jpg". If keep_ext if false, it would
    /// return "blah.thumb"
    fn slide_filename(path: &Path, ext: &str, keep_ext: bool) -> anyhow::Result<String> {
        let mut new_ext: OsString = ext.into();
        if keep_ext {
            if let Some(e) = path.extension() {
                new_ext = OsString::from(
                    ext.to_string()
                        + "."
                        + e.to_str().ok_or(anyhow!(
                            "Image {} extension is not valid UTF-8",
                            path.display()
                        ))?,
                )
            }
        }

        let new_path = path.with_extension(new_ext);
        let new_name = new_path
            .file_name()
            .ok_or(anyhow!("Image {} missing a file name", path.display()))?
            .to_str()
            .ok_or(anyhow!("Unable to convert {} to a string", path.display()))?
            .to_string();

        Ok(new_name)
    }

    /// Returns the path to the file in the slides dir given the path to the original image
    fn slide_path(path: &Path, file_name: &str) -> PathBuf {
        let mut new_path = path.to_path_buf();
        new_path.pop();
        new_path.push("slides");
        new_path.push(file_name);
        new_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_paths() {
        let img = Image::new(PathBuf::from("foo/bar/image.jpg"), String::new()).unwrap();
        assert_eq!(
            img.thumb_path,
            PathBuf::from("foo/bar/slides/image.thumb.jpg")
        );
        assert_eq!(
            img.screen_path,
            PathBuf::from("foo/bar/slides/image.screen.jpg")
        );
    }
}
