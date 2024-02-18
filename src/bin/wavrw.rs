use anyhow::Result;
use clap::{Parser, Subcommand};
use itertools::Itertools;
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Summarize WAV file structure and metadata
    View(ViewConfig),
    /// List chunks contained in each file
    Chunks(ChunksConfig),
}

#[derive(Parser, Debug)]
#[command(long_about = None)]
struct ViewConfig {
    /// One or more paths to WAV files
    wav_path: Vec<OsString>,

    #[arg(long, short)]
    detailed: bool,
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
                    println!(
                        "{:12} {:8} {:>10} {}",
                        offset,
                        chunk.id(),
                        chunk.size(),
                        // TODO: truncate summary & add ... when long
                        chunk.summary()
                    );
                    if config.detailed {
                        let mut had_items = false;
                        for (key, value) in chunk.items() {
                            had_items = true;
                            println!("             |{key:>23} : {value}");
                        }
                        if had_items {
                            println!("             --------------------------------------");
                        }
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
    eprintln!("directory: {base_path:?}");

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
    for res in wavrw::metadata_chunks(file)? {
        match res {
            Ok(chunk) => {
                chunks.push(chunk.name().to_string());
            }
            Err(err) => {
                println!("ERROR: {err}");
            }
        }
    }
    output.push_str(&chunks.iter().map(|c| c.trim()).join(", "));

    Ok(output)
}

fn main() -> Result<()> {
    let mut args = Args::parse();

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
    Args::command().debug_assert()
}
