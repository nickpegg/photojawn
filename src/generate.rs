pub mod album_dir;

use crate::config::Config;
use album_dir::{AlbumDir, Image};
use anyhow::anyhow;
use image::imageops::FilterType;
use serde::Serialize;
use std::collections::VecDeque;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use tera::Tera;

const IMG_RESIZE_FILTER: FilterType = FilterType::Lanczos3;

#[derive(Serialize, Debug)]
struct Breadcrumb {
    path: PathBuf,
    name: OsString,
}

/// A Tera context for album pages
#[derive(Serialize, Debug)]
struct AlbumContext {
    // Path required to get back to the root album
    root_path: PathBuf,

    // TODO: Do we actualy need the whole albumDir? Probably better off pulling data we need out of
    // it.
    // album_dir: AlbumDir,
    name: OsString,

    // TODO: images

    // Path to the cover image thumbnail within /slides/. Used when linking to an album from a
    // parent album
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
        let name: OsString = match album.path.file_name() {
            Some(n) => n.to_os_string(),
            None => OsString::new(),
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
                    name: filename.into(),
                });
            }
        }
        breadcrumbs.reverse();
        log::debug!("Crumbs for {}: {breadcrumbs:?}", album.path.display());

        // The first breadcrumb path is the relative path to the root album
        let root_path = if breadcrumbs.len() > 0 {
            breadcrumbs[0].path.clone()
        } else {
            PathBuf::new()
        };

        // Pick a cover image thumbnail and build a path to it
        let cover_thumbnail_path = match &album.cover {
            Some(i) => i.path.clone(),
            None => PathBuf::from(""),
        };

        let children: Vec<AlbumContext> = album
            .children
            .iter()
            .map(|a| AlbumContext::try_from(a))
            .collect::<anyhow::Result<Vec<AlbumContext>>>()?;

        Ok(AlbumContext {
            name,
            breadcrumbs,
            root_path,
            children,
            cover_thumbnail_path,
        })
    }
}

/// A Tera context for slide (individual image) pages
#[derive(Serialize)]
struct SlideContext {
    // TODO: Path or String?
    root_path: PathBuf,
    image: Image,
    prev_image: Image,
    next_image: Image,
}

pub fn generate(root_path: &PathBuf) -> anyhow::Result<PathBuf> {
    log::debug!("Generating album in {}", root_path.display());
    let config = Config::from_album(root_path.to_path_buf())?;
    let orig_path = env::current_dir()?;

    // Jump into the root path so that all paths are relative to the root of the album
    env::set_current_dir(root_path)?;
    let album = AlbumDir::try_from(root_path)?;

    copy_static(&config)?;
    generate_images(&config, &album)?;
    generate_html(&config, &album)?;

    env::set_current_dir(orig_path)?;
    Ok(root_path.join(config.output_dir))
}

fn copy_static(config: &Config) -> anyhow::Result<()> {
    Ok(())
}

fn generate_images(config: &Config, album: &AlbumDir) -> anyhow::Result<()> {
    let output_path = album.path.join(&config.output_dir);
    // TODO: use par_iter() ?
    // TODO: progress bar ?
    for img in album.iter() {
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

        // TODO: resize to screen size
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
    }

    Ok(())
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

    // Queue of album dir and depth (distance from root AlbumDir)
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
            dir_queue.push_back(&child);
        }
    }

    Ok(())
}
