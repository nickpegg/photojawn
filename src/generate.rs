pub mod album_dir;

use crate::config::Config;
use album_dir::{AlbumDir, Image};
use anyhow::anyhow;
use image::imageops::FilterType;
use rayon::prelude::*;
use serde::Serialize;
use std::collections::VecDeque;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use tera::Tera;

const IMG_RESIZE_FILTER: FilterType = FilterType::Lanczos3;

#[derive(Serialize, Debug)]
struct Breadcrumb {
    path: PathBuf,
    name: String,
}

/// A Tera context for album pages
#[derive(Serialize, Debug)]
struct AlbumContext {
    // Path required to get back to the root album
    root_path: PathBuf,

    name: String,
    description: String,

    images: Vec<Image>,

    // Path to the cover image thumbnail within /slides/, relative to the album dir. Used when
    // linking to an album from a parent album
    cover_thumbnail_path: PathBuf,

    // list of:
    // - relative dir to walk up to root, e.g. "../../.."
    // - dir name
    breadcrumbs: Vec<Breadcrumb>,

    // Immediate children of this album
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
        let root_path = if !breadcrumbs.is_empty() {
            breadcrumbs[0].path.clone()
        } else {
            PathBuf::new()
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
    // TODO: progress bar ?
    let mut all_images: Vec<&Image> = album.iter_all_images().collect();

    // also resize cover image
    all_images.push(&album.cover);

    all_images.par_iter().try_for_each(|img| {
        // TODO: Hard-link if it's supported, to save on hard drive space
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
        fs::copy(&img.path, &full_size_path)?;

        let orig_image = image::open(&img.path)?;
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
            .save(&thumb_path)?;

        let screen_path = output_path.join(&img.screen_path);
        log::info!(
            "Resizing {} -> {}",
            img.path.display(),
            screen_path.display()
        );
        fs::create_dir_all(thumb_path.parent().unwrap_or(Path::new("")))?;
        orig_image
            .resize(config.view_size.0, config.view_size.1, IMG_RESIZE_FILTER)
            .save(&screen_path)?;

        Ok(())
    })
}

fn generate_html(config: &Config, album: &AlbumDir) -> anyhow::Result<()> {
    let output_path = album.path.join(&config.output_dir);
    let tera = Tera::new(
        album
            .path
            .join("_templates/*.html")
            .to_str()
            .ok_or(anyhow!("Missing _templates dir in album dir"))?,
    )?;

    let mut dir_queue: VecDeque<&AlbumDir> = VecDeque::from([album]);
    while let Some(album) = dir_queue.pop_front() {
        let html_path = output_path.join(&album.path).join("index.html");
        log::info!("Rendering album {}", html_path.display());
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

            log::info!("Rendering image {}", img.html_path.display());
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
