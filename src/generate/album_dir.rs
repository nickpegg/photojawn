use image::ImageReader;
use std::ffi::OsString;
use std::io;
use std::path::PathBuf;
use std::slice::Iter;

/// An album directory, which has images and possibly child albums
#[derive(Clone)]
pub struct AlbumDir {
    path: PathBuf,
    images: Vec<Image>,
    // TOOD: Remove the parent reference? Causes a lot of issues
    // parent: Option<Box<&'a AlbumDir>>,
    children: Vec<AlbumDir>,

    description: String,
}

impl AlbumDir {
    /// Returns an iterator over all images in the album and subalbums
    fn iter(&self) -> AlbumIter {
        AlbumIter::new(self)
    }
}

impl TryFrom<&PathBuf> for AlbumDir {
    type Error = io::Error;

    fn try_from(p: &PathBuf) -> io::Result<AlbumDir> {
        let mut images = vec![];
        let mut children = vec![];
        let mut description = "".to_string();

        for entry in p.read_dir()? {
            let entry_path = entry?.path();

            if entry_path.is_file() {
                println!("Found file: {}", entry_path.display());
                if let Some(filename) = entry_path.file_name() {
                    if filename == "description.txt" {
                        todo!();
                        // description = String::from_utf8(fs::read(entry_path)?)?;
                    } else if filename == "description.md" {
                        todo!();
                    } else {
                        let reader = ImageReader::open(&entry_path)?.with_guessed_format()?;
                        if reader.format().is_some() {
                            // Found an image
                            // TODO: If image filename but with .md or .txt exists, read that in as
                            // the image description OR make this part of Image::from<Path>
                            todo!();
                            let description = String::new();
                            images.push(Image {
                                filename: filename.to_os_string(),
                                description,
                            });
                        }
                    }
                }
            } else if entry_path.is_dir() {
                println!("Found dir: {}", entry_path.display());

                if let Some(dirname) = entry_path.file_name().and_then(|n| n.to_str()) {
                    if dirname.starts_with("_") {
                        // Likely a templates or static dir
                        continue;
                    } else if dirname == "site" {
                        // Is a generated site dir, don't descend into it
                        continue;
                    }

                    children.push(AlbumDir::try_from(&entry_path)?);
                }
            }
        }

        // Find all directories in directory and make AlbumDirs out of them,
        // but skip dirs known to have interesting stuff
        Ok(AlbumDir {
            path: p.clone(),
            images,
            children,
            description,
        })
    }
}

// TODO: from-path, find all images and children

/// An iterator which walks through all of the images in an album, and its sub-albums
struct AlbumIter<'a> {
    root_album: &'a AlbumDir,
    image_iter: Box<dyn Iterator<Item = &'a Image> + 'a>,
    children_iter: Iter<'a, AlbumDir>,
}

impl<'a> AlbumIter<'a> {
    fn new(ad: &'a AlbumDir) -> Self {
        Self {
            root_album: ad,
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

#[derive(Clone, Hash, PartialEq, Eq)]
struct Image {
    filename: OsString,
    description: String,
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
                path: "deeper_subdir".into(),
                description: "".to_string(),
                images: vec![Image {
                    path: "deeper_subdir/image.jpg".into(),
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

        let imgs: HashSet<String> = ad
            .iter()
            .map(|i| i.path.clone().to_str().unwrap().to_string())
            .collect();
        let expected: HashSet<String> = [
            "foo",
            "bar",
            "subdir/foo",
            "subdir/bar",
            "deeper_subdir/image.jpg",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        assert_eq!(imgs, expected);
    }
}
