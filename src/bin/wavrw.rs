//! wavrw Command Line Interface

#![deny(missing_docs)]

use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::PathBuf;

use anyhow::Result;
use clap::{crate_version, ArgAction, Parser, Subcommand};
use itertools::Itertools;
use tracing::{debug, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(author, about, long_about = None,
    disable_help_flag = true,
    disable_version_flag = true,
    next_help_heading="Global Options",  
    version=crate_version!())]
struct WavrwArgs {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, short, global = true, action=ArgAction::Help,
        help = "Print help")]
    help: (),

    #[arg(long, short='V', action=ArgAction::Version,
        help = "Print version")]
    version: (),
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Summarize WAV file structure and metadata
    View(ViewConfig),
    /// List chunks contained in files, one per line
    Chunks(ChunksConfig),
}

#[derive(Parser, Debug)]
#[command(long_about = None)]
struct ViewConfig {
    /// One or more paths to WAV files
    wav_path: Vec<OsString>,

    /// show detailed info for each chunk
    #[arg(long, short)]
    detailed: bool,

    #[arg(
        long,
        short = 'w',
        default_value_t = 80,
        help = "trim output to <WIDTH> columns"
    )]
    width: u16,
}

fn trim(text: String, width: u16) -> String {
    let text = text.replace('\r', "");
    let mut text = text.replace('\n', "");
    let padded_width: usize = width.saturating_sub(4).into();

    // truncate based on unicode chars
    if text.chars().count() > padded_width {
        // .truncate takes byte offsets and panics if not on a char boundary,
        // so we need to find the offset by iterating over chars
        let upto = text
            .char_indices()
            .map(|(i, _)| i)
            .nth(padded_width)
            .unwrap_or(text.len());

        text.truncate(upto);
        text.push_str(" ...");
    }
    text.into()
}

fn view(config: &ViewConfig) -> Result<()> {
    for path in &config.wav_path {
        println!("{}", path.to_string_lossy());
        let mut file = File::open(path)?;
        file.seek(SeekFrom::Start(0))?;
        let mut offset: u32 = 12;
        println!("      offset id         size summary");

        for res in wavrw::metadata_chunks(file)? {
            match res {
                Ok(chunk) => {
                    if config.detailed {
                        println!(
                            "{:12} {:8} {:10} {}",
                            offset,
                            chunk.id(),
                            chunk.size(),
                            chunk.item_summary_header()
                        );
                        let mut had_items = false;
                        for (key, value) in chunk.items() {
                            had_items = true;
                            println!("             |{key:>23} : {value}");
                        }
                        if had_items {
                            println!("             --------------------------------------");
                        }
                    } else {
                        println!(
                            "{:12} {:8} {:10} {}",
                            offset,
                            chunk.id(),
                            chunk.size(),
                            // trim to config.width minus width of previous text
                            trim(chunk.summary(), config.width.saturating_sub(30))
                        );
                    }
                    // remove offset calculations once handled by metadata_chunks()
                    offset += chunk.size() + 8;
                    // RIFF offsets must be on word boundaries (divisible by 2)
                    if offset % 2 == 1 {
                        offset += 1;
                    };
                }
                Err(err) => {
                    println!("ERROR: {err}");
                }
            }
        }
    }
    Ok(())
}

#[derive(Parser, Debug)]
#[command(long_about = None)]
struct ChunksConfig {
    /// directory to list
    #[arg(default_value = ".")]
    path: OsString,

    /// case insensitive file extensions to include
    #[arg(long, short, default_value_os = "wav")]
    ext: Vec<OsString>,

    /// recurse through subdirectories as well
    #[arg(long, short, default_value_t = false)]
    recurse: bool,
}

fn chunks(config: &ChunksConfig) -> Result<()> {
    walk_paths(&config.path.clone().into(), config)?;
    Ok(())
}

fn walk_paths(base_path: &PathBuf, config: &ChunksConfig) -> Result<()> {
    debug!("directory: {base_path:?}");

    let mut paths = fs::read_dir(base_path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    paths.sort_unstable();
    for path in paths {
        if path.is_dir() & config.recurse {
            walk_paths(&path, config)?;
        } else if let Some(ext) = path.extension() {
            let ext = ext.to_ascii_lowercase();
            if !config.ext.contains(&ext) {
                continue;
            }
            // eprintln!("--------> {path:?}");
            println!(
                "{}: {}",
                &path.strip_prefix(base_path)?.display(),
                chunks_for_path(&path)?
            );
        }
    }
    Ok(())
}

fn chunks_for_path(path: &PathBuf) -> Result<String> {
    let mut output = String::new();
    let mut chunks: Vec<String> = vec![];

    let file = File::open(path)?;
    let parse_res = wavrw::metadata_chunks(file);
    match parse_res {
        Ok(it) => {
            for chunk_res in it {
                match chunk_res {
                    Ok(chunk) => {
                        chunks.push(chunk.name());
                    }
                    Err(err) => {
                        println!("ERROR: {err}");
                    }
                }
            }
        }
        Err(err) => {
            println!("ERROR: {err}");
        }
    }
    output.push_str(&chunks.iter().join(", "));

    Ok(output)
}

fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let mut args = WavrwArgs::parse();

    match &mut args.command {
        Commands::View(config) => view(config),

        Commands::Chunks(config) => {
            // TODO: figure out how to do this in CLAP
            for ext in &mut config.ext {
                ext.make_ascii_lowercase();
            }
            chunks(config)
        }
    }
}

#[test]
fn verify_args() {
    use clap::CommandFactory;
    WavrwArgs::command().debug_assert();
}
