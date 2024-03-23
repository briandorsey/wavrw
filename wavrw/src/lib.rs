#![doc = include_str!("lib.md")]

extern crate alloc;

use core::fmt::{Debug, Display, Formatter};

use binrw::io::BufReader;
use binrw::io::TakeSeekExt;
use binrw::io::{Read, Seek};
use binrw::{binrw, io::SeekFrom, BinRead, BinResult, BinWrite, PosValue};
use tracing::{instrument, trace_span, warn};

pub mod chunk;
use crate::chunk::adtl::{ListAdtl, ListAdtlData};
use crate::chunk::info::{ListInfo, ListInfoData};
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
pub mod fixedstr;
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
    /// Return RIFF chunk id.
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

    /// Returns an iterator over a sequence of contents of the contained
    /// chunk as strings (field, value).
    fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        Box::new(core::iter::empty())
    }

    /// Alternative header for use above `items`
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
        write!(f, "FourCC({}=", String::from_utf8_lossy(&self.0),)?;
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

/// Parses a WAV file, returns all known chunks.
///
/// It attempts to continue parsing even if some chunks have parsing errors.
#[instrument]
pub fn metadata_chunks<R>(reader: R) -> Result<Vec<BinResult<Box<dyn SizedChunk>>>, std::io::Error>
where
    R: Read + Seek + Debug,
{
    let mut reader = BufReader::new(reader);

    // TODO: research errors and figure out an error plan for wavrw
    // remove wrapping Result, and map IO and BinErrors to wavrw errors
    let riff = Riff::read(&mut reader).map_err(std::io::Error::other)?;
    // TODO: convert this temp error into returned wavrw error type
    if riff.form_type != FourCC(*b"WAVE") {
        return Err(std::io::Error::other(format!(
            "not a wave file. Expected RIFF form_type 'WAVE', found: {}",
            riff.form_type
        )));
    }

    let mut buff: [u8; 4] = [0; 4];
    let mut offset = reader.stream_position()?;
    let mut chunks: Vec<BinResult<Box<dyn SizedChunk>>> = Vec::new();

    loop {
        let _span_ = trace_span!("metadata_chunks_loop").entered();
        let chunk_id = {
            reader.read_exact(&mut buff)?;
            buff
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
                let list = Riff::read(&mut reader).map_err(std::io::Error::other)?;
                reader.seek(SeekFrom::Current(-12))?;
                match list.form_type {
                    ListInfoData::LIST_TYPE => ListInfo::read(&mut reader).map(box_chunk),
                    ListAdtlData::LIST_TYPE => ListAdtl::read(&mut reader).map(box_chunk),
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
        chunks.push(res);

        // setup for next iteration
        offset += chunk_size as u64 + 8;
        // RIFF offsets must be on word boundaries (divisible by 2)
        if offset % 2 == 1 {
            offset += 1;
        };
        if offset != reader.stream_position()? {
            // TODO: inject error into chunk vec and remove print
            warn!("{}: parsed less data than chunk size", FourCC(chunk_id));
            reader.seek(SeekFrom::Start(offset))?;
        }

        if offset >= riff.size as u64 {
            break;
        };
    }
    Ok(chunks)
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
    #[br(temp, assert(id == T::ID))]
    #[bw(calc = T::ID)]
    pub id: FourCC,

    // TODO: calc by querying content + extra_bytes.len() when writing, or seeking back after you know
    pub size: u32,

    #[br(temp)]
    #[bw(ignore)]
    begin_pos: PosValue<()>,
    // ensure that we don't read outside the bounds for this chunk
    #[br(map_stream = |r| r.take_seek(size as u64), args(size))]
    pub data: T,

    // assert for better error message if too many bytes processed
    // seems like it should be impossible, but found a case where T
    // used align_after to pad bytes.
    #[br(temp, assert((end_pos.pos - begin_pos.pos) <= size as u64, "(end_pos.pos - begin_pos.pos) <= size while parsing {}. end: {}, begin: {}, size: {}", T::ID, end_pos.pos, begin_pos.pos, size))]
    #[bw(ignore)]
    end_pos: PosValue<()>,

    // calculate how much was read, and read any extra bytes that remain in the chunk
    #[br(align_after = 2, count = size as u64 - (end_pos.pos - begin_pos.pos))]
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

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnknownChunk {
    id: FourCC,
    size: u32,
    #[br(align_after = 2, count = size )]
    #[bw(align_after = 2)]
    raw: Vec<u8>,
}

impl Display for UnknownChunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "UnknownChunk({}, {})", self.id, self.size)
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

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChunkEnum {
    Fmt(Fmt),
    Data(Data),
    Fact(Fact),
    Cue(Cue),
    Info(ListInfo),
    Adtl(ListAdtl),
    Cset(Cset),
    Plst(Plst),
    Bext(Box<Bext>),
    Md5(Md5),
    Fllr(Fllr),
    Junk(Junk),
    Pad(Pad),
    Unknown(UnknownChunk),
}

impl Display for ChunkEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let display_string = match self {
            ChunkEnum::Fmt(e) => e.to_string(),
            ChunkEnum::Data(e) => e.to_string(),
            ChunkEnum::Fact(e) => e.to_string(),
            ChunkEnum::Cue(e) => e.to_string(),
            ChunkEnum::Info(e) => e.to_string(),
            ChunkEnum::Adtl(e) => e.to_string(),
            ChunkEnum::Cset(e) => e.to_string(),
            ChunkEnum::Plst(e) => e.to_string(),
            ChunkEnum::Bext(e) => e.to_string(),
            ChunkEnum::Md5(e) => e.to_string(),
            ChunkEnum::Fllr(e) => e.to_string(),
            ChunkEnum::Junk(e) => e.to_string(),
            ChunkEnum::Pad(e) => e.to_string(),
            ChunkEnum::Unknown(e) => e.to_string(),
        };
        write!(f, "{}", display_string)
    }
}

impl ChunkID for ChunkEnum {
    /// Returns the `FourCC` (chunk id) for the contained chunk.
    fn id(&self) -> FourCC {
        match self {
            ChunkEnum::Fmt(e) => e.id(),
            ChunkEnum::Data(e) => e.id(),
            ChunkEnum::Fact(e) => e.id(),
            ChunkEnum::Cue(e) => e.id(),
            ChunkEnum::Info(e) => e.id(),
            ChunkEnum::Adtl(e) => e.id(),
            ChunkEnum::Cset(e) => e.id(),
            ChunkEnum::Plst(e) => e.id(),
            ChunkEnum::Bext(e) => e.id(),
            ChunkEnum::Md5(e) => e.id(),
            ChunkEnum::Fllr(e) => e.id(),
            ChunkEnum::Junk(e) => e.id(),
            ChunkEnum::Pad(e) => e.id(),
            ChunkEnum::Unknown(e) => e.id(),
        }
    }
}

impl SizedChunk for ChunkEnum {
    /// Returns the logical (used) size in bytes of the contained chunk.
    fn size(&self) -> u32 {
        match self {
            ChunkEnum::Fmt(e) => e.size,
            ChunkEnum::Data(e) => e.size,
            ChunkEnum::Fact(e) => e.size,
            ChunkEnum::Cue(e) => e.size,
            ChunkEnum::Info(e) => e.size,
            ChunkEnum::Adtl(e) => e.size,
            ChunkEnum::Cset(e) => e.size,
            ChunkEnum::Plst(e) => e.size,
            ChunkEnum::Bext(e) => e.size,
            ChunkEnum::Md5(e) => e.size,
            ChunkEnum::Fllr(e) => e.size,
            ChunkEnum::Junk(e) => e.size,
            ChunkEnum::Pad(e) => e.size,
            ChunkEnum::Unknown(e) => e.size,
        }
    }
}

impl Summarizable for ChunkEnum {
    /// Returns a short text summary of the contents of the contained chunk.
    fn summary(&self) -> String {
        match self {
            ChunkEnum::Fmt(e) => e.summary(),
            ChunkEnum::Data(e) => e.summary(),
            ChunkEnum::Fact(e) => e.summary(),
            ChunkEnum::Cue(e) => e.summary(),
            ChunkEnum::Info(e) => e.summary(),
            ChunkEnum::Adtl(e) => e.summary(),
            ChunkEnum::Cset(e) => e.summary(),
            ChunkEnum::Plst(e) => e.summary(),
            ChunkEnum::Bext(e) => e.summary(),
            ChunkEnum::Md5(e) => e.summary(),
            ChunkEnum::Fllr(e) => e.summary(),
            ChunkEnum::Junk(e) => e.summary(),
            ChunkEnum::Pad(e) => e.summary(),
            ChunkEnum::Unknown(e) => e.summary(),
        }
    }
    /// Returns an iterator over a sequence of contents of the contained
    /// chunk as strings (field, value).
    fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        match self {
            ChunkEnum::Fmt(e) => Box::new(e.items()),
            ChunkEnum::Info(e) => Box::new(e.items()),
            ChunkEnum::Bext(e) => Box::new(e.items()),
            _ => Box::new(core::iter::empty()),
        }
    }
}

// impl Chunk for ChunkEnum {}

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {

    use super::*;

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
        let md5 = ChunkEnum::Md5(Md5 {
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
        has_standard_traits::<ChunkEnum>();
    }
}
