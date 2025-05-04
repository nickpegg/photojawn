mod album_dir;

use album_dir::AlbumDir;
use std::env;
use std::path::{Path, PathBuf};

const OUTPUT_PATH: &'static str = "site";

pub fn generate(root_path: &PathBuf) -> anyhow::Result<()> {
    let orig_path = env::current_dir()?;
    let album = AlbumDir::try_from(root_path)?;
    env::set_current_dir(&root_path)?;

    generate_images(&album)?;
    generate_html(&album)?;

    env::set_current_dir(orig_path)?;
    Ok(())
}

fn generate_images(album: &AlbumDir) -> anyhow::Result<()> {
    let output_path = album.path.join(OUTPUT_PATH);
    Ok(())
}

fn generate_html(album: &AlbumDir) -> anyhow::Result<()> {
    Ok(())
}
