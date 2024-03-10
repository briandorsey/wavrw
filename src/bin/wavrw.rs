//! wavrw Command Line Interface

#![deny(missing_docs)]

use std::ffi::OsString;
use std::fmt::Write;
use std::fs;
use std::fs::File;
use std::io;
use std::path::PathBuf;

use anyhow::Result;
use clap::{crate_version, ArgAction, Parser, Subcommand, ValueEnum};
use itertools::Itertools;
use tracing::Level;
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
    /// List directories of files, show single line summary of chunks
    List(ChunksConfig),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Format {
    Line,
    Summary,
    Detailed,
}

const WIDTH_DEFAULT: u16 = 80;

#[derive(Parser, Debug)]
#[command(long_about = None)]
struct ViewConfig {
    /// One or more paths to WAV files
    wav_path: Vec<OsString>,

    /// output format
    #[arg(long, short, value_enum, default_value_t = Format::Summary)]
    format: Format,

    #[arg(
        long,
       short = 'w',
        default_value_t = WIDTH_DEFAULT,
        help = "trim output to <WIDTH> columns"
    )]
    width: u16,
}

impl Default for ViewConfig {
    fn default() -> Self {
        ViewConfig {
            wav_path: vec![],
            format: Format::Summary,
            width: WIDTH_DEFAULT,
        }
    }
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

fn trim(text: &str, width: u16) -> String {
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
    text
}

fn view(config: &ViewConfig) -> Result<()> {
    for path in &config.wav_path {
        let path = PathBuf::from(path);
        if path.is_dir() {
            println!(
                "{} is a directory, skipping. Consider using 'list' command for directories.",
                path.display()
            );
            continue;
        }

        println!("{}", path.to_string_lossy());
        let file = File::open(path)?;

        match config.format {
            Format::Line => {
                println!("{}", view_line(file)?);
            }
            Format::Summary => {
                println!("{}", view_summary(file, config)?);
            }
            Format::Detailed => {
                println!("{}", view_detailed(file)?);
            }
        }
    }
    Ok(())
}

fn view_line(file: File) -> Result<String> {
    let mut out = String::new();
    out.push_str("    ");
    let mut chunks: Vec<String> = vec![];

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
    out.push_str(&chunks.iter().join(", "));

    Ok(out)
}

fn view_summary(file: File, config: &ViewConfig) -> Result<String> {
    let mut out = String::new();

    let mut offset: u32 = 12;
    println!("      offset id              size summary");

    for res in wavrw::metadata_chunks(file)? {
        match res {
            Ok(chunk) => {
                writeln!(
                    out,
                    "{:12} {:9} {:10} {}",
                    offset,
                    chunk.name(),
                    chunk.size(),
                    trim(&chunk.summary(), config.width.saturating_sub(29))
                )?;

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
    Ok(out)
}

fn view_detailed(file: File) -> Result<String> {
    let mut out = String::new();

    let mut offset: u32 = 12;
    println!("      offset id              size summary");

    for res in wavrw::metadata_chunks(file)? {
        match res {
            Ok(chunk) => {
                writeln!(
                    out,
                    "{:12} {:9} {:10} {}",
                    offset,
                    chunk.name(),
                    chunk.size(),
                    chunk.item_summary_header()
                )?;
                let mut had_items = false;
                for (key, value) in chunk.items() {
                    had_items = true;
                    writeln!(out, "             |{key:>23} : {value}")?;
                }
                if had_items {
                    writeln!(out, "             --------------------------------------")?;
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
    Ok(out)
}

fn chunks(config: &ChunksConfig) -> Result<()> {
    walk_paths(&config.path.clone().into(), config)?;
    Ok(())
}

fn walk_paths(base_path: &PathBuf, config: &ChunksConfig) -> Result<()> {
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

            view(&ViewConfig {
                wav_path: vec![path.into()],
                format: Format::Line,
                ..Default::default()
            })?;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let mut args = WavrwArgs::parse();

    match &mut args.command {
        Commands::View(config) => view(config),

        Commands::List(config) => {
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
