mod album_dir;

use album_dir::AlbumDir;
use std::io;
use std::path::{Path, PathBuf};

pub fn generate(root_path: &PathBuf) -> Result<(), io::Error> {
    let _ = AlbumDir::try_from(root_path)?;
    Ok(())
}
