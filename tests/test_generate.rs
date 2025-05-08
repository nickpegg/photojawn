// TODO: Is this really an intergration test? Or should it go into generate.rs?
// orrrr make a function in our CLI which does everything and test _that_
use mktemp::Temp;
use photojawn::generate::generate;
use photojawn::skel::make_skeleton;
use std::collections::{HashSet, VecDeque};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[test]
/// Test that the generate function creates a rendered site as we expect it
fn test_generate() {
    init();
    let album_path = make_test_album();
    let output_path = generate(&album_path.to_path_buf(), false).unwrap();

    check_album(output_path).unwrap();
}

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

/// Copies the test album to a tempdir and returns the path to it
fn make_test_album() -> Temp {
    let tmpdir = Temp::new_dir().unwrap();
    let source_path = Path::new("resources/test_album");

    log::info!("Creating test album in {}", tmpdir.display());
    make_skeleton(&tmpdir.to_path_buf()).unwrap();
    fs_extra::dir::copy(
        &source_path,
        &tmpdir,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )
    .unwrap();

    tmpdir
}

/// Does basic sanity checks on an output album
fn check_album(root_path: PathBuf) -> anyhow::Result<()> {
    log::debug!("Checking album dir {}", root_path.display());

    // The _static dir should have gotten copied into <output>/static
    assert!(root_path.join("static/index.css").exists());

    let mut dirs: VecDeque<PathBuf> = VecDeque::from([root_path.clone()]);
    while let Some(dir) = dirs.pop_front() {
        let mut files: Vec<PathBuf> = Vec::new();
        for entry in dir.read_dir().unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir()
                && !path.ends_with(Path::new("slides"))
                && path.file_name() != Some(OsStr::new("static"))
            {
                dirs.push_back(path.clone());
            } else if path.is_file() {
                files.push(path);
            }
        }

        // There should be an index.html
        let index_path = dir.join("index.html");
        assert!(
            index_path.exists(),
            "Expected {} to exist",
            index_path.display()
        );

        // There should be a slides dir
        let slides_path = dir.join("slides");
        assert!(
            slides_path.is_dir(),
            "Expected {} to be a dir",
            slides_path.display()
        );

        // No two images should have the same path
        let image_set: HashSet<&PathBuf> = files.iter().collect();
        assert_eq!(image_set.len(), files.len());

        // For each image in the album (including the cover), in slides there should be a:
        // - <image>.html
        // - <image>.screen.<ext>
        // - <image>.thumb.<ext>
        for file in &files {
            if let Some(ext) = file.extension() {
                if ext != "jpg" {
                    continue;
                }
            }
            log::debug!("Checking associated files for {}", file.display());

            let html_path = slides_path.join(&file.with_extension("html").file_name().unwrap());
            assert!(
                html_path.exists(),
                "Expected {} to exist",
                html_path.display()
            );
            for ext in ["screen.jpg", "thumb.jpg"] {
                let img_path = slides_path.join(file.with_extension(ext).file_name().unwrap());
                assert!(
                    img_path.exists(),
                    "Expected {} to exist",
                    img_path.display()
                );

                // TODO: Make sure the screen/thumb is smaller than the original
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
    Ok(())
}
