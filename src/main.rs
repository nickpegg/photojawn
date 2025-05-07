use clap::{Parser, Subcommand};
use photojawn::generate::generate;
use photojawn::skel::make_skeleton;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    let album_path = Path::new(&cli.album_path);

    match cli.subcommand {
        Commands::Init {} => {
            make_skeleton(album_path)?;
            println!("Album created in {}", album_path.display());
        }
        Commands::Generate { quick } => {
            let path = generate(&album_path.to_path_buf())?;
            println!("Album site generated in {}", path.display());
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
        /// Don't re-generate things that already exist (thumbnails, etc.)
        #[arg(long)]
        quick: bool,
    },
}
