#![doc = include_str!("lib.md")]

extern crate alloc;

use core::default::Default;
use core::fmt::{Debug, Display, Formatter};
use std::error;
use std::io::BufRead;

use binrw::io::TakeSeekExt;
use binrw::io::{Read, Seek};
use binrw::{binrw, io::SeekFrom, BinRead, BinWrite, PosValue};
use tracing::{instrument, trace_span, warn};

pub mod chunk;
use crate::chunk::adtl::{ListAdtl, ListAdtlData};
use crate::chunk::info::{ListInfo, ListInfoData};
use crate::chunk::wavl::{ListWavl, ListWavlData};
use crate::chunk::Bext;
use crate::chunk::Cset;
use crate::chunk::Cue;
use crate::chunk::Data;
use crate::chunk::Fact;
use crate::chunk::Fllr;
use crate::chunk::Fmt;
use crate::chunk::Junk;
use crate::chunk::Md5;
use crate::chunk::Pad;
use crate::chunk::Plst;
use crate::chunk::Riff;
pub mod fixedstring;
pub mod testing;

// helper types
// ----

// Since const generics do not support arrays, generic structs are storing the
// FourCC id as a `u32`... which makes instantiation awkward. This is a helper
// function to make it a bit easier.
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
    /// Returns the logical (used) size in bytes of the chunk.
    fn size(&self) -> u32;
}

/// Utility methods for describing any chunk.
pub trait Summarizable: ChunkID {
    /// Returns a short text summary of the contents of the chunk.
    fn summary(&self) -> String;

    /// User friendly name of the chunk, usually the chunk id
    ///
    /// An ascii friendly chunk name, with whitespace removed. Chunks with
    /// sub types (forms, in RIFF terms) include their sub type in the name.
    /// Ex: [`ListInfo`] is "LIST-INFO".    
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

/// A chunk with size information.
///
/// We don't actually have these yet, but expect to use them when creating chunks
/// from scratch and later writing them to a file.
// pub trait Chunk: Summarizable + Debug {}

impl<T> ChunkID for T
where
    T: KnownChunkID,
{
    fn id(&self) -> FourCC {
        T::ID
    }
}

// Helper for cleaner Result.map() calls when boxing inner chunk.
// Reduces code repetition, but mostly helps the compiler with type
// inference.
fn box_chunk<T: SizedChunk + 'static>(t: T) -> Box<dyn SizedChunk> {
    Box::new(t)
}

fn box_chunk_res<T>(result: Result<KnownChunk<T>, binrw::Error>) -> Box<dyn SizedChunk>
where
    T: for<'a> BinRead<Args<'a> = KCArgs>
        + for<'a> BinWrite<Args<'a> = ()>
        + KnownChunkID
        + Summarizable
        + Debug
        + 'static,
{
    match result {
        Ok(chunk) => Box::new(chunk),
        Err(err) => Box::new(ErrorChunk::from(err)),
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
///
/// Work In Progress
/// cases to handle:
/// Unexpected `FourCC`
/// Bin Read Error, Bin Error, Parse Error, Deserialize Error, Chunk Read Error
#[derive(Debug)]
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

    /// temp placeholder: TODO: delete me
    Other(Box<dyn error::Error + Send + Sync>),
}

impl error::Error for WaveError {}

impl Display for WaveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            WaveError::UnknownFourCC { message, .. } => write!(f, "UnknownFourCC: {}", message),
            WaveError::Io(err) => write!(f, "Io: {}", err),
            WaveError::Parse { message, .. } => write!(f, "Parse: {}", message),
            WaveError::Other(err) => write!(f, "{}", err),
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

/// Error wrapper satisfying chunk traits
#[derive(Debug)]
pub struct ErrorChunk {
    error: WaveError,
}

impl ErrorChunk {
    fn new(error: WaveError) -> Self {
        ErrorChunk { error }
    }
}

impl Display for ErrorChunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "ErrorChunk({}, {})", self.id(), self.error)
    }
}

impl ChunkID for ErrorChunk {
    fn id(&self) -> FourCC {
        FourCC(*b"ERR:")
    }
}

impl SizedChunk for ErrorChunk {
    // TODO: consider this... ErrorChunk isn't actually a sized chunk,
    // since we don't know its size... bleh.
    fn size(&self) -> u32 {
        0
    }
}

impl Summarizable for ErrorChunk {
    fn summary(&self) -> String {
        self.error.to_string()
    }
}

impl From<binrw::Error> for ErrorChunk {
    fn from(err: binrw::Error) -> Self {
        Self::new(WaveError::from(err))
    }
}

// TODO: test this... use it from the cli
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
    fn handle(&mut self, err: WaveError) -> Option<Box<dyn SizedChunk>> {
        self.finished = true;
        Some(Box::new(ErrorChunk::new(err)))
    }
}

impl<'a, R> Iterator for WaveIterator<'a, R>
where
    R: Read + Seek + Debug + BufRead,
{
    type Item = Box<dyn SizedChunk>;

    #[instrument]
    fn next(&mut self) -> Option<Self::Item> {
        // errors must call self.handle() before returning to avoid infinite iteration
        // instead, what if we refactored to have a helper function which returned result
        // io::Error, and we handle turning that into ErrorChunk here?
        // then we could use ? for all the io::Error issues below.
        if self.finished {
            return None;
        }

        let mut offset = match self.reader.stream_position() {
            Ok(v) => v,
            Err(err) => {
                return self.handle(WaveError::Parse {
                    pos: None,
                    message: err.to_string(),
                })
            }
        };

        let mut buff: [u8; 4] = [0; 4];
        let chunk_id: [u8; 4] = match self.reader.read_exact(&mut buff) {
            Ok(_) => buff,
            Err(err) => {
                return self.handle(WaveError::Parse {
                    pos: Some(offset),
                    message: format!("Io: {}", err),
                });
            }
        };
        let chunk_size = match self.reader.read_exact(&mut buff) {
            Ok(_) => u32::from_le_bytes(buff),
            Err(err) => {
                return self.handle(WaveError::Parse {
                    pos: Some(offset + 4),
                    message: format!("Io: {}", err),
                });
            }
        };

        match self.reader.seek(SeekFrom::Current(-8)) {
            Ok(_) => (),
            Err(err) => {
                return self.handle(WaveError::Parse {
                    pos: Some(offset),
                    message: format!("Io: {}", err),
                });
            }
        }

        let id = FourCC(chunk_id);
        let chunk = match id {
            Fmt::ID => box_chunk_res(Fmt::read(&mut self.reader)),
            Data::ID => box_chunk_res(Data::read(&mut self.reader)),
            Fact::ID => box_chunk_res(Fact::read(&mut self.reader)),
            ListInfo::ID => {
                let list = match Riff::read(&mut self.reader) {
                    Ok(v) => v,
                    Err(err) => {
                        return self.handle(WaveError::Parse {
                            pos: Some(offset),
                            message: err.to_string(),
                        })
                    }
                };
                match self.reader.seek(SeekFrom::Current(-12)) {
                    Ok(_) => (),
                    Err(err) => {
                        return self.handle(WaveError::Parse {
                            pos: Some(offset),
                            message: format!("Io: {}", err),
                        });
                    }
                }
                match list.form_type {
                    ListInfoData::LIST_TYPE => box_chunk_res(ListInfo::read(&mut self.reader)),
                    ListAdtlData::LIST_TYPE => box_chunk_res(ListAdtl::read(&mut self.reader)),
                    ListWavlData::LIST_TYPE => box_chunk_res(ListWavl::read(&mut self.reader)),
                    _ => match UnknownChunk::read(&mut self.reader) {
                        Ok(chunk) => box_chunk(chunk),
                        Err(err) => Box::new(ErrorChunk::from(err)),
                    },
                }
            }
            Cue::ID => box_chunk_res(Cue::read(&mut self.reader)),
            Cset::ID => box_chunk_res(Cset::read(&mut self.reader)),
            Plst::ID => box_chunk_res(Plst::read(&mut self.reader)),
            Bext::ID => box_chunk_res(Bext::read(&mut self.reader)),
            Md5::ID => box_chunk_res(Md5::read(&mut self.reader)),
            Junk::ID => box_chunk_res(Junk::read(&mut self.reader)),
            Pad::ID => box_chunk_res(Pad::read(&mut self.reader)),
            Fllr::ID => box_chunk_res(Fllr::read(&mut self.reader)),
            _ => match UnknownChunk::read(&mut self.reader) {
                Ok(chunk) => box_chunk(chunk),
                Err(err) => Box::new(ErrorChunk::from(err)),
            },
        };

        // setup for next iteration
        offset += chunk_size as u64 + 8;
        // RIFF offsets must be on word boundaries (divisible by 2)
        if offset % 2 == 1 {
            offset += 1;
        };

        // returns after parsing a chunk would cause a missing chunk
        // this logic needs refactoring for an iterator, I guess. :/
        let stream_position = match self.reader.stream_position() {
            Ok(v) => v,
            Err(err) => {
                return self.handle(WaveError::Parse {
                    pos: Some(offset),
                    message: format!("Io: {}", err),
                });
            }
        };
        if offset != stream_position {
            // TODO: inject error into chunk vec and remove print
            // oh dang, this is tricky. We actually successfully (probably)
            // parsed the chunk, but there is still an error.
            // I guess we could add a syntetic error chunk as well... but...
            // should we have a mechanism for annotating chunks with
            // warnings and notes from the parsers?
            warn!("{:?}: parsed less data than chunk size", FourCC(chunk_id));
            match self.reader.seek(SeekFrom::Start(offset)) {
                Ok(_) => (),
                Err(err) => {
                    return self.handle(WaveError::Parse {
                        pos: Some(offset),
                        message: format!("Io: {}", err),
                    });
                }
            }
        }

        // TODO: is this correct? Should it be > instead of >=  ??
        if offset >= self.riff_size as u64 {
            return None;
        };
        Some(chunk)
    }
}

/// Wrapper around RIFF-WAVE data.
pub struct Wave<R>
where
    R: Read + Seek + Debug + BufRead,
{
    bytes: R,
    riff: Riff,
}

impl<R> Wave<R>
where
    R: Read + Seek + Debug + BufRead,
{
    /// Create a new Wave handle. This keeps a reference to the data
    /// until dropped.
    // TODO: consider renaming to from_reader/from_bufreader to avoid changing interface
    // when later adding write support
    pub fn new(mut reader: R) -> Result<Self, WaveError> {
        let riff = Riff::read(&mut reader).map_err(std::io::Error::other)?;
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

    #[instrument]
    pub fn iter_chunks<'a>(&'a mut self) -> WaveIterator<'a, R> {
        tracing::info!("hello, from iter_chunks()");
        WaveIterator {
            reader: &mut self.bytes,
            riff_size: self.riff.size,
            finished: false,
        }
    }

    // TODO: rename: chunks(), read_chunks()?

    /// Parses WAV (RIFF-WAVE) data, returns all known chunks.
    ///
    /// It attempts to continue parsing even if some chunks have parsing errors.
    /// In some cases, it may return before reading all chunks, such when
    /// an error results from parsing the RIFF container, or the data is
    /// not a WAVE form type.
    #[instrument]
    pub fn metadata_chunks(
        &mut self,
    ) -> Result<Vec<Result<Box<dyn SizedChunk>, WaveError>>, WaveError> {
        let mut reader = &mut self.bytes;

        let mut buff: [u8; 4] = [0; 4];
        let mut offset = reader.stream_position()?;
        let mut chunks: Vec<Result<Box<dyn SizedChunk>, WaveError>> = Vec::new();

        loop {
            let _span_ = trace_span!("metadata_chunks_loop").entered();

            let chunk_id: [u8; 4] = match reader.read_exact(&mut buff) {
                Ok(_) => buff,
                Err(err) => {
                    chunks.push(Err(WaveError::Parse {
                        pos: Some(offset),
                        message: format!("Io: {}", err),
                    }));
                    return Ok(chunks);
                }
            };
            let chunk_size = {
                reader.read_exact(&mut buff)?;
                u32::from_le_bytes(buff)
            };

            reader.seek(SeekFrom::Current(-8))?;
            let id = FourCC(chunk_id);
            let res = match id {
                Fmt::ID => Fmt::read(&mut reader).map(box_chunk),
                Data::ID => Data::read(&mut reader).map(box_chunk),
                Fact::ID => Fact::read(&mut reader).map(box_chunk),
                ListInfo::ID => {
                    let list = Riff::read(&mut reader)?;
                    reader.seek(SeekFrom::Current(-12))?;
                    match list.form_type {
                        ListInfoData::LIST_TYPE => ListInfo::read(&mut reader).map(box_chunk),
                        ListAdtlData::LIST_TYPE => ListAdtl::read(&mut reader).map(box_chunk),
                        ListWavlData::LIST_TYPE => ListWavl::read(&mut reader).map(box_chunk),
                        _ => UnknownChunk::read(&mut reader).map(box_chunk),
                    }
                }
                Cue::ID => Cue::read(&mut reader).map(box_chunk),
                Cset::ID => Cset::read(&mut reader).map(box_chunk),
                Plst::ID => Plst::read(&mut reader).map(box_chunk),
                Bext::ID => Bext::read(&mut reader).map(box_chunk),
                Md5::ID => Md5::read(&mut reader).map(box_chunk),
                Junk::ID => Junk::read(&mut reader).map(box_chunk),
                Pad::ID => Pad::read(&mut reader).map(box_chunk),
                Fllr::ID => Fllr::read(&mut reader).map(box_chunk),
                _ => UnknownChunk::read(&mut reader).map(box_chunk),
            };
            chunks.push(res.map_err(WaveError::from));

            // setup for next iteration
            offset += chunk_size as u64 + 8;
            // RIFF offsets must be on word boundaries (divisible by 2)
            if offset % 2 == 1 {
                offset += 1;
            };
            if offset != reader.stream_position()? {
                // TODO: inject error into chunk vec and remove print
                warn!("{:?}: parsed less data than chunk size", FourCC(chunk_id));
                reader.seek(SeekFrom::Start(offset))?;
            }

            if offset >= self.riff.size as u64 {
                break;
            };
        }
        Ok(chunks)
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KnownChunk<
    T: for<'a> BinRead<Args<'a> = KCArgs> + for<'a> BinWrite<Args<'a> = ()> + KnownChunkID,
> {
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnknownChunk {
    id: FourCC,
    size: u32,
    #[brw(align_after = 2)]
    #[br(count = size )]
    raw: Vec<u8>,
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
    Fmt(Fmt),
    Data(Data),
    Fact(Fact),
    Cue(Cue),
    Info(ListInfo),
    Adtl(ListAdtl),
    Wavl(ListWavl),
    Cset(Cset),
    Plst(Plst),
    Bext(Box<Bext>),
    Md5(Md5),
    Fllr(Fllr),
    Junk(Junk),
    Pad(Pad),
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
        let md5 = Md5 {
            size: 16,
            data: chunk::Md5Data { md5: 0 },
            extra_bytes: vec![],
        };
        // ensure trait bounds are satisfied
        let mut _trt: Box<dyn SizedChunk> = Box::new(md5);
    }

    #[test]
    fn chunkenum_as_trait() {
        let md5 = SizedChunkEnum::Md5(Md5 {
            size: 16,
            data: chunk::Md5Data { md5: 0 },
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
        has_standard_traits::<Riff>();

        // this Enum transitively ensures the traits of all subchunks
        has_standard_traits::<SizedChunkEnum>();
    }
}
