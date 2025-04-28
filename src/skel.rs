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
    fs::create_dir_all(&tmpl_path)?;
    fs::write(
        tmpl_path.join("base.html"),
        include_bytes!("../resources/skel/_templates/base.html"),
    )?;
    fs::write(
        tmpl_path.join("album.html"),
        include_bytes!("../resources/skel/_templates/album.html"),
    )?;
    fs::write(
        tmpl_path.join("photo.html"),
        include_bytes!("../resources/skel/_templates/photo.html"),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mktemp::Temp;

    #[test]
    fn not_exist() {
        let tmpdir = Temp::new_dir().unwrap();
        make_skeleton(&tmpdir).unwrap();
        assert!(tmpdir.join("photojawn.conf.yml").exists());
        assert!(tmpdir.join("static/index.css").exists());
        assert!(tmpdir.join("_templates/base.html").exists());
    }

    #[test]
    fn config_exists() {
        let tmpdir = Temp::new_dir().unwrap();
        fs::write(tmpdir.join("photojawn.conf.yml"), "some: config").unwrap();
        let res = make_skeleton(&tmpdir);
        assert!(res.is_err());
    }

    #[test]
    fn dir_exists_no_config() {
        let tmpdir = Temp::new_dir().unwrap();
        fs::create_dir(tmpdir.join("_templates")).unwrap();
        fs::write(tmpdir.join("_templates/base.html"), "some template").unwrap();
        make_skeleton(&tmpdir).unwrap();

        let contents = fs::read(tmpdir.join("_templates/base.html")).unwrap();
        assert_ne!(contents, "some template".as_bytes());
    }
}
