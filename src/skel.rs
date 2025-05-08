use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InitError {
    #[error("Album directory already initialized - contains a {0}")]
    AlreadyInitialized(PathBuf),
    #[error(transparent)]
    IoError(#[from] io::Error),
}

/// Creates a new album directory and creates basic versions
pub fn make_skeleton(album_path: &Path) -> Result<(), InitError> {
    let files = HashMap::from([
        (
            album_path.join("photojawn.conf.yml"),
            include_bytes!("../resources/skel/photojawn.conf.yml").as_slice(),
        ),
        (
            album_path.join("_static/index.css"),
            include_bytes!("../resources/skel/_static/index.css").as_slice(),
        ),
        (
            album_path.join("_templates/base.html"),
            include_bytes!("../resources/skel/_templates/base.html").as_slice(),
        ),
        (
            album_path.join("_templates/album.html"),
            include_bytes!("../resources/skel/_templates/album.html").as_slice(),
        ),
        (
            album_path.join("_templates/photo.html"),
            include_bytes!("../resources/skel/_templates/photo.html").as_slice(),
        ),
    ]);

    // Bail if any of the files we would create exist
    for path in files.keys() {
        if path.exists() {
            return Err(InitError::AlreadyInitialized(path.to_path_buf()));
        }
    }

    fs::create_dir_all(album_path)?;
    for (path, contents) in files.iter() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, contents)?;
    }

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
        assert!(tmpdir.join("_static/index.css").exists());
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
        let res = make_skeleton(&tmpdir);
        assert!(res.is_err());

        // Make sure it didn't clobber our template
        let contents = fs::read(tmpdir.join("_templates/base.html")).unwrap();
        assert_eq!(contents, "some template".as_bytes());
    }
}
