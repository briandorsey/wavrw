use anyhow::Result;
use binrw::BinRead;
use binrw::NullString;
use clap::{Parser, Subcommand};
use std::ffi::OsString;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

mod wav;
use wav::Wav;

struct FileChunk {
    offset: u32,
    chunk_id: [u8; 4],
    chunk_size: u32,
    chunk: Chunk,
}

impl Display for FileChunk {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{:12} {:8} {:-10} {}",
            self.offset,
            String::from_utf8_lossy(&self.chunk_id),
            self.chunk_size,
            self.chunk
        )
    }
}

// based on http://soundfile.sapp.org/doc/WaveFormat/
#[derive(BinRead, Debug)]
#[br(little)]
#[allow(dead_code)]
struct FmtChunk {
    audio_format: u16,
    num_channels: u16,
    sample_rate: u32,
    byte_rate: u32,
    block_align: u16,
    bits_per_sample: u16,
    // currently ignoring optional extra data
    // padding to end of chunk_size
}

#[allow(dead_code)]
impl FmtChunk {
    const DESC: &'static str = "desc of 'fmt ' chunk TOOD";
    const SAMPLE_RATE_DESC: &'static str = "the sample rate TODO";
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

#[derive(BinRead, Debug)]
#[br(little)]
#[allow(dead_code)]
struct ListChunk {
    #[br(big)]
    chunk_id: [u8; 4],
    chunk_size: u32,
    list_type: [u8; 4],
    // need to add magic here to choose the right enum
    // items: ListType,
}

impl Display for ListChunk {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        writeln!(
            f,
            "{}",
            String::from_utf8_lossy(&self.list_type),
            // self.items
        )?;
        Ok(())
    }
}

#[derive(BinRead, Debug)]
#[br(little)]
//#[allow(dead_code)]
struct InfoItem {
    #[br(big)]
    chunk_id: [u8; 4],
    chunk_size: u32,
    content: NullString,
}

impl Display for InfoItem {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{:12}     {:4} {:-10} {}",
            "???",
            String::from_utf8_lossy(&self.chunk_id),
            self.chunk_size,
            self.content,
        )
    }
}

// #[derive(BinRead, Debug)]
// #[br(little)]
#[allow(dead_code)]
enum ListType {
    UnParsed,
    // #[br(args { default: ListType::UnParsed})]
    // Info(Vec<InfoItem>),
}

impl Display for ListType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            ListType::UnParsed => write!(f, "Unparsed"),
            // ListType::Info(item) => write!(f, "\n{item:?}"),
        }
    }
}

enum Chunk {
    UnParsed,
    Fmt(FmtChunk),
    List(ListChunk),
}

impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        let output = match self {
            Chunk::UnParsed => "...".to_string(),
            Chunk::Fmt(chunk) => format!("{chunk}"),
            Chunk::List(chunk) => format!("{chunk}"),
        };
        write!(f, "{}", output)
    }
}
struct WavInfo {
    chunks: Vec<FileChunk>,
}

fn parse_wav(file: &mut File) -> Result<WavInfo> {
    let mut foffset: u32 = 0;

    let mut wi = WavInfo { chunks: vec![] };

    let mut buff4: [u8; 4] = [0; 4];
    file.read_exact(&mut buff4)?;
    let _fourcc = buff4;

    file.read_exact(&mut buff4)?;
    let _len = u32::from_le_bytes(buff4);

    file.read_exact(&mut buff4)?;
    let _format = String::from_utf8(buff4.to_vec())?;
    // TODO: abort parsing if not a "WAVE" file

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

        let chunk = match &chunk_id {
            b"fmt " => {
                let fmt = FmtChunk::read(file)?;
                Chunk::Fmt(fmt)
            }
            b"LIST" => {
                file.seek(SeekFrom::Current(-8))?;
                let list = ListChunk::read(file)?;
                Chunk::List(list)
            }
            _ => Chunk::UnParsed,
        };

        wi.chunks.push(FileChunk {
            offset: foffset,
            chunk_id,
            chunk_size,
            chunk,
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
                let wavinfo = parse_wav(&mut file)?;
                println!(
                    "{:>12} {:4}{:>4} {:>10} summary",
                    "offset", "id", "", "size"
                );
                for chunk in wavinfo.chunks.iter() {
                    print!("{chunk}");
                    println!();
                }
                println!();
                file.seek(SeekFrom::Start(0))?;
                // wav module version
                let wav = Wav::read(&mut file)?;
                let mut offset: u32 = 12;
                for chunk in wav.chunks {
                    println!("{}, {}, {}", offset, chunk.chunk_id(), chunk.chunk_size());
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
