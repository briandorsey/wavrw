use anyhow::Result;
use binrw::BinRead;
use clap::{Parser, Subcommand};
use std::ffi::OsString;
use std::fs::File;
use std::io::Seek;
use std::io::SeekFrom;

mod wav;
use wav::Wav;

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
        wav_path: Vec<OsString>,

        #[arg(long, short)]
        _detailed: bool,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match &args.command {
        Commands::View {
            wav_path,
            _detailed,
        } => {
            // TODO: move command logic into a function
            for path in wav_path {
                println!("{}", path.to_string_lossy());
                let mut file = File::open(path)?;
                file.seek(SeekFrom::Start(0))?;
                let wav = Wav::read(&mut file)?;
                let mut offset: u32 = 12;
                println!("      offset chunk_id   size summary");

                for chunk in wav.chunks {
                    println!(
                        "{:12} {:8} {:>10} {}",
                        offset,
                        chunk.chunk_id(),
                        chunk.chunk_size(),
                        // TODO: truncate summary & add ... when long
                        chunk.summary()
                    );
                    offset += chunk.chunk_size();
                }
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
