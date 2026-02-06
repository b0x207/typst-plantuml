use clap::Parser;
use log::info;
use notify::event::{DataChange, ModifyKind};
use notify::{EventKind, Watcher};
use rootcause::prelude::*;
use std::path::PathBuf;
use std::sync::mpsc;

mod parser;

#[cfg(not(debug_assertions))]
const DEFAULT_LOG_LEVEL: log::LevelFilter = log::LevelFilter::Info;

#[cfg(debug_assertions)]
const DEFAULT_LOG_LEVEL: log::LevelFilter = log::LevelFilter::Debug;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    target: PathBuf,
}

fn process_file(path: PathBuf) -> Result<(), Report> {
    // This is a very dumb heuristic that should be improved upon later. For the sake of speed,
    // however, we will just check the file extension to ensure that the file is actually typst
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
        info!("-> {:?}", block.0);
    }

    Ok(())
}

fn event_handler(event: notify::Event) -> Result<(), Report> {
    if event.kind != EventKind::Modify(ModifyKind::Data(DataChange::Any)) {
        return Ok(());
    }

    for path in event.paths {
        process_file(path)?;
    }

    Ok(())
}

fn main() -> Result<(), Report> {
    pretty_env_logger::formatted_builder()
        .filter_level(DEFAULT_LOG_LEVEL)
        .init();

    let cli = Cli::parse();

    let (tx, rx) = mpsc::channel::<Result<notify::Event, notify::Error>>();
    let mut watcher = notify::recommended_watcher(tx)?;

    watcher.watch(&cli.target, notify::RecursiveMode::Recursive)?;
    info!("Watching directory {:?}", cli.target);

    for res in rx {
        event_handler(res?)?;
    }

    Ok(())
}
