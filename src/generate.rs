mod album_dir;

use crate::config::Config;
pub use album_dir::AlbumDir;
use std::env;
use std::path::PathBuf;

pub fn generate(root_path: &PathBuf) -> anyhow::Result<PathBuf> {
    let config = Config::from_album(root_path.to_path_buf())?;
    let orig_path = env::current_dir()?;
    let album = AlbumDir::try_from(root_path)?;
    env::set_current_dir(&root_path)?;

    generate_images(&config, &album)?;
    generate_html(&config, &album)?;

    env::set_current_dir(orig_path)?;
    Ok(root_path.join(config.output_dir))
}

fn generate_images(config: &Config, album: &AlbumDir) -> anyhow::Result<()> {
    Ok(())
}

fn generate_html(config: &Config, album: &AlbumDir) -> anyhow::Result<()> {
    Ok(())
}
