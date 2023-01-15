use clap::{Parser, Subcommand};

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
        wav_path: Vec<String>,
    },
}

fn main() {
    let args = Args::parse();

    match &args.command {
        Commands::View { wav_path } => {
            for path in wav_path {
                println!("path: {}", path);
            }
        }
    }
}
