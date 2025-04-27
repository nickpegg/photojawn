use std::fs;
use std::io;
use std::path::Path;
use thiserror::Error;

const BASE_TMPL: &'static [u8; 653] = include_bytes!("../resources/skel/_templates/base.html");
const ALBUM_TMPL: &'static [u8; 1682] = include_bytes!("../resources/skel/_templates/album.html");
const PHOTO_TMPL: &'static [u8; 1333] = include_bytes!("../resources/skel/_templates/photo.html");
const INDEX_CSS: &'static [u8; 1421] = include_bytes!("../resources/skel/static/index.css");
const BASE_CONFIG: &'static [u8; 315] = include_bytes!("../resources/skel/photojawn.conf.yml");

#[derive(Error, Debug)]
pub enum InitError {
    #[error("Album directory already initialized - contains a photojawn.conf.yml")]
    AlreadyInitialized,
    #[error(transparent)]
    IoError(#[from] io::Error),
}

pub fn make_skeleton(album_path: &Path) -> Result<(), InitError> {
    let cfg_path = album_path.join("photojawn.conf.yml");
    if cfg_path.exists() {
        return Err(InitError::AlreadyInitialized);
    }

    fs::create_dir_all(album_path)?;
    fs::write(cfg_path, BASE_CONFIG)?;

    let static_path = album_path.join("static");
    fs::create_dir_all(&static_path)?;
    fs::write(static_path.join("index.css"), INDEX_CSS)?;

    let tmpl_path = album_path.join("_templates");
    fs::create_dir_all(tmpl_path)?;
    fs::write(static_path.join("base.html"), BASE_TMPL)?;
    fs::write(static_path.join("album.html"), ALBUM_TMPL)?;
    fs::write(static_path.join("photo.html"), PHOTO_TMPL)?;

    Ok(())
}
