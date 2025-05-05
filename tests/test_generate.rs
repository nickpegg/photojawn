// TODO: Is this really an intergration test? Or should it go into generate.rs?
// orrrr make a function in our CLI which does everything and test _that_
use mktemp::Temp;
use photojawn::generate::generate;
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
/// Test that the generate function creates a rendered site as we expect it
fn test_generate() {
    init();
    let album_path = make_test_album();
    let output_path = generate(&album_path.to_path_buf()).unwrap();

    check_album(output_path).unwrap();
}

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

/// Copies the test album to a tempdir and returns the path to it
fn make_test_album() -> Temp {
    let tmpdir = Temp::new_dir().unwrap();
    let source_path = Path::new("resources/test_album");

    let mut dirs: VecDeque<PathBuf> = VecDeque::from([source_path.to_path_buf()]);
    while let Some(dir) = dirs.pop_front() {
        for entry in dir.read_dir().unwrap() {
            let entry_path = entry.unwrap().path();
            let path_in_album = entry_path.strip_prefix(&source_path).unwrap();
            if entry_path.is_dir() {
                dirs.push_back(entry_path);
            } else {
                let dest_path = tmpdir.join(&path_in_album);
                fs::create_dir_all(dest_path.parent().unwrap()).unwrap();
                fs::copy(&entry_path, &dest_path).unwrap();
                log::debug!("Copied {} -> {}", entry_path.display(), dest_path.display());
            }
        }
    }

    tmpdir
}

/// Does basic sanity checks on an output album
fn check_album(album_dir: PathBuf) -> anyhow::Result<()> {
    log::debug!("Checking dir {}", album_dir.display());

    let mut dirs: VecDeque<PathBuf> = VecDeque::from([album_dir]);
    while let Some(dir) = dirs.pop_front() {
        for entry in dir.read_dir().unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() && !path.ends_with(Path::new("slides")) {
                check_album(path.clone())?;
            }
            let files: Vec<PathBuf> = path
                .read_dir()
                .unwrap()
                .into_iter()
                .map(|e| e.unwrap().path())
                .filter(|e| e.is_file())
                .collect();

            // There should be an index.html
            assert!(path.join("index.html").exists());

            // There should be a cover image
            let cover_path = path.join("cover.jpg");
            assert!(&cover_path.exists());

            // The cover should be equal contents to some other image
            let cover_contents = fs::read(&cover_path).unwrap();
            let mut found = false;
            for file in &files {
                if file != &cover_path {
                    let file_contents = fs::read(file).unwrap();
                    if file_contents == cover_contents {
                        found = true;
                    }
                }
            }
            if !found {
                panic!(
                    "cover.jpg in {} does not have a matching file",
                    path.display()
                );
            }

            // There should be a slides dir
            let slides_path = path.join("slides");
            assert!(slides_path.is_dir());

            // For each image in the album (including the cover), in slides there should be a:
            // - <image>.html
            // - <image>.screen.<ext>
            // - <image>.thumb.<ext>
            for file in &files {
                assert!(slides_path.join(file.with_extension("html")).exists());
                for ext in ["screen.jpg", "thumb.jpg"] {
                    assert!(slides_path.join(file.with_extension(ext)).exists());

                    // Make sure the screen/thumb is smaller than the original
                    todo!();
                }
            }

            // There shouldn't be any .txt or .md files hanging around
            for file in &files {
                if let Some(ext) = file.extension() {
                    assert_ne!(ext, "md");
                    assert_ne!(ext, "txt");
                }
            }
        }
    }
    Ok(())
}
