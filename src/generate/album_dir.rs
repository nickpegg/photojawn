use anyhow::anyhow;
use image::ImageReader;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::slice::Iter;

/// An album directory, which has images and possibly child albums
#[derive(Clone, Serialize)]
pub struct AlbumDir {
    pub path: PathBuf,
    pub images: Vec<Image>,
    // TOOD: Remove the parent reference? Causes a lot of issues
    // parent: Option<Box<&'a AlbumDir>>,
    pub children: Vec<AlbumDir>,

    pub description: String,
}

impl AlbumDir {
    /// Returns an iterator over all images in the album and subalbums
    pub fn iter(&self) -> AlbumIter {
        AlbumIter::new(self)
    }

    /// Create an AlbumDir recursively from a path. The root path is so that we can make every path
    /// relative to the root.
    fn from_path(p: &Path, root: &Path) -> anyhow::Result<Self> {
        let mut images = vec![];
        let mut children = vec![];
        let mut description = String::new();

        for entry in p.read_dir()? {
            let entry_path = entry?.path().to_path_buf();

            if entry_path.is_file() {
                if let Some(filename) = entry_path.file_name() {
                    if filename == "description.txt" {
                        description = fs::read_to_string(entry_path)?;
                    } else if filename == "description.md" {
                        let _conents = fs::read_to_string(entry_path)?;
                        // TODO: render markdown
                        todo!();
                    } else {
                        let reader = ImageReader::open(&entry_path)?.with_guessed_format()?;
                        if reader.format().is_some() {
                            // Found an image
                            let mut description = String::new();

                            // Read in any associated description file
                            if entry_path.with_extension(".txt").exists() {
                                description = fs::read_to_string(&entry_path)?;
                            } else if entry_path.with_extension(".md").exists() {
                                let _contents = fs::read(entry_path)?;
                                // TODO: render markdown
                                todo!();
                            }

                            images.push(Image {
                                path: entry_path.strip_prefix(&root)?.to_path_buf(),
                                description,
                            });
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
                    }

                    children.push(AlbumDir::from_path(&entry_path, &root)?);
                }
            }
        }

        // Find all directories in directory and make AlbumDirs out of them,
        // but skip dirs known to have interesting stuff
        Ok(AlbumDir {
            path: p.strip_prefix(root)?.to_path_buf(),
            images,
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
pub struct AlbumIter<'a> {
    image_iter: Box<dyn Iterator<Item = &'a Image> + 'a>,
    children_iter: Iter<'a, AlbumDir>,
}

impl<'a> AlbumIter<'a> {
    fn new(ad: &'a AlbumDir) -> Self {
        Self {
            image_iter: Box::new(ad.images.iter()),
            children_iter: ad.children.iter(),
        }
    }
}

impl<'a> Iterator for AlbumIter<'a> {
    type Item = &'a Image;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(img) = self.image_iter.next() {
            return Some(img);
        }

        for album in self.children_iter.by_ref() {
            // Set the child album as the current image iterator
            self.image_iter = Box::new(album.iter());
            // If we found a child album with an image, return the image. Otherwise we'll keep
            // iterating over children.
            if let Some(i) = self.image_iter.next() {
                return Some(i);
            }
        }

        None
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Serialize)]
pub struct Image {
    /// Path to the image, relative to the root album
    pub path: PathBuf,

    /// Text description of the image which is displayed below it on the HTML page
    pub description: String,
}

impl Image {
    pub fn thumb_path(&self) -> anyhow::Result<PathBuf> {
        self.slide_path("thumb")
    }

    pub fn screen_path(&self) -> anyhow::Result<PathBuf> {
        self.slide_path("screen")
    }

    /// Returns the path to the file in the slides dir with the given extention insert, e.g.
    /// "thumb" or "display"
    fn slide_path(&self, ext: &str) -> anyhow::Result<PathBuf> {
        let new_ext = match self.path.extension() {
            Some(e) => {
                ext.to_string()
                    + "."
                    + e.to_str().ok_or(anyhow!(
                        "Image {} extension is not valid UTF-8",
                        self.path.display()
                    ))?
            }
            None => ext.to_string(),
        };

        let new_path = self.path.with_extension(new_ext);
        let new_name = new_path
            .file_name()
            .ok_or(anyhow!("Image {} missing a file name", self.path.display()))?;
        let parent = self
            .path
            .parent()
            .ok_or(anyhow!("Image {} has no parent dir", self.path.display()))?;

        Ok(parent.join("slides").join(new_name))
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
            images: vec![
                Image {
                    path: "foo".into(),
                    description: "".to_string(),
                },
                Image {
                    path: "bar".into(),
                    description: "".to_string(),
                },
            ],
            children: vec![],
        };
        // A child album with some images
        ad.children.push(AlbumDir {
            path: "subdir".into(),
            description: "".to_string(),
            images: vec![
                Image {
                    path: "subdir/foo".into(),
                    description: "".to_string(),
                },
                Image {
                    path: "subdir/bar".into(),
                    description: "".to_string(),
                },
            ],
            children: vec![AlbumDir {
                path: "subdir/deeper_subdir".into(),
                description: "".to_string(),
                images: vec![Image {
                    path: "subdir/deeper_subdir/image.jpg".into(),
                    description: "".to_string(),
                }],
                children: vec![],
            }],
        });
        // A child album with no images
        ad.children.push(AlbumDir {
            description: "".to_string(),
            path: "another_subdir".into(),
            images: vec![],
            children: vec![],
        });

        let imgs: HashSet<&str> = ad.iter().map(|i| i.path.to_str().unwrap()).collect();
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
        let img = Image {
            path: PathBuf::from("foo/bar/image.jpg"),
            description: String::new(),
        };
        assert_eq!(
            img.thumb_path().unwrap(),
            PathBuf::from("foo/bar/slides/image.thumb.jpg")
        );
        assert_eq!(
            img.screen_path().unwrap(),
            PathBuf::from("foo/bar/slides/image.screen.jpg")
        );
    }
}
