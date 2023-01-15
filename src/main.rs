use anyhow::Result;
use clap::{Parser, Subcommand};
use std::fs::File;
use std::io::Read;

mod wav;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Summarize WAV file structure and metadata
    View {
        // TODO: figure out the right way to make this an OS path safe string
        /// One or more paths to WAV files
        wav_path: Vec<String>,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match &args.command {
        Commands::View { wav_path } => {
            // TODO: move command logic into a function, fix unwrap()
            for path in wav_path {
                println!("path: {}", path);
                let mut f = File::open(path)?;
                let mut buffer = [0; 8];

                f.read_exact(&mut buffer)?;
                let (_, wav_data) = wav::parse(&buffer).unwrap();
                println!("wav_data: {:?}", wav_data);
            }
        }
    }
    Ok(())
}

#[test]
fn verify_args() {
    use clap::CommandFactory;
    Args::command().debug_assert()
}
