use std::path::PathBuf;

/// An album directory, which has images and possibly child albums
#[derive(Clone)]
struct AlbumDir {
    path: PathBuf,
    images: Vec<Image>,
    // TOOD: Remove the parent reference? Causes a lot of issues
    // parent: Option<Box<&'a AlbumDir>>,
    children: Vec<AlbumDir>,
}

impl AlbumDir {
    fn iter(&self) -> AlbumIter {
        AlbumIter::new(self)
    }
}

// TODO: from-path

/// An iterator which walks through all of the images in an album, and its sub-albums
struct AlbumIter<'a> {
    root_album: &'a AlbumDir,
    image_iter: Box<dyn Iterator<Item = &'a Image> + 'a>,
    children_iter: std::slice::Iter<'a, AlbumDir>,
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
    path: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn basic_album_iter() {
        let mut ad = AlbumDir {
            path: "".into(),
            images: vec![Image { path: "foo".into() }, Image { path: "bar".into() }],
            children: vec![],
        };
        // A child album with some images
        ad.children.push(AlbumDir {
            path: "subdir".into(),
            images: vec![
                Image {
                    path: "subdir/foo".into(),
                },
                Image {
                    path: "subdir/bar".into(),
                },
            ],
            children: vec![AlbumDir {
                path: "deeper_subdir".into(),
                images: vec![Image {
                    path: "deeper_subdir/image.jpg".into(),
                }],
                children: vec![],
            }],
        });
        // A child album with no images
        ad.children.push(AlbumDir {
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
