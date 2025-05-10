pub mod album_dir;
mod image;

use crate::config::Config;
use crate::generate::image::Image;
use album_dir::AlbumDir;
use anyhow::{Context, anyhow};
use indicatif::ProgressBar;
use rayon::prelude::*;
use serde::Serialize;
use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tera::Tera;

const IMG_RESIZE_FILTER: ::image::imageops::FilterType = ::image::imageops::FilterType::Lanczos3;
/// Generate an album
///
/// `root_path` is a path to the root directory of the album. `full` if true will regenerate
/// everything, including images that already exist.
pub fn generate(root_path: &PathBuf, full: bool) -> anyhow::Result<PathBuf> {
    log::debug!("Generating album in {}", root_path.display());
    let config = Config::from_album(root_path.to_path_buf())?;
    let orig_path = env::current_dir()?;

    // Jump into the root path so that all paths are relative to the root of the album
    env::set_current_dir(root_path)?;
    let album = AlbumDir::try_from(root_path)?;

    fs::create_dir_all(&config.output_dir)?;
    copy_static(&config)?;
    generate_images(&config, &album, full)?;
    generate_html(&config, &album)?;

    env::set_current_dir(orig_path)?;
    Ok(root_path.join(config.output_dir))
}

fn copy_static(config: &Config) -> anyhow::Result<()> {
    let dst = &config.output_dir.join("static");
    log::info!("Copying static files from _static to {}", dst.display());
    fs_extra::dir::copy(
        "_static",
        dst,
        &fs_extra::dir::CopyOptions::new()
            .content_only(true)
            .overwrite(true),
    )?;
    Ok(())
}

fn generate_images(config: &Config, album: &AlbumDir, full: bool) -> anyhow::Result<()> {
    let output_path = album.path.join(&config.output_dir);
    // HashSet is to avoid dupliates, which could happen since we add covers in later
    let mut all_images: HashSet<&Image> = album.iter_all_images().collect();

    // also resize cover images, since we didn't count those as part of the image collections
    let mut album_queue: VecDeque<&AlbumDir> = VecDeque::from([album]);
    while let Some(album) = album_queue.pop_front() {
        all_images.insert(&album.cover);
        album_queue.extend(&album.children);
    }

    let path_locks: HashMap<&PathBuf, Mutex<&PathBuf>> = all_images
        .iter()
        .map(|i| (&i.path, Mutex::new(&i.path)))
        .collect();

    println!("Generating images...");
    let progress = ProgressBar::new(all_images.len() as u64);
    let result = all_images.par_iter().try_for_each(|img| {
        // Get the lock on the path to make sure two threads don't try to generate the same image
        // on disk.
        let _path_lock = path_locks.get(&img.path).unwrap().lock().unwrap();

        let full_size_path = output_path.join(&img.path);
        if !full
            && full_size_path.exists()
            && fs::read(&full_size_path)? == fs::read(&full_size_path)?
        {
            log::info!("Skipping {}, already generated", img.path.display());
            return Ok(());
        }

        log::info!(
            "Copying original {} -> {}",
            img.path.display(),
            full_size_path.display()
        );
        fs::create_dir_all(full_size_path.parent().unwrap_or(Path::new("")))?;
        if full_size_path.exists() {
            fs::remove_file(&full_size_path)?;
        }
        fs::hard_link(&img.path, &full_size_path)
            .with_context(|| format!("Error creating hard link at {}", full_size_path.display()))?;

        let orig_image = ::image::open(&img.path)?;
        let thumb_path = output_path.join(&img.thumb_path);
        log::info!(
            "Resizing {} -> {}",
            img.path.display(),
            thumb_path.display()
        );
        fs::create_dir_all(thumb_path.parent().unwrap_or(Path::new("")))?;
        orig_image
            .resize(
                config.thumbnail_size.0,
                config.thumbnail_size.1,
                IMG_RESIZE_FILTER,
            )
            .save(&thumb_path)
            .with_context(|| format!("Error saving {}", thumb_path.display()))?;

        let screen_path = output_path.join(&img.screen_path);
        log::info!(
            "Resizing {} -> {}",
            img.path.display(),
            screen_path.display()
        );
        fs::create_dir_all(thumb_path.parent().unwrap_or(Path::new("")))?;
        orig_image
            .resize(config.view_size.0, config.view_size.1, IMG_RESIZE_FILTER)
            .save(&screen_path)
            .with_context(|| format!("Error saving {}", screen_path.display()))?;

        progress.inc(1);
        Ok(())
    });
    progress.finish();

    result
}

fn generate_html(config: &Config, album: &AlbumDir) -> anyhow::Result<()> {
    let output_path = album.path.join(&config.output_dir);
    let tera = Tera::new(
        album
            .path
            .join("_templates/*.html")
            .to_str()
            .ok_or(anyhow!("Album path {} is invalid", album.path.display()))?,
    )?;

    println!("Generating HTML...");

    let mut dir_queue: VecDeque<&AlbumDir> = VecDeque::from([album]);
    while let Some(album) = dir_queue.pop_front() {
        let html_path = output_path.join(&album.path).join("index.html");
        log::info!("Rendering album page {}", html_path.display());
        let ctx = AlbumContext::try_from(album)?;
        log::debug!("Album context: {ctx:?}");
        fs::write(
            output_path.join(&album.path).join("index.html"),
            tera.render("album.html", &tera::Context::from_serialize(&ctx)?)?,
        )?;

        for child in album.children.iter() {
            dir_queue.push_back(child);
        }

        for (pos, img) in album.images.iter().enumerate() {
            let prev_image: Option<&Image> = match pos {
                0 => None,
                n => Some(&album.images[n - 1]),
            };
            let next_image: Option<&Image> = album.images.get(pos + 1);

            // Find the path to the root by counting the parts of the path
            // Start with 1 .. to get out of the slides dir
            let mut path_to_root = PathBuf::from("..");
            if let Some(parent) = img.path.parent() {
                let mut parent = parent.to_path_buf();
                while parent.pop() {
                    path_to_root.push("..");
                }
            }

            log::info!("Rendering image page {}", img.html_path.display());
            let ctx = SlideContext {
                root_path: path_to_root,
                image: img.clone(),
                prev_image: prev_image.cloned(),
                next_image: next_image.cloned(),
            };
            log::debug!("Image context: {ctx:?}");
            fs::write(
                output_path.join(&img.html_path),
                tera.render("photo.html", &tera::Context::from_serialize(&ctx)?)?,
            )?;
        }
    }

    Ok(())
}

#[derive(Serialize, Debug)]
struct Breadcrumb {
    path: PathBuf,
    name: String,
}

/// A Tera context for album pages
#[derive(Serialize, Debug)]
struct AlbumContext {
    /// Path required to get back to the root album
    root_path: PathBuf,

    name: String,
    description: String,

    images: Vec<Image>,

    /// Path to the cover image thumbnail within /slides/, relative to the album dir. Used when
    /// linking to an album from a parent album
    cover_thumbnail_path: PathBuf,

    /// list of:
    /// - relative dir to walk up to root, e.g. "../../.."
    /// - dir name
    breadcrumbs: Vec<Breadcrumb>,

    /// Immediate children of this album
    children: Vec<AlbumContext>,
}

impl TryFrom<&AlbumDir> for AlbumContext {
    type Error = anyhow::Error;

    fn try_from(album: &AlbumDir) -> anyhow::Result<Self> {
        let name: String = match album.path.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => String::new(),
        };

        // Build breadcrumbs
        let mut breadcrumbs: Vec<Breadcrumb> = Vec::new();
        {
            let mut album_path = album.path.clone();
            let mut relpath: PathBuf = PathBuf::new();
            while album_path.pop() {
                let filename: &OsStr = album_path.file_name().unwrap_or(OsStr::new("Home"));
                relpath.push("..");
                breadcrumbs.push(Breadcrumb {
                    path: relpath.clone(),
                    name: filename.to_string_lossy().to_string(),
                });
            }
        }
        breadcrumbs.reverse();
        log::debug!("Crumbs for {}: {breadcrumbs:?}", album.path.display());

        // The first breadcrumb path is the relative path to the root album
        let root_path = match breadcrumbs.is_empty() {
            false => breadcrumbs[0].path.clone(),
            true => PathBuf::new(),
        };

        // Make the path to the thumbnail relative to the album that we're in
        let cover_thumbnail_path: PathBuf;
        if let Some(parent) = album.path.parent() {
            cover_thumbnail_path = album.cover.thumb_path.strip_prefix(parent)?.to_path_buf()
        } else {
            cover_thumbnail_path = album
                .cover
                .thumb_path
                .strip_prefix(&album.path)?
                .to_path_buf()
        };

        let children: Vec<AlbumContext> = album
            .children
            .iter()
            .map(AlbumContext::try_from)
            .collect::<anyhow::Result<Vec<AlbumContext>>>()?;
        let images: Vec<Image> = album.images.clone();

        Ok(AlbumContext {
            root_path,
            name,
            description: album.description.clone(),
            breadcrumbs,
            images,
            children,
            cover_thumbnail_path,
        })
    }
}

/// A Tera context for slide (individual image) pages
#[derive(Serialize, Debug)]
struct SlideContext {
    // Path required to get back to the root album
    root_path: PathBuf,
    image: Image,
    prev_image: Option<Image>,
    next_image: Option<Image>,
}

#[cfg(test)]
mod tests {
    use super::generate;
    use crate::skel::make_skeleton;
    use mktemp::Temp;
    use std::collections::{HashSet, VecDeque};
    use std::ffi::OsStr;
    use std::path::{Path, PathBuf};

    #[test]
    /// Test that the generate function creates a rendered site as we expect it
    fn test_generate() {
        init();
        let album_path = make_test_album();
        let output_path = generate(&album_path.to_path_buf(), false).unwrap();

        check_album(output_path).unwrap();
    }

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    /// Copies the test album to a tempdir and returns the path to it
    fn make_test_album() -> Temp {
        let tmpdir = Temp::new_dir().unwrap();
        let source_path = Path::new("resources/test_album");

        log::info!("Creating test album in {}", tmpdir.display());
        make_skeleton(&tmpdir.to_path_buf()).unwrap();
        fs_extra::dir::copy(
            &source_path,
            &tmpdir,
            &fs_extra::dir::CopyOptions::new().content_only(true),
        )
        .unwrap();

        tmpdir
    }

    /// Does basic sanity checks on an output album
    fn check_album(root_path: PathBuf) -> anyhow::Result<()> {
        log::debug!("Checking album dir {}", root_path.display());

        // The _static dir should have gotten copied into <output>/static
        assert!(root_path.join("static/index.css").exists());

        let mut dirs: VecDeque<PathBuf> = VecDeque::from([root_path.clone()]);
        while let Some(dir) = dirs.pop_front() {
            let mut files: Vec<PathBuf> = Vec::new();
            for entry in dir.read_dir().unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir()
                    && !path.ends_with(Path::new("slides"))
                    && path.file_name() != Some(OsStr::new("static"))
                {
                    dirs.push_back(path.clone());
                } else if path.is_file() {
                    files.push(path);
                }
            }

            // There should be an index.html
            let index_path = dir.join("index.html");
            assert!(
                index_path.exists(),
                "Expected {} to exist",
                index_path.display()
            );

            // There should be a slides dir
            let slides_path = dir.join("slides");
            assert!(
                slides_path.is_dir(),
                "Expected {} to be a dir",
                slides_path.display()
            );

            // No two images should have the same path
            let image_set: HashSet<&PathBuf> = files.iter().collect();
            assert_eq!(image_set.len(), files.len());

            // For each image in the album (including the cover), in slides there should be a:
            // - <image>.html
            // - <image>.screen.<ext>
            // - <image>.thumb.<ext>
            for file in &files {
                if let Some(ext) = file.extension() {
                    if ext != "jpg" {
                        continue;
                    }
                }
                log::debug!("Checking associated files for {}", file.display());

                if !file
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with("cover")
                {
                    let html_path =
                        slides_path.join(&file.with_extension("html").file_name().unwrap());
                    assert!(
                        html_path.exists(),
                        "Expected {} to exist",
                        html_path.display()
                    );
                }
                for ext in ["screen.jpg", "thumb.jpg"] {
                    let img_path = slides_path.join(file.with_extension(ext).file_name().unwrap());
                    assert!(
                        img_path.exists(),
                        "Expected {} to exist",
                        img_path.display()
                    );
                }
            }

            // There shouldn't be any .txt or .md files hanging around
            for file in &files {
                if let Some(ext) = file.extension() {
                    assert_ne!(ext, "md");
                    assert_ne!(ext, "txt");
                }
            }
        }
        Ok(())
    }
}
