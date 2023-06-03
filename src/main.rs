use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;
use id3::{Tag, TagLike};
use owo_colors::OwoColorize;
use thiserror::Error;

#[derive(Parser)]
#[command(name = "fmmd - fix music metadata")]
#[command(author = "Travis Hathaway")]
#[command(version = "0.1.0")]
#[command(about = "Used to rename music files based on their metadata")]
struct Cli {
    /// List of files to rename
    files: Vec<PathBuf>,

    /// Perform a dry run without renaming files
    #[arg(short, long)]
    dry_run: bool,

    /// Print verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Error, Debug)]
enum FmmdError {
    #[error("Could not parse the file")]
    FileParse(#[from] id3::Error),

    #[error("Could not rename the file")]
    FileRename(#[from] std::io::Error),

    #[error("Could not find enough information in the file to rename it")]
    NotEnoughMetadata,
}

/// Attempts to read metadata from file and renames it if it has enough data
fn rename_file(file: &Path, cli: &Cli) -> Result<(), FmmdError> {
    let tag = match Tag::read_from_path(file) {
        Ok(tag) => tag,
        Err(error) => return Err(FmmdError::FileParse(error)),
    };

    let new_file = get_filename(tag, file)?;

    if cli.dry_run || cli.verbose {
        println!(
            "{} -> {}",
            file.to_str().unwrap(),
            new_file.to_str().unwrap()
        );
    }

    if cli.dry_run {
        return Ok(());
    }

    if let Err(error) = fs::rename(file, new_file) {
        return Err(FmmdError::FileRename(error));
    }

    Ok(())
}

/// Attempts to crate a new file name based on the `Tag` and `PathBuf` provided.
///
/// We need at least the track name and the track number to create a new file name.
fn get_filename(tag: Tag, file: &Path) -> Result<PathBuf, FmmdError> {
    let title = tag.title().unwrap_or_default();
    let track = tag.track().unwrap_or(0);

    if title.is_empty() && track == 0 {
        return Err(FmmdError::NotEnoughMetadata);
    }

    let parent = file.parent().unwrap().to_str().unwrap();
    let extension = file.extension().unwrap().to_str().unwrap();
    let track = format!("{:0>2}", track);
    let new_path = Path::new(parent).join(format!("{}-{}.{}", track, title, extension));

    Ok(new_path)
}

fn main() {
    let cli = Cli::parse();

    for file in &cli.files {
        if file.exists() {
            if let Err(error) = rename_file(file, &cli) {
                eprintln!("{}: \"{}\"", error.red(), file.to_str().unwrap().red());
            }
        }
    }
}
