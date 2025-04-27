use clap::{Parser, Subcommand};

fn main() {
    let cli = Cli::parse();
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the album
    #[arg(long, default_value = ".")]
    album_path: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Photojawn album directory
    Init {},
    /// Generates a photo album
    Generate {
        #[arg(long)]
        quick: bool,
    },
    /// Remove all generated content from the photo album directory
    Clean {},
}
