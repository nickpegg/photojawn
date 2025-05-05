mod album_dir;

use crate::config::Config;
pub use album_dir::AlbumDir;
use image::imageops::FilterType;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const IMG_RESIZE_FILTER: FilterType = FilterType::Lanczos3;

pub fn generate(root_path: &PathBuf) -> anyhow::Result<PathBuf> {
    log::debug!("Generating album in {}", root_path.display());
    let config = Config::from_album(root_path.to_path_buf())?;
    let orig_path = env::current_dir()?;

    // Jump into the root path so that all paths are relative to the root of the album
    env::set_current_dir(root_path)?;
    let album = AlbumDir::try_from(root_path)?;

    generate_images(&config, &album)?;
    generate_html(&config, &album)?;

    env::set_current_dir(orig_path)?;
    Ok(root_path.join(config.output_dir))
}

fn generate_images(config: &Config, album: &AlbumDir) -> anyhow::Result<()> {
    let output_path = album.path.join(&config.output_dir);
    // TODO: use par_iter() ?
    // TODO: progress bar ?
    for img in album.iter() {
        let orig_image = image::open(&img.path)?;

        let thumb_path = output_path.join(img.thumb_path()?);
        fs::create_dir_all(thumb_path.parent().unwrap_or(Path::new("")))?;
        orig_image
            .resize(
                config.thumbnail_size.0,
                config.thumbnail_size.1,
                IMG_RESIZE_FILTER,
            )
            .save(&thumb_path)?;
        log::info!("Resized {} -> {}", img.path.display(), thumb_path.display());

        // TODO: resize to screen size
        let screen_path = output_path.join(img.screen_path()?);
        fs::create_dir_all(thumb_path.parent().unwrap_or(Path::new("")))?;
        orig_image
            .resize(config.view_size.0, config.view_size.1, IMG_RESIZE_FILTER)
            .save(&screen_path)?;
        log::info!(
            "Resized {} -> {}",
            img.path.display(),
            screen_path.display()
        );
    }
    Ok(())
}

fn generate_html(config: &Config, album: &AlbumDir) -> anyhow::Result<()> {
    Ok(())
}
