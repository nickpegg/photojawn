use crate::skel::make_skeleton;
use mktemp::Temp;
use std::path::Path;

pub fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

/// Copies the test album to a tempdir and returns the path to it. Returns a Temp object which
/// cleans up the directory on drop, so make sure to persist the variable until you're done with it
pub fn make_test_album() -> Temp {
    let tmpdir = Temp::new_dir().unwrap();
    let source_path = Path::new("resources/test_album").canonicalize().unwrap();

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
