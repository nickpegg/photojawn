use std::fs;
use std::io;
use std::path::Path;
use thiserror::Error;

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
    fs::write(
        cfg_path,
        include_bytes!("../resources/skel/photojawn.conf.yml"),
    )?;

    let static_path = album_path.join("static");
    fs::create_dir_all(&static_path)?;
    fs::write(
        static_path.join("index.css"),
        include_bytes!("../resources/skel/static/index.css"),
    )?;

    let tmpl_path = album_path.join("_templates");
    fs::create_dir_all(tmpl_path)?;
    fs::write(
        static_path.join("base.html"),
        include_bytes!("../resources/skel/_templates/base.html"),
    )?;
    fs::write(
        static_path.join("album.html"),
        include_bytes!("../resources/skel/_templates/album.html"),
    )?;
    fs::write(
        static_path.join("photo.html"),
        include_bytes!("../resources/skel/_templates/photo.html"),
    )?;

    Ok(())
}
