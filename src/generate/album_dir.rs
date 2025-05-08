use anyhow::anyhow;
use image::ImageReader;
use serde::Serialize;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::slice::Iter;

/// An album directory, which has images and possibly child albums
#[derive(Clone, Serialize)]
pub struct AlbumDir {
    pub path: PathBuf,
    pub images: Vec<Image>,
    pub cover: Image,
    pub children: Vec<AlbumDir>,

    pub description: String,
}

impl AlbumDir {
    // TODO: Add iterator over image dirs
    /// Returns an iterator over all images in the album and subalbums
    pub fn iter_all_images(&self) -> AlbumImageIter {
        AlbumImageIter::new(self)
    }

    /// Create an AlbumDir recursively from a path. The root path is so that we can make every path
    /// relative to the root.
    fn from_path(p: &Path, root: &Path) -> anyhow::Result<Self> {
        let mut images = vec![];
        let mut cover: Option<Image> = None;
        let mut children = vec![];
        let mut description = String::new();

        for entry in p.read_dir()? {
            let entry_path = entry?.path().to_path_buf();

            if entry_path.is_file() {
                if let Some(filename) = entry_path.file_name() {
                    if filename == "description.txt" {
                        description = fs::read_to_string(entry_path)?;
                    } else if filename == "description.md" {
                        log::debug!("Loading Markdown from {}", entry_path.display());
                        let contents = fs::read_to_string(&entry_path)?;
                        let parser = pulldown_cmark::Parser::new(&contents);
                        pulldown_cmark::html::push_html(&mut description, parser);
                    } else {
                        if filename.to_string_lossy().starts_with("cover") {
                            cover = Some(Image::new(
                                entry_path.strip_prefix(root)?.to_path_buf(),
                                String::new(),
                            )?);
                            // Don't include the cover in the set of images
                            continue;
                        }

                        let reader = ImageReader::open(&entry_path)?.with_guessed_format()?;
                        if reader.format().is_some() {
                            // Found an image
                            let mut description = String::new();

                            // Read in any associated description file
                            if entry_path.with_extension("txt").exists() {
                                description = fs::read_to_string(entry_path.with_extension("txt"))?;
                            } else if entry_path.with_extension("md").exists() {
                                log::debug!(
                                    "Loading Markdown from {}",
                                    entry_path.with_extension("md").display()
                                );
                                let contents = fs::read_to_string(entry_path.with_extension("md"))?;
                                let parser = pulldown_cmark::Parser::new(&contents);
                                pulldown_cmark::html::push_html(&mut description, parser);
                            }

                            images.push(Image::new(
                                entry_path.strip_prefix(root)?.to_path_buf(),
                                description,
                            )?);
                        }
                    }
                }
            } else if entry_path.is_dir() {
                if let Some(dirname) = entry_path.file_name().and_then(|n| n.to_str()) {
                    if dirname.starts_with("_") {
                        // Likely a templates or static dir
                        continue;
                    } else if dirname == "site" {
                        // Is a generated site dir, don't descend into it
                        continue;
                    } else if dirname == "slides" {
                        continue;
                    }

                    children.push(AlbumDir::from_path(&entry_path, root)?);
                }
            }
        }

        children.sort_by_key(|c| c.path.clone());
        images.sort_by_key(|i| i.path.clone());

        // Find a cover image if we didn't have an explicit one. Either the first image, or the
        // first image from the first album that has a cover.
        if cover.is_none() {
            if !images.is_empty() {
                cover = Some(images[0].clone());
            } else {
                // Find a cover image from one of the children
                if !children.is_empty() {
                    cover = Some(children[0].cover.clone());
                }
            }
        }
        let cover = cover.ok_or(anyhow!("Could not find a cover image for {}", p.display()))?;
        log::debug!("Cover for {} is {cover:?}", p.display());

        Ok(AlbumDir {
            path: p.strip_prefix(root)?.to_path_buf(),
            images,
            cover,
            children,
            description,
        })
    }
}

impl TryFrom<&PathBuf> for AlbumDir {
    type Error = anyhow::Error;

    fn try_from(p: &PathBuf) -> anyhow::Result<AlbumDir> {
        AlbumDir::from_path(p, p)
    }
}

/// An iterator which walks through all of the images in an album, and its sub-albums
pub struct AlbumImageIter<'a> {
    image_iter: Box<dyn Iterator<Item = &'a Image> + 'a>,
    children_iter: Iter<'a, AlbumDir>,
}

impl<'a> AlbumImageIter<'a> {
    fn new(ad: &'a AlbumDir) -> Self {
        Self {
            image_iter: Box::new(ad.images.iter()),
            children_iter: ad.children.iter(),
        }
    }
}

impl<'a> Iterator for AlbumImageIter<'a> {
    type Item = &'a Image;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(img) = self.image_iter.next() {
            return Some(img);
        }

        for album in self.children_iter.by_ref() {
            // Set the child album as the current image iterator
            self.image_iter = Box::new(album.iter_all_images());
            // If we found a child album with an image, return the image. Otherwise we'll keep
            // iterating over children.
            if let Some(i) = self.image_iter.next() {
                return Some(i);
            }
        }

        None
    }
}

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
    use std::collections::HashSet;

    #[test]
    fn basic_album_iter() {
        let mut ad = AlbumDir {
            path: "".into(),
            description: "".to_string(),
            cover: Image::new("foo".into(), "".to_string()).unwrap(),
            images: vec![
                Image::new("foo".into(), "".to_string()).unwrap(),
                Image::new("bar".into(), "".to_string()).unwrap(),
            ],
            children: vec![],
        };
        // A child album with some images
        ad.children.push(AlbumDir {
            path: "subdir".into(),
            description: "".to_string(),
            cover: Image::new("subdir/foo".into(), "".to_string()).unwrap(),
            images: vec![
                Image::new("subdir/foo".into(), "".to_string()).unwrap(),
                Image::new("subdir/bar".into(), "".to_string()).unwrap(),
            ],
            children: vec![AlbumDir {
                path: "subdir/deeper_subdir".into(),
                description: "".to_string(),
                cover: Image::new("subdir/deeper_subdir/image.jpg".into(), "".to_string()).unwrap(),
                images: vec![
                    Image::new("subdir/deeper_subdir/image.jpg".into(), String::new()).unwrap(),
                ],
                children: vec![],
            }],
        });
        // A child album with no images
        ad.children.push(AlbumDir {
            description: "".to_string(),
            cover: Image::new("blah".into(), "".to_string()).unwrap(),
            path: "another_subdir".into(),
            images: vec![],
            children: vec![],
        });

        let imgs: HashSet<&str> = ad
            .iter_all_images()
            .map(|i| i.path.to_str().unwrap())
            .collect();
        let expected: HashSet<&str> = HashSet::from([
            "foo",
            "bar",
            "subdir/foo",
            "subdir/bar",
            "subdir/deeper_subdir/image.jpg",
        ]);
        assert_eq!(imgs, expected);
    }

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
