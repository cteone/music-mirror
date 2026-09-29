use std::path::PathBuf;
mod structs;

use clap::Parser;

mod diff;
use diff::diff;
mod apply;
use apply::apply;
mod compare_file;
mod watcher;

use crate::structs::{Config, FileConfig};

#[derive(Parser, Debug)]
#[command(name = "music-mirror")]
#[command(about = "Mirror a music library to a target library")]
struct Args {
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,

    #[arg(short, long)]
    target: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let config_text = std::fs::read_to_string(&args.config)?;

    let file_config: FileConfig = toml::from_str(&config_text)?;

    let config: Config = file_config.try_into()?;

    let target = config
        .targets
        .iter()
        .find(|target| target.library.name == args.target)
        .ok_or_else(|| format!("target '{}' not found in config", args.target))?;

    println!(
        "Source: {} ({})",
        config.source.library.name,
        config.source.library.path.display()
    );

    println!(
        "Target: {} ({})",
        target.library.name,
        target.library.path.display()
    );

    let diffs = diff(&config.source, target);

    println!("{diffs:#?}");

    apply(&diffs);

    Ok(())
}
