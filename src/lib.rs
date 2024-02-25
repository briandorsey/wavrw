//! `wavrw` provides tools for reading (and someday writing) wave audio  file
//! chunks with a focus on metadata.

use std::cmp::min;
use std::fmt::{Debug, Display, Formatter};
use std::io::BufReader;
use std::io::{Read, Seek};
use std::str::FromStr;

use binrw::io::TakeSeekExt;
use binrw::Endian;
use binrw::{binrw, io::SeekFrom, BinRead, BinResult, BinWrite, Error, PosValue};
use tracing::{instrument, trace_span, warn};

pub mod chunk;
pub mod testing;
use crate::chunk::Bext;
use crate::chunk::Cset;
use crate::chunk::Data;
use crate::chunk::Fact;
use crate::chunk::Fmt;
use crate::chunk::Md5;
use crate::chunk::Riff;
use crate::chunk::{ListAdtl, ListAdtlData};
use crate::chunk::{ListInfo, ListInfoData};

// helper types
// ----

// Since const generics do not support arrays, generic structs are storing the
// FourCC id as a `u32`... which makes instantiation awkward. This is a helper
// function to make it a bit easier.
#[doc(hidden)]
pub const fn fourcc(id: &[u8; 4]) -> u32 {
    u32::from_le_bytes(*id)
}

pub trait KnownChunkID {
    const ID: FourCC;
}

pub trait ChunkID {
    fn id(&self) -> FourCC;
}

pub trait SizedChunk: ChunkID {
    /// Returns the logical (used) size in bytes of the chunk.
    fn size(&self) -> u32;
}

pub trait Summarizable: ChunkID {
    /// Returns a short text summary of the contents of the chunk.
    fn summary(&self) -> String;

    /// User friendly name of the chunk, usually the chunk id
    ///
    /// An ascii friendly chunk name, with whitespace removed. Chunks with
    /// sub types (forms, in RIFF terms) use their sub type as the name.
    /// Ex: `ListInfoChunk` is "INFO".    
    fn name(&self) -> String {
        self.id().to_string().trim().to_string()
    }

    /// Returns an iterator over a sequence of contents of the contained
    /// chunk as strings (field, value).
    fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        Box::new(std::iter::empty())
    }
}

pub trait Chunk: SizedChunk + Summarizable + Debug {}

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
fn box_chunk<T: Chunk + 'static>(t: T) -> Box<dyn Chunk> {
    Box::new(t)
}

#[binrw]
#[brw(big)]
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct FourCC(pub [u8; 4]);

impl Display for FourCC {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", String::from_utf8_lossy(&self.0),)?;
        Ok(())
    }
}

impl Debug for FourCC {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
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

#[derive(Debug)]
pub struct FixedStrErr;

#[derive(BinWrite, PartialEq, Eq)]
/// `FixedStr` holds Null terminated fixed length strings (from BEXT for example)
///
/// `FixedStr` is intended to be used via binrw's [`BinRead`] trait and its
/// Null parsing is implmented there. Do not directly create the struct
/// or that logic will be bypassed. If there is a future need, we should
/// implement a constructor which in turn calls the [`FixedStr::read_options()`]
/// implementation.
pub struct FixedStr<const N: usize>([u8; N]);

// FixedStr display design question. RIFF spec uses ""Z notation for fixed strings. Should we do the same?

impl<const N: usize> Debug for FixedStr<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "FixedStr<{}>(\"{}\")", N, self)
    }
}

impl<const N: usize> Display for FixedStr<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            String::from_utf8_lossy(&self.0).trim_end_matches('\0')
        )
    }
}

impl<const N: usize> FromStr for FixedStr<N> {
    type Err = FixedStrErr;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        let mut array_tmp = [0u8; N];
        let l = min(s.len(), N);
        array_tmp[..l].copy_from_slice(&s.as_bytes()[..l]);
        Ok(FixedStr::<N>(array_tmp))
    }
}

impl<const N: usize> BinRead for FixedStr<N> {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut values: [u8; N] = [0; N];
        let mut index = 0;

        loop {
            if index >= N {
                return Ok(Self(values));
            }
            let val = <u8>::read_options(reader, endian, ())?;
            if val == 0 {
                let offset = (N - index - 1).try_into();
                return match offset {
                    Ok(offset) => {
                        reader.seek(SeekFrom::Current(offset))?;
                        Ok(Self(values))
                    }
                    Err(err) => Err(Error::Custom {
                        pos: index as u64,
                        err: Box::new(err),
                    }),
                };
            }
            values[index] = val;
            index += 1;
        }
    }
}

// parsing helpers
// ----

/// `metadata_chunks` parses a WAV file chunk by chunk, continuing
///  even if some chunks have parsing errors.
#[instrument]
pub fn metadata_chunks<R>(reader: R) -> Result<Vec<BinResult<Box<dyn Chunk>>>, std::io::Error>
where
    R: Read + Seek + Debug,
{
    let mut reader = BufReader::new(reader);

    // TODO: research errors and figure out an error plan for wavrw
    // remove wrapping Result, and map IO and BinErrors to wavrw errors
    let riff = Riff::read(&mut reader).map_err(std::io::Error::other)?;
    // TODO: convert assert into returned wav error type
    assert_eq!(
        riff.form_type,
        FourCC(*b"WAVE"),
        "{} != WAVE",
        riff.form_type
    );

    let mut buff: [u8; 4] = [0; 4];
    let mut offset = reader.stream_position()?;
    let mut chunks: Vec<BinResult<Box<dyn Chunk>>> = Vec::new();

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
            Cset::ID => Cset::read(&mut reader).map(box_chunk),
            Bext::ID => Bext::read(&mut reader).map(box_chunk),
            Md5::ID => Md5::read(&mut reader).map(box_chunk),
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

#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
/// `KnownChunk` is a wrapper around chunk data, handling ID and size.
pub struct KnownChunk<
    T: for<'a> BinRead<Args<'a> = ()> + for<'a> BinWrite<Args<'a> = ()> + KnownChunkID,
> {
    #[br(temp, assert(id == T::ID))]
    #[bw(calc = T::ID)]
    id: FourCC,

    // TODO: calc by querying content + extra_bytes.len() when writing, or seeking back after you know
    size: u32,

    #[br(temp)]
    #[bw(ignore)]
    begin_pos: PosValue<()>,
    // ensure that we don't read outside the bounds for this chunk
    #[br(map_stream = |r| r.take_seek(size as u64))]
    data: T,

    // assert for better error message if too many bytes processed
    // seems like it should be impossible, but found a case where T
    // used align_after to pad bytes.
    #[br(temp, assert((end_pos.pos - begin_pos.pos) <= size as u64, "(end_pos.pos - begin_pos.pos) <= size while parsing {}. end: {}, begin: {}, size: {}", T::ID, end_pos.pos, begin_pos.pos, size))]
    #[bw(ignore)]
    end_pos: PosValue<()>,

    // calculate how much was read, and read any extra bytes that remain in the chunk
    #[br(align_after = 2, count = size as u64 - (end_pos.pos - begin_pos.pos))]
    extra_bytes: Vec<u8>,
}

impl<T> KnownChunkID for KnownChunk<T>
where
    T: for<'a> BinRead<Args<'a> = ()> + for<'a> BinWrite<Args<'a> = ()> + KnownChunkID,
{
    const ID: FourCC = T::ID;
}

impl<T> SizedChunk for KnownChunk<T>
where
    T: for<'a> BinRead<Args<'a> = ()> + for<'a> BinWrite<Args<'a> = ()> + KnownChunkID,
{
    fn size(&self) -> u32 {
        self.size + self.extra_bytes.len() as u32
    }
}

impl<T> Summarizable for KnownChunk<T>
where
    T: for<'a> BinRead<Args<'a> = ()>
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
}

impl<T> Chunk for KnownChunk<T> where
    T: for<'a> BinRead<Args<'a> = ()>
        + for<'a> BinWrite<Args<'a> = ()>
        + KnownChunkID
        + Summarizable
        + Debug
{
}

#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
pub struct UnknownChunk {
    id: FourCC,
    size: u32,
    #[br(align_after = 2, count = size )]
    #[bw(align_after = 2)]
    raw: Vec<u8>,
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

impl Chunk for UnknownChunk {}

#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
pub enum ChunkEnum {
    Fmt(Fmt),
    Data(Data),
    Fact(Fact),
    Info(ListInfo),
    Adtl(ListAdtl),
    Cset(Cset),
    Bext(Box<Bext>),
    Md5(Md5),
    Unknown(UnknownChunk),
}

impl ChunkID for ChunkEnum {
    /// Returns the `FourCC` (chunk id) for the contained chunk.
    fn id(&self) -> FourCC {
        match self {
            ChunkEnum::Fmt(e) => e.id(),
            ChunkEnum::Data(e) => e.id(),
            ChunkEnum::Fact(e) => e.id(),
            ChunkEnum::Info(e) => e.id(),
            ChunkEnum::Adtl(e) => e.id(),
            ChunkEnum::Cset(e) => e.id(),
            ChunkEnum::Bext(e) => e.id(),
            ChunkEnum::Md5(e) => e.id(),
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
            ChunkEnum::Info(e) => e.size,
            ChunkEnum::Adtl(e) => e.size,
            ChunkEnum::Cset(e) => e.size,
            ChunkEnum::Bext(e) => e.size,
            ChunkEnum::Md5(e) => e.size,
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
            ChunkEnum::Info(e) => e.summary(),
            ChunkEnum::Adtl(e) => e.summary(),
            ChunkEnum::Cset(e) => e.summary(),
            ChunkEnum::Bext(e) => e.summary(),
            ChunkEnum::Md5(e) => e.summary(),
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
            _ => Box::new(std::iter::empty()),
        }
    }
}

impl Chunk for ChunkEnum {}

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {

    use super::*;
    use crate::testing::hex_to_cursor;

    #[test]
    fn fixed_string() {
        let fs = FixedStr::<6>(*b"abc\0\0\0");
        assert_eq!(6, fs.0.len());
        let s = fs.to_string();
        assert_eq!("abc".to_string(), s);
        assert_eq!(3, s.len());
        let new_fs = FixedStr::<6>::from_str(&s).unwrap();
        assert_eq!(fs, new_fs);
        assert_eq!(6, fs.0.len());
        assert_eq!(
            b"\0\0\0"[..],
            new_fs.0[3..6],
            "extra space after the string should be null bytes"
        );

        // strings longer than fixed size should get truncated
        let long_str = "this is a longer str";
        let fs = FixedStr::<6>::from_str(long_str).unwrap();
        assert_eq!(fs.to_string(), "this i");
    }

    #[test]
    fn parse_fixedstr_data_after_zero() {
        // REAPER had (still has?) a bug where data from other BEXT fields
        // can be left in a fixed lenth string field after the terminating
        // zero byte. This input is an example that starts with "REAPER"
        // but has part of a path string after the terminating zero.
        let mut buff = hex_to_cursor(
            "52454150 45520065 72732F62 7269616E 2F70726F 6A656374 732F7761 7672772F",
        );
        let fs = FixedStr::<32>::read_options(&mut buff, binrw::Endian::Big, ())
            .expect("error parsing FixedStr");
        assert_eq!(fs, FixedStr::<32>::from_str("REAPER").unwrap());
    }

    #[test]
    fn knownchunk_as_trait() {
        let md5 = Md5 {
            size: 16,
            data: chunk::Md5Data { md5: 0 },
            extra_bytes: vec![],
        };
        // ensure trait bounds are satisfied
        let mut _trt: Box<dyn Chunk> = Box::new(md5);
    }

    #[test]
    fn chunkenum_as_trait() {
        let md5 = ChunkEnum::Md5(Md5 {
            size: 16,
            data: chunk::Md5Data { md5: 0 },
            extra_bytes: vec![],
        });
        // ensure trait bounds are satisfied
        let mut _trt: Box<dyn Chunk> = Box::new(md5);
    }
}
