//! wavrw Command Line Interface

#![deny(missing_docs)]

use std::fmt::Write;
use std::fs;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::{ffi::OsString, io::BufReader};

use anyhow::Result;
use clap::{ArgAction, Parser, Subcommand, ValueEnum, crate_version};
use itertools::Itertools;
use tracing::Level;
use tracing::instrument;
use tracing_subscriber::FmtSubscriber;
use tracing_subscriber::fmt::format::FmtSpan;
use wavrw::{ChunkID, SizedChunk, SizedChunkEnum, Summarizable};

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
    View(ViewConfig),
    List(ListConfig),
    #[command(alias = "topics")]
    Topic(TopicConfig),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Format {
    Line,
    Summary,
    Detailed,
}

const WIDTH_DEFAULT: u16 = 80;

/// Summarize WAV file structure and metadata
#[derive(Parser, Debug)]
#[command(long_about = None)]
struct ViewConfig {
    /// One or more paths to WAV files
    wav_path: Vec<OsString>,

    /// Output format
    #[arg(long, short, value_enum, default_value_t = Format::Summary, group="output")]
    format: Format,

    /// Alias for: --format detailed
    #[arg(short = 'd', default_value_t = false, group = "output")]
    detailed: bool,

    #[arg(
        long,
        short = 'w',
        default_value_t = WIDTH_DEFAULT,
        help = "Trim output to <WIDTH> columns"
    )]
    width: u16,
}

impl Default for ViewConfig {
    fn default() -> Self {
        ViewConfig {
            wav_path: vec![],
            format: Format::Summary,
            detailed: false,
            width: WIDTH_DEFAULT,
        }
    }
}

/// List directories of files, show single line summary of chunks
#[derive(Parser, Debug)]
#[command(long_about = None)]
struct ListConfig {
    /// directory to list
    #[arg(default_value = ".")]
    path: OsString,

    /// Filter to only these extensions, case insensitive.
    ///
    /// To include multiple extenstions, use commas:
    /// Ex: --ext=wav,wave
    #[arg(long, short, value_delimiter = ',', default_value_os = "wav")]
    ext: Vec<OsString>,

    /// Recurse through subdirectories as well
    #[arg(long, short, default_value_t = false)]
    recurse: bool,
}

/// Print additional help and reference topics.
#[derive(Parser, Debug)]
#[command()]
struct TopicConfig {
    /// Topic to display information about
    #[arg(value_enum)]
    topic: Topic,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Topic {
    ///  Licences used by wavrw and all dependencies
    #[value(alias = "license")]
    Licenses,

    /// List currently supported chunks
    #[value(alias = "chunk")]
    Chunks,

    /// A Great Wave
    #[value(alias = "great_wave")]
    GreatWave,
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

#[instrument]
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

        print!("{}: ", path.to_string_lossy());
        let file = File::open(path)?;
        let file = BufReader::new(file);

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

#[instrument]
fn view_line(file: BufReader<File>) -> Result<String> {
    let mut out = String::new();
    let mut chunk_strings: Vec<String> = vec![];

    let mut wave = wavrw::WaveFile::from_reader(file)?;

    for result in wave.iter_chunks() {
        match result {
            // special case smpl to show loop count
            Ok(SizedChunkEnum::Smpl(chunk)) => {
                chunk_strings.push(format!(
                    "{}[{}]",
                    chunk.name(),
                    chunk.data.sample_loops.len()
                ));
            }
            // match on id() to catch all current and future LIST variants
            Ok(chunk) if chunk.id() == b"LIST" => {
                chunk_strings.push(format!("{}[{}]", chunk.name(), chunk.summary()));
            }
            Ok(chunk) => {
                chunk_strings.push(chunk.name());
            }
            Err(_) => {
                chunk_strings.push("ERROR".to_string());
            }
        }
    }
    out.push_str(&chunk_strings.iter().join(", "));

    Ok(out)
}

#[instrument]
fn view_summary(file: BufReader<File>, config: &ViewConfig) -> Result<String> {
    let mut out = "\n".to_string();
    writeln!(out, "      offset id              size summary")?;

    let mut wave = wavrw::WaveFile::from_reader(file)?;
    for result in wave.iter_chunks() {
        match result {
            Ok(chunk) => {
                writeln!(
                    out,
                    "{:>12} {:9} {:10} {}",
                    chunk.offset().map_or("???".to_string(), |v| v.to_string()),
                    chunk.name(),
                    chunk.size(),
                    trim(&chunk.summary(), config.width.saturating_sub(29))
                )?;
            }
            Err(err) => {
                writeln!(
                    out,
                    "{:>12} {:9} {:10} {}",
                    "???".to_string(),
                    "ERROR".to_string(),
                    "".to_string(),
                    trim(&err.to_string(), config.width.saturating_sub(29))
                )?;
            }
        };
    }
    Ok(out)
}

#[instrument]
fn view_detailed(file: BufReader<File>) -> Result<String> {
    let mut out = "\n".to_string();
    writeln!(out, "      offset id              size summary")?;

    let mut wave = wavrw::WaveFile::from_reader(file)?;
    for result in wave.iter_chunks() {
        match result {
            Ok(chunk) => {
                writeln!(
                    out,
                    "{:>12} {:9} {:10} {}",
                    chunk.offset().map_or("???".to_string(), |v| v.to_string()),
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
            }
            Err(err) => {
                writeln!(
                    out,
                    "{:>12} {:9} {:10} {}",
                    "???".to_string(),
                    "ERROR".to_string(),
                    "".to_string(),
                    err,
                )?;
            }
        }
    }
    Ok(out)
}

#[instrument]
fn list(config: &ListConfig) -> Result<()> {
    walk_paths(&config.path.clone().into(), config)?;
    Ok(())
}

#[instrument]
fn walk_paths(base_path: &PathBuf, config: &ListConfig) -> Result<()> {
    let mut paths = fs::read_dir(base_path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    paths.sort_unstable();
    for path in paths {
        if path.is_dir() & config.recurse {
            eprintln!("directory: {}", path.to_string_lossy());
            walk_paths(&path, config)?;
        } else if let Some(ext) = path.extension() {
            // config.ext entries are assumed to have been converted to lowercase already.
            let ext = ext.to_ascii_lowercase();
            if !config.ext.contains(&ext) {
                continue;
            }

            let path_name = path.clone();
            let path_name = path_name.to_string_lossy();
            let file = File::open(path)?;
            let file = BufReader::new(file);

            match view_line(file) {
                Ok(output) => println!("{path_name}: {output}"),
                Err(err) => println!("{path_name}: ERROR: {}", err),
            }
        }
    }
    Ok(())
}

#[instrument]
fn topic(config: &mut TopicConfig) -> Result<()> {
    match config.topic {
        Topic::Licenses => println!(include_str!("../../generated/licenses.txt")),
        Topic::Chunks => println!(include_str!("../../static/topic/chunks.txt")),
        Topic::GreatWave => {
            print!(include_str!("../../static/topic/wave.ansi"));
            println!("Great Wave by Hokusai");
        }
    }
    Ok(())
}

#[instrument]
fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        // TODO: --option to set log level and span events
        .with_max_level(Level::TRACE)
        .with_span_events(FmtSpan::NONE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let mut args = WavrwArgs::parse();

    match &mut args.command {
        Commands::View(config) => {
            // .detailed is an alias, update format
            if config.detailed {
                config.format = Format::Detailed;
            }
            view(config)
        }

        Commands::List(config) => {
            // Convert extensions to lowercase for case insensitive comparison later.
            for ext in &mut config.ext {
                ext.make_ascii_lowercase();
            }
            list(config)
        }
        Commands::Topic(config) => topic(config),
    }
}

#[test]
fn verify_args() {
    use clap::CommandFactory;
    WavrwArgs::command().debug_assert();
}
