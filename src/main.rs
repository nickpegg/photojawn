use clap::{Parser, Subcommand};
use photojawn::generate::generate;
use photojawn::reorganize::reorganize;
use photojawn::skel::make_skeleton;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::parse();
    let album_path = Path::new(&cli.album_path).canonicalize()?;

    match cli.subcommand {
        Commands::Init {} => {
            make_skeleton(&album_path)?;
            println!("Album created in {}", album_path.display());
        }
        Commands::Generate { full } => {
            let path = generate(&album_path.to_path_buf(), full)?;
            println!("Album site generated in {}", path.display());
        }
        Commands::Reorganize { path, dry_run } => {
            reorganize(Path::new(&path), dry_run)?;
        }
    }

    Ok(())
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the album
    #[arg(long, default_value = ".")]
    album_path: String,

    #[command(subcommand)]
    subcommand: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Photojawn album directory
    Init {},
    /// Generates a photo album
    Generate {
        /// Regenerate everything, including images that have already been generated
        #[arg(long)]
        full: bool,
    },
    /// Reorganize photos in an album by date
    Reorganize {
        /// Directory of images you want to reorganize. Only image files will be moved.
        ///
        /// The new image filenames will be the date and time taken, followed by the original
        /// filename. For example:
        /// original_filename.jpg -> YYYYMMDD_HHSS_original_filename.jpg
        #[arg()]
        path: String,
        /// Don't actually reorganize, just say what renames would happen
        #[arg(long)]
        dry_run: bool,
    },
}
