use anyhow::Result;
// use binrw::BinRead;
use clap::{Parser, Subcommand};
use std::ffi::OsString;
use std::fs::File;
use std::io::Seek;
use std::io::SeekFrom;

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
        /// One or more paths to WAV files
        wav_path: Vec<OsString>,

        #[arg(long, short)]
        detailed: bool,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match &args.command {
        Commands::View { wav_path, detailed } => {
            // TODO: move command logic into a function
            for path in wav_path {
                println!("{}", path.to_string_lossy());
                let mut file = File::open(path)?;
                file.seek(SeekFrom::Start(0))?;
                let mut offset: u32 = 12;
                println!("      offset id         size summary");

                for (header, res) in wavrw::metadata_chunks(file)? {
                    match res {
                        Ok(chunk) => {
                            println!(
                                "{:12} {:8} {:>10} {}",
                                offset,
                                chunk.id(),
                                header.size,
                                // TODO: truncate summary & add ... when long
                                chunk.summary()
                            );
                            if *detailed {
                                for (key, value) in chunk.items() {
                                    println!("             |{key:>23} : {value}");
                                }
                            }
                            // remove offset calculations once handled by metadata_chunks()
                            offset += header.size + 8;
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
        }
    }
    Ok(())
}

#[test]
fn verify_args() {
    use clap::CommandFactory;
    Args::command().debug_assert()
}
