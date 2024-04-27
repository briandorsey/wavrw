//!
//! Read (and someday write) wave audio file chunks with a focus on metadata.
//!
//! This is the API reference documentation, it is a bit dry.
//!
//! Iterate over all chunk objects from a file, returns [`SizedChunkEnum`]s with
//! convenience methods exposed via the [`SizedChunk`] trait:
//!
//! ```
//! # use std::fs::File;
//! # use std::io::BufReader;
//! use wavrw::{Summarizable, SizedChunk};
//!
//! let file = File::open("../test_wavs/example_a.wav")?;
//! let file = BufReader::new(file);
//! let mut wave = wavrw::Wave::from_reader(file)?;
//!
//! for result in wave.iter_chunks() {
//!     match result {
//!         Ok(chunk) => {
//!             println!(
//!                 "{:12} {:10} {}",
//!                 chunk.name(),
//!                 chunk.size(),
//!                 chunk.summary()
//!             )
//!         },
//!         Err(err) => {
//!             println!("{:12} {}", "ERROR".to_string(), err)
//!         }
//!     }
//! }
//! # Ok::<(), wavrw::WaveError>(())
//! ```
//!
//! Or parse a single chunk from a buffer:
//!
//! ```
//! # use binrw::BinRead;
//! # use wavrw::testing::hex_to_cursor;
//! # let mut buff = hex_to_cursor("666D7420 10000000 01000100 80BB0000 80320200 03001800");
//! use wavrw::{SizedChunkEnum, ChunkID, Summarizable, FourCC};
//!
//! let chunk = SizedChunkEnum::read(&mut buff).unwrap();
//!
//! // Use methods from SizedChunk trait on any chunk
//! assert_eq!(chunk.id(), FourCC(*b"fmt "));
//! assert_eq!(chunk.summary(), "PCM (0x0001), 1 chan, 24/48000".to_string());
//!
//! // Or match on type and handle various chunks individually
//! match chunk {
//!     SizedChunkEnum::Fmt(fmt) => println!("sample rate: {}", fmt.data.samples_per_sec),
//!     _ => ()
//! }
//! ```
//!
//!
//! NOTE: Many WAVE chunk specifications assume or specify ASCII strings. This
//! library parses ASCII strings as UTF8 encoded strings instead. All ASCII
//! characters are valid UTF8, and writing UTF8 strings appears to be common
//! practice in applications which write metadata.
//!
//! WARNING: This library does not attempt to interpret strings according to code
//! page settings specified via CSET. Setting character set information in CSET
//! chunks appears to be very rare, however if a file *did* specify an extended
//! codepage, text would likely be misinterpreted when decoded as UTF8. If you
//! do run into this situation, please consider filing an issue and if possible,
//! sharing sample files to test against so I can improve codepage handling.

extern crate alloc;

use core::default::Default;
use core::fmt::{Debug, Display, Formatter};
use std::error;
use std::io::BufRead;

use binrw::io::TakeSeekExt;
use binrw::io::{Read, Seek};
use binrw::{binrw, io::SeekFrom, BinRead, BinWrite, PosValue};
use tracing::{instrument, warn};

pub mod chunk;
use crate::chunk::adtl::ListAdtlChunk;
use crate::chunk::info::ListInfoChunk;
use crate::chunk::wavl::ListWavlChunk;
use crate::chunk::BextChunk;
use crate::chunk::CsetChunk;
use crate::chunk::CueChunk;
use crate::chunk::DataChunk;
use crate::chunk::FactChunk;
use crate::chunk::FllrChunk;
use crate::chunk::FmtChunk;
use crate::chunk::JunkChunk;
use crate::chunk::Md5Chunk;
use crate::chunk::PadChunk;
use crate::chunk::PlstChunk;
use crate::chunk::RiffChunk;
pub mod fixedstring;
pub mod testing;

// helper types
// ----

// Since const generics do not support arrays, generic structs are storing the
// FourCC id as a `u32`... which makes instantiation awkward. This is a helper
// function to make it a bit easier.
// revisit when Rust 1.79 is released
#[doc(hidden)]
pub const fn fourcc(id: &[u8; 4]) -> u32 {
    u32::from_le_bytes(*id)
}

/// const ID stored for every chunk with a parser.
pub trait KnownChunkID {
    /// RIFF chunk id.
    const ID: FourCC;
}

/// Retrieve a chunk ID from a chunk (even if dynamic, ex: [`UnknownChunk`]).
pub trait ChunkID {
    /// Returns the `FourCC` (chunk id) for the contained chunk.
    fn id(&self) -> FourCC;
}

/// A chunk with size information.
///
/// Parsed representation of the full chunk data as stored. Likely a [`KnownChunk<T>`]
/// where T is the inner chunk specific data.
pub trait SizedChunk: Summarizable + Debug {
    /// The logical (used) size in bytes of the chunk.
    fn size(&self) -> u32;

    /// The byte offset from the start of the read data stream.
    fn offset(&self) -> Option<u64>;
}

/// Utility methods for describing any chunk.
pub trait Summarizable: ChunkID {
    /// Returns a short text summary of the contents of the chunk.
    fn summary(&self) -> String;

    /// User friendly name of the chunk, usually the chunk id
    ///
    /// An ascii friendly chunk name, with whitespace removed. Chunks with
    /// sub types (forms, in RIFF terms) include their sub type in the name.
    /// Ex: [`ListInfoChunk`] is "LIST-INFO".    
    fn name(&self) -> String {
        self.id().to_string().trim().to_string()
    }

    /// Returns an iterator over a sequence of contents of the
    /// chunk as strings (field, value).
    fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        Box::new(core::iter::empty())
    }

    /// Alternative header for use above `items()`.
    fn item_summary_header(&self) -> String {
        self.summary()
    }
}

impl<T> ChunkID for T
where
    T: KnownChunkID,
{
    fn id(&self) -> FourCC {
        T::ID
    }
}

/// RIFF FOURCC type. Four bytes, often readable id of a chunk.
///
/// Used as chunk ids, [`ListInfoData.list_type`], etc.
#[binrw]
#[brw(big)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct FourCC(pub [u8; 4]);

impl Display for FourCC {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}", String::from_utf8_lossy(&self.0),)?;
        Ok(())
    }
}

impl Debug for FourCC {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "FourCC(*b\"{}\"=", self)?;
        write!(f, "{:?})", &self.0)?;
        Ok(())
    }
}

impl From<&[u8; 4]> for FourCC {
    fn from(value: &[u8; 4]) -> Self {
        FourCC(*value)
    }
}

impl<'a> PartialEq<&'a [u8; 4]> for FourCC {
    fn eq(&self, other: &&'a [u8; 4]) -> bool {
        self == FourCC(**other)
    }
}

// needed for assert in br() attribute
impl<'a> PartialEq<&'a FourCC> for FourCC {
    fn eq(&self, other: &&'a FourCC) -> bool {
        self == *other
    }
}

impl<'a> PartialEq<FourCC> for &'a FourCC {
    fn eq(&self, other: &FourCC) -> bool {
        *self == other
    }
}

// parsing helpers
// ----

/// Wave parsing Errors.
#[derive(Debug)]
#[non_exhaustive]
pub enum WaveError {
    /// An unknown FourCC code, could not process.
    UnknownFourCC {
        /// The encountered FourCC code.
        found: FourCC,

        /// Summary of the context.
        message: String,
    },

    /// An error occurred in the underlying reader while reading or seeking to data.
    ///
    /// Contains an [`std::io::Error`]
    Io(std::io::Error),

    /// An error occured while parsing wav chunk data.
    ///
    /// A string representation of the underlying error.
    Parse {
        /// The byte position of the unparsable data in the reader.
        pos: Option<u64>,

        /// Summary of the underlying parsing error.
        message: String,
    },
}

impl error::Error for WaveError {}

impl Display for WaveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            WaveError::UnknownFourCC { message, .. } => write!(f, "UnknownFourCC: {}", message),
            WaveError::Io(err) => write!(f, "Io: {}", err),
            WaveError::Parse { message, .. } => write!(f, "Parse: {}", message),
        }
    }
}

impl From<std::io::Error> for WaveError {
    fn from(err: std::io::Error) -> Self {
        WaveError::Io(err)
    }
}

/// Map `binrw::Error` to Parse, to avoid introducing external dependencies on it.
impl From<binrw::Error> for WaveError {
    fn from(err: binrw::Error) -> Self {
        #[allow(clippy::match_same_arms)] // so _ is its own case
        match err {
            binrw::Error::BadMagic { pos, .. }
            | binrw::Error::Custom { pos, .. }
            | binrw::Error::EnumErrors { pos, .. }
            | binrw::Error::NoVariantMatch { pos } => WaveError::Parse {
                pos: Some(pos),
                message: err.to_string(),
            },
            binrw::Error::AssertFail { pos, message } => WaveError::Parse {
                pos: Some(pos),
                message,
            },
            binrw::Error::Io(_) | binrw::Error::Backtrace(_) => WaveError::Parse {
                pos: None,
                message: err.to_string(),
            },
            _ => WaveError::Parse {
                pos: None,
                message: err.to_string(),
            },
        }
    }
}

/// Implements `Wave.iter_chunks()`
#[derive(Debug)]
pub struct WaveIterator<'a, R>
where
    R: Read + Seek + Debug + BufRead,
{
    reader: &'a mut R,
    riff_size: u32,
    finished: bool,
}

impl<'a, R> WaveIterator<'a, R>
where
    R: Read + Seek + Debug + BufRead,
{
    fn parse_next_chunk(&mut self) -> Result<(SizedChunkEnum, u64), WaveError> {
        let mut offset = self.reader.stream_position()?;
        let mut buff: [u8; 4] = [0; 4];

        let chunk_id = {
            self.reader.read_exact(&mut buff)?;
            buff
        };
        let chunk_size = {
            self.reader.read_exact(&mut buff)?;
            u32::from_le_bytes(buff)
        };

        self.reader.seek(SeekFrom::Current(-8))?;

        let chunk = SizedChunkEnum::read(&mut self.reader)?;

        // setup for next iteration
        offset += chunk_size as u64 + 8;
        // RIFF offsets must be on word boundaries (divisible by 2)
        if offset % 2 == 1 {
            offset += 1;
        };

        // Returning after parsing a chunk would cause a missing chunk.
        // Oh dang, this is tricky. We actually successfully (probably)
        // parsed the chunk, but there could be additional errors.
        // Should we have a mechanism for annotating chunks with
        // warnings and notes from the parsers?
        // https://github.com/briandorsey/wavrw/issues/95
        // if/when fixed, update docs on iter_chunks()
        let stream_position = self.reader.stream_position()?;
        if offset != stream_position {
            warn!("{:?}: parsed less data than chunk size", FourCC(chunk_id));
            self.reader.seek(SeekFrom::Start(offset))?;
        }

        Ok((chunk, offset))
    }
}

impl<'a, R> Iterator for WaveIterator<'a, R>
where
    R: Read + Seek + Debug + BufRead,
{
    type Item = Result<SizedChunkEnum, WaveError>;

    #[instrument]
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let (chunk, offset) = match self.parse_next_chunk() {
            Ok(v) => v,
            Err(err) => {
                // TODO... hmmm... would be great to continue after normal errors
                // but if we remove this, we get an infinite loop on files
                // with a larger riff.size than disk size.
                self.finished = true;
                return Some(Err(err));
            }
        };

        if offset >= self.riff_size as u64 {
            self.finished = true;
        };
        Some(Ok(chunk))
    }
}

/// Wrapper around RIFF-WAVE data.
pub struct Wave<R>
where
    R: Read + Seek + Debug + BufRead,
{
    bytes: R,
    riff: RiffChunk,
}

impl<R> Wave<R>
where
    R: Read + Seek + Debug + BufRead,
{
    /// Create a new Wave handle. This keeps a reference to the data
    /// until dropped.
    pub fn from_reader(mut reader: R) -> Result<Self, WaveError> {
        let riff = RiffChunk::read(&mut reader).map_err(std::io::Error::other)?;
        if riff.form_type != FourCC(*b"WAVE") {
            return Err(WaveError::UnknownFourCC {
                found: riff.form_type,
                message: format!(
                    "not a wave file. Expected RIFF form_type 'WAVE', found: {}",
                    riff.form_type
                ),
            });
        }
        Ok(Self {
            bytes: reader,
            riff,
        })
    }

    /// Parses WAV (RIFF-WAVE) data, returns all known chunks.
    ///
    /// It attempts to continue parsing even if some chunks have parsing errors.
    /// In some cases, it may return before reading all chunks, such as:
    ///
    /// * when an error results from parsing the RIFF container
    /// * the data is not a WAVE form type
    /// * an IO error occurs while seeking before or after parsing chunk data
    #[instrument]
    pub fn iter_chunks<'a>(&'a mut self) -> WaveIterator<'a, R> {
        WaveIterator {
            reader: &mut self.bytes,
            riff_size: self.riff.size,
            finished: false,
        }
    }
}

impl<R> Debug for Wave<R>
where
    R: Read + Seek + Debug + BufRead,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Wave").finish()
    }
}

// parsing structs
// ----

type KCArgs = (u32,);

/// A generic wrapper around chunk data, handling ID, size and padding.
#[binrw]
#[brw(little)]
#[br(stream = r)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KnownChunk<
    T: for<'a> BinRead<Args<'a> = KCArgs> + for<'a> BinWrite<Args<'a> = ()> + KnownChunkID,
> {
    /// Calculated byte offset from the beginning of the data stream or None.
    ///
    /// Ignored when writing chunks.
    #[br(try_calc = Some(r.stream_position()).transpose())]
    #[bw(ignore)]
    pub offset: Option<u64>,

    /// RIFF chunk id.
    #[br(temp, assert(id == T::ID))]
    #[bw(calc = T::ID)]
    pub id: FourCC,

    // TODO: calc by querying content + extra_bytes.len() when writing, or seeking back after you know
    /// RIFF chunk size in bytes.
    pub size: u32,

    #[br(temp)]
    #[bw(ignore)]
    begin_pos: PosValue<()>,

    // take_seek() to ensure that we don't read outside the bounds for this chunk
    /// Generic inner data struct.
    #[br(map_stream = |r| r.take_seek(size as u64), args(size))]
    pub data: T,

    // assert for better error message if too many bytes processed
    // seems like it should be impossible, but found a case where T
    // used align_after to pad bytes.
    #[br(temp, assert((end_pos.pos - begin_pos.pos) <= size as u64, "(end_pos.pos - begin_pos.pos) <= size while parsing {}. end: {}, begin: {}, size: {}", T::ID, end_pos.pos, begin_pos.pos, size))]
    #[bw(ignore)]
    end_pos: PosValue<()>,

    // calculate how much was read, then read...
    /// Any extra bytes in the chunk after parsing.
    ///
    /// May include RIFF padding byte.
    #[brw(align_after = 2)]
    #[br(count = size as u64 - (end_pos.pos - begin_pos.pos))]
    pub extra_bytes: Vec<u8>,
}

impl<T> Display for KnownChunk<T>
where
    T: for<'a> BinRead<Args<'a> = KCArgs>
        + for<'a> BinWrite<Args<'a> = ()>
        + KnownChunkID
        + Summarizable,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} {}", self.name(), self.data.summary())
    }
}

impl<T> KnownChunkID for KnownChunk<T>
where
    T: for<'a> BinRead<Args<'a> = KCArgs> + for<'a> BinWrite<Args<'a> = ()> + KnownChunkID,
{
    const ID: FourCC = T::ID;
}

impl<T> SizedChunk for KnownChunk<T>
where
    T: for<'a> BinRead<Args<'a> = KCArgs>
        + for<'a> BinWrite<Args<'a> = ()>
        + KnownChunkID
        + Summarizable
        + Debug,
{
    fn size(&self) -> u32 {
        self.size
    }

    fn offset(&self) -> Option<u64> {
        self.offset
    }
}

impl<T> Summarizable for KnownChunk<T>
where
    T: for<'a> BinRead<Args<'a> = KCArgs>
        + for<'a> BinWrite<Args<'a> = ()>
        + KnownChunkID
        + Summarizable,
{
    fn summary(&self) -> String {
        self.data.summary()
    }
    fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        self.data.items()
    }

    fn item_summary_header(&self) -> String {
        self.data.item_summary_header()
    }

    fn name(&self) -> String {
        self.data.name()
    }
}

// impl<T> Chunk for KnownChunk<T> where
//     T: for<'a> BinRead<Args<'a> = KCArgs>
//         + for<'a> BinWrite<Args<'a> = ()>
//         + KnownChunkID
//         + Summarizable
//         + Debug
// {
// }

/// Raw chunk data container for unrecognized chunks
#[binrw]
#[brw(little)]
#[br(stream = r)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnknownChunk {
    /// Calculated offset from the beginning of the data stream this chunk is from or None.
    ///
    /// Ignored when writing chunks.
    #[br(try_calc = Some(r.stream_position()).transpose())]
    #[bw(ignore)]
    pub offset: Option<u64>,

    /// RIFF chunk id.
    pub id: FourCC,

    /// RIFF chunk size in bytes.
    pub size: u32,

    /// Unparsed chunk data as bytes.
    #[brw(align_after = 2)]
    #[br(count = size )]
    pub raw: Vec<u8>,
}

impl Display for UnknownChunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "UnknownChunk({}, {})", self.id, self.size)
    }
}

impl Default for UnknownChunk {
    fn default() -> Self {
        Self {
            id: FourCC(*b"UNKN"),
            size: 0,
            raw: Vec::new(),
            offset: None,
        }
    }
}

impl ChunkID for UnknownChunk {
    fn id(&self) -> FourCC {
        self.id
    }
}

impl SizedChunk for UnknownChunk {
    fn size(&self) -> u32 {
        self.size
    }

    fn offset(&self) -> Option<u64> {
        self.offset
    }
}

impl Summarizable for UnknownChunk {
    fn summary(&self) -> String {
        "...".to_string()
    }
}

// impl Chunk for UnknownChunk {}

/// All chunk structs as an enum
#[allow(missing_docs)]
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SizedChunkEnum {
    Fmt(FmtChunk),
    Data(DataChunk),
    Fact(FactChunk),
    Cue(CueChunk),
    Info(ListInfoChunk),
    Adtl(ListAdtlChunk),
    Wavl(ListWavlChunk),
    Cset(CsetChunk),
    Plst(PlstChunk),
    Bext(Box<BextChunk>),
    Md5(Md5Chunk),
    Fllr(FllrChunk),
    Junk(JunkChunk),
    Pad(PadChunk),
    Unknown(UnknownChunk),
}

impl Display for SizedChunkEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let display_string = match self {
            SizedChunkEnum::Fmt(e) => e.to_string(),
            SizedChunkEnum::Data(e) => e.to_string(),
            SizedChunkEnum::Fact(e) => e.to_string(),
            SizedChunkEnum::Cue(e) => e.to_string(),
            SizedChunkEnum::Info(e) => e.to_string(),
            SizedChunkEnum::Adtl(e) => e.to_string(),
            SizedChunkEnum::Wavl(e) => e.to_string(),
            SizedChunkEnum::Cset(e) => e.to_string(),
            SizedChunkEnum::Plst(e) => e.to_string(),
            SizedChunkEnum::Bext(e) => e.to_string(),
            SizedChunkEnum::Md5(e) => e.to_string(),
            SizedChunkEnum::Fllr(e) => e.to_string(),
            SizedChunkEnum::Junk(e) => e.to_string(),
            SizedChunkEnum::Pad(e) => e.to_string(),
            SizedChunkEnum::Unknown(e) => e.to_string(),
        };
        write!(f, "{}", display_string)
    }
}

impl ChunkID for SizedChunkEnum {
    fn id(&self) -> FourCC {
        match self {
            SizedChunkEnum::Fmt(e) => e.id(),
            SizedChunkEnum::Data(e) => e.id(),
            SizedChunkEnum::Fact(e) => e.id(),
            SizedChunkEnum::Cue(e) => e.id(),
            SizedChunkEnum::Info(e) => e.id(),
            SizedChunkEnum::Adtl(e) => e.id(),
            SizedChunkEnum::Wavl(e) => e.id(),
            SizedChunkEnum::Cset(e) => e.id(),
            SizedChunkEnum::Plst(e) => e.id(),
            SizedChunkEnum::Bext(e) => e.id(),
            SizedChunkEnum::Md5(e) => e.id(),
            SizedChunkEnum::Fllr(e) => e.id(),
            SizedChunkEnum::Junk(e) => e.id(),
            SizedChunkEnum::Pad(e) => e.id(),
            SizedChunkEnum::Unknown(e) => e.id(),
        }
    }
}

impl SizedChunk for SizedChunkEnum {
    fn size(&self) -> u32 {
        match self {
            SizedChunkEnum::Fmt(e) => e.size,
            SizedChunkEnum::Data(e) => e.size,
            SizedChunkEnum::Fact(e) => e.size,
            SizedChunkEnum::Cue(e) => e.size,
            SizedChunkEnum::Info(e) => e.size,
            SizedChunkEnum::Adtl(e) => e.size,
            SizedChunkEnum::Wavl(e) => e.size,
            SizedChunkEnum::Cset(e) => e.size,
            SizedChunkEnum::Plst(e) => e.size,
            SizedChunkEnum::Bext(e) => e.size,
            SizedChunkEnum::Md5(e) => e.size,
            SizedChunkEnum::Fllr(e) => e.size,
            SizedChunkEnum::Junk(e) => e.size,
            SizedChunkEnum::Pad(e) => e.size,
            SizedChunkEnum::Unknown(e) => e.size,
        }
    }

    fn offset(&self) -> Option<u64> {
        match self {
            SizedChunkEnum::Fmt(e) => e.offset,
            SizedChunkEnum::Data(e) => e.offset,
            SizedChunkEnum::Fact(e) => e.offset,
            SizedChunkEnum::Cue(e) => e.offset,
            SizedChunkEnum::Info(e) => e.offset,
            SizedChunkEnum::Adtl(e) => e.offset,
            SizedChunkEnum::Wavl(e) => e.offset,
            SizedChunkEnum::Cset(e) => e.offset,
            SizedChunkEnum::Plst(e) => e.offset,
            SizedChunkEnum::Bext(e) => e.offset,
            SizedChunkEnum::Md5(e) => e.offset,
            SizedChunkEnum::Fllr(e) => e.offset,
            SizedChunkEnum::Junk(e) => e.offset,
            SizedChunkEnum::Pad(e) => e.offset,
            SizedChunkEnum::Unknown(e) => e.offset,
        }
    }
}

impl Summarizable for SizedChunkEnum {
    fn summary(&self) -> String {
        match self {
            SizedChunkEnum::Fmt(e) => e.summary(),
            SizedChunkEnum::Data(e) => e.summary(),
            SizedChunkEnum::Fact(e) => e.summary(),
            SizedChunkEnum::Cue(e) => e.summary(),
            SizedChunkEnum::Info(e) => e.summary(),
            SizedChunkEnum::Adtl(e) => e.summary(),
            SizedChunkEnum::Wavl(e) => e.summary(),
            SizedChunkEnum::Cset(e) => e.summary(),
            SizedChunkEnum::Plst(e) => e.summary(),
            SizedChunkEnum::Bext(e) => e.summary(),
            SizedChunkEnum::Md5(e) => e.summary(),
            SizedChunkEnum::Fllr(e) => e.summary(),
            SizedChunkEnum::Junk(e) => e.summary(),
            SizedChunkEnum::Pad(e) => e.summary(),
            SizedChunkEnum::Unknown(e) => e.summary(),
        }
    }

    fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        match self {
            SizedChunkEnum::Fmt(e) => Box::new(e.items()),
            SizedChunkEnum::Cue(e) => Box::new(e.items()),
            SizedChunkEnum::Info(e) => Box::new(e.items()),
            SizedChunkEnum::Adtl(e) => Box::new(e.items()),
            SizedChunkEnum::Wavl(e) => Box::new(e.items()),
            SizedChunkEnum::Cset(e) => Box::new(e.items()),
            SizedChunkEnum::Plst(e) => Box::new(e.items()),
            SizedChunkEnum::Bext(e) => Box::new(e.items()),
            SizedChunkEnum::Data(_)
            | SizedChunkEnum::Fact(_)
            | SizedChunkEnum::Md5(_)
            | SizedChunkEnum::Fllr(_)
            | SizedChunkEnum::Junk(_)
            | SizedChunkEnum::Pad(_)
            | SizedChunkEnum::Unknown(_) => Box::new(core::iter::empty()),
        }
    }

    fn name(&self) -> String {
        match self {
            SizedChunkEnum::Fmt(e) => e.name(),
            SizedChunkEnum::Data(e) => e.name(),
            SizedChunkEnum::Fact(e) => e.name(),
            SizedChunkEnum::Cue(e) => e.name(),
            SizedChunkEnum::Info(e) => e.name(),
            SizedChunkEnum::Adtl(e) => e.name(),
            SizedChunkEnum::Wavl(e) => e.name(),
            SizedChunkEnum::Cset(e) => e.name(),
            SizedChunkEnum::Plst(e) => e.name(),
            SizedChunkEnum::Bext(e) => e.name(),
            SizedChunkEnum::Md5(e) => e.name(),
            SizedChunkEnum::Fllr(e) => e.name(),
            SizedChunkEnum::Junk(e) => e.name(),
            SizedChunkEnum::Pad(e) => e.name(),
            SizedChunkEnum::Unknown(e) => e.name(),
        }
    }

    fn item_summary_header(&self) -> String {
        match self {
            SizedChunkEnum::Fmt(e) => e.item_summary_header(),
            SizedChunkEnum::Data(e) => e.item_summary_header(),
            SizedChunkEnum::Fact(e) => e.item_summary_header(),
            SizedChunkEnum::Cue(e) => e.item_summary_header(),
            SizedChunkEnum::Info(e) => e.item_summary_header(),
            SizedChunkEnum::Adtl(e) => e.item_summary_header(),
            SizedChunkEnum::Wavl(e) => e.item_summary_header(),
            SizedChunkEnum::Cset(e) => e.item_summary_header(),
            SizedChunkEnum::Plst(e) => e.item_summary_header(),
            SizedChunkEnum::Bext(e) => e.item_summary_header(),
            SizedChunkEnum::Md5(e) => e.item_summary_header(),
            SizedChunkEnum::Fllr(e) => e.item_summary_header(),
            SizedChunkEnum::Junk(e) => e.item_summary_header(),
            SizedChunkEnum::Pad(e) => e.item_summary_header(),
            SizedChunkEnum::Unknown(e) => e.item_summary_header(),
        }
    }
}

// impl Chunk for ChunkEnum {}

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn fourcc() {
        let f = FourCC(*b"TST ");

        println!("Display: {f}");
        assert_eq!(f.to_string(), "TST ");
        println!("Debug: {f:?}");
        assert_eq!(format!("{f:?}"), r#"FourCC(*b"TST "=[84, 83, 84, 32])"#);
    }

    #[test]
    fn knownchunk_as_trait() {
        let md5 = Md5Chunk {
            offset: Some(0),
            size: 16,
            data: chunk::Md5 { md5: 0 },
            extra_bytes: vec![],
        };
        // ensure trait bounds are satisfied
        let mut _trt: Box<dyn SizedChunk> = Box::new(md5);
    }

    #[test]
    fn chunkenum_as_trait() {
        let md5 = SizedChunkEnum::Md5(Md5Chunk {
            offset: None,
            size: 16,
            data: chunk::Md5 { md5: 0 },
            extra_bytes: vec![],
        });
        // ensure trait bounds are satisfied
        let mut _trt: Box<dyn SizedChunk> = Box::new(md5);
    }

    // compile time check to ensure all chunks implement consistent traits
    fn has_standard_traits<T>()
    where
        T: Debug + Display + Clone + PartialEq + Eq + core::hash::Hash + Send + Sync,
    {
    }

    #[test]
    fn consistent_traits() {
        has_standard_traits::<RiffChunk>();

        // this Enum transitively ensures the traits of all subchunks
        has_standard_traits::<SizedChunkEnum>();
    }
}
