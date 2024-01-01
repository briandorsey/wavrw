use anyhow::Result;
use binrw::BinRead;
use clap::{Parser, Subcommand};
use std::ffi::OsString;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

struct WavChunk {
    offset: u32,
    chunk_id: [u8; 4],
    chunk_size: u32,
}

impl Display for WavChunk {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{:12} {:8} {:-10} ",
            self.offset,
            String::from_utf8_lossy(&self.chunk_id),
            self.chunk_size
        )
    }
}

// based on http://soundfile.sapp.org/doc/WaveFormat/
#[derive(BinRead, Debug)]
#[br(little)]
struct FmtChunk {
    #[br(big)]
    chunk_id: [u8; 4],
    chunk_size: u32,
    audio_format: u16,
    num_channels: u16,
    sample_rate: u32,
    byte_rate: u32,
    block_align: u16,
    bits_per_sample: u16,
    // padding to end of chunk_size
}

impl FmtChunk {
    fn _chunk_id() -> [u8; 4] {
        *b"fmt "
    }
}

impl Display for FmtChunk {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{} chan, {}/{}",
            self.num_channels,
            self.bits_per_sample,
            self.sample_rate,
            // TODO: audio_format
        )
    }
}

struct WavInfo {
    chunks: Vec<WavChunk>,
}

fn parse_wav(file: &mut File) -> Result<WavInfo> {
    let mut foffset: u32 = 0;

    let mut wi = WavInfo { chunks: vec![] };

    let mut buff4: [u8; 4] = [0; 4];
    file.read_exact(&mut buff4)?;
    let fourcc = buff4;

    file.read_exact(&mut buff4)?;
    let len = u32::from_le_bytes(buff4);

    file.read_exact(&mut buff4)?;
    let _format = String::from_utf8(buff4.to_vec())?;
    // TODO: abort parsing if not a "WAVE" file

    wi.chunks.push(WavChunk {
        offset: foffset,
        chunk_id: fourcc,
        chunk_size: len,
    });

    foffset += 12;

    loop {
        let n = file.read(&mut buff4)?;
        if n == 0 {
            break;
        }
        let chunk_id = buff4;

        let n = file.read(&mut buff4)?;
        if n == 0 {
            break;
        }
        let chunk_size = u32::from_le_bytes(buff4);
        wi.chunks.push(WavChunk {
            offset: foffset,
            chunk_id,
            chunk_size,
        });

        foffset += 8 + chunk_size;
        // TODO: research this to find the official rules:
        foffset += foffset % 2;

        file.seek(SeekFrom::Start(foffset.into()))?;
    }

    Ok(wi)
}

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
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match &args.command {
        Commands::View { wav_path } => {
            // TODO: move command logic into a function
            for path in wav_path {
                println!("{}", path.to_string_lossy());
                let mut file = File::open(path)?;
                let wavinfo = parse_wav(&mut file)?;
                println!(
                    "{:>12} {:4}{:>4} {:>10} description",
                    "offset", "id", "", "size"
                );
                for chunk in wavinfo.chunks.iter() {
                    print!("{chunk}");
                    if chunk.chunk_id == *b"fmt " {
                        //println!("fmt chunk @ {0}", chunk.offset);
                        file.seek(SeekFrom::Start(chunk.offset.into()))?;
                        let fmt = FmtChunk::read(&mut file).unwrap();
                        print!("{fmt}");
                        // println!("{:?}", file.stream_position());
                    } else {
                        print!("...");
                    };
                    println!();
                }
                println!();
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
