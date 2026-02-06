use clap::Parser;
use log::info;
use notify::event::{DataChange, ModifyKind};
use notify::{EventKind, Watcher};
use rootcause::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

mod parser;
mod render;

#[cfg(not(debug_assertions))]
const DEFAULT_LOG_LEVEL: log::LevelFilter = log::LevelFilter::Info;

#[cfg(debug_assertions)]
const DEFAULT_LOG_LEVEL: log::LevelFilter = log::LevelFilter::Debug;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    target: PathBuf,

    #[arg(long)]
    asset_dir: Option<PathBuf>,
}

fn process_file(path: PathBuf, asset_dir: &Path) -> Result<(), Report> {
    // This is a very dumb heuristic that should be improved upon later. For the sake of speed,
    // however, we will just check the file extension to ensure that the file is actually Typst
    let extension = path
        .extension()
        .unwrap_or_default()
        .to_str()
        .ok_or(report!("Unsupported file path contents!"))?;
    if !extension.ends_with("typ") {
        return Ok(());
    }

    log::debug!("Processing {path:?}");

    let contents = std::fs::read_to_string(path)?;
    let tree = typst_syntax::parse(&contents);

    let raw_blocks = parser::search_ast_tree(&tree)?;
    info!("Found {} items to process", raw_blocks.len());
    for block in raw_blocks {
        let output_path = asset_dir.join(block.0);
        let target_format = match output_path.extension() {
            Some(ext) => ext
                .to_str()
                .ok_or(report!("Output path didn't have a valid extension"))?,
            None => ".svg",
        };

        render::render_plantuml(&output_path, target_format, &block.1)?;
    }

    Ok(())
}

fn event_handler(event: notify::Event, asset_dir: &Path) -> Result<(), Report> {
    if event.kind != EventKind::Modify(ModifyKind::Data(DataChange::Any)) {
        return Ok(());
    }

    for path in event.paths {
        process_file(path, asset_dir)?;
    }

    Ok(())
}

fn main() -> Result<(), Report> {
    pretty_env_logger::formatted_builder()
        .filter_level(DEFAULT_LOG_LEVEL)
        .init();

    let cli = Cli::parse();
    let asset_dir = cli.asset_dir.unwrap_or(PathBuf::from("."));

    if !asset_dir.exists() {
        std::fs::create_dir(&asset_dir)?;
    }

    let (tx, rx) = mpsc::channel::<Result<notify::Event, notify::Error>>();
    let mut watcher = notify::recommended_watcher(tx)?;

    watcher.watch(&cli.target, notify::RecursiveMode::Recursive)?;
    info!("Watching directory {:?}", cli.target);

    for res in rx {
        event_handler(res?, &asset_dir)?;
    }

    Ok(())
}
