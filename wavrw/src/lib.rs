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
        f.debug_tuple("FourCC")
            .field(&format!("*b{}", &self.to_string()))
            .finish()
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
            SizedChunkEnum::Info(e) => Box::new(e.items()),
            SizedChunkEnum::Bext(e) => Box::new(e.items()),
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
    fn fourcc() {
        let f = FourCC(*b"TST ");

        println!("Display: {f}");
        assert_eq!(f.to_string(), "TST ");
        println!("Debug: {f:?}");
        assert_eq!(format!("{f:?}"), r#"FourCC("*bTST ")"#);
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
