//! `wavl` A `LIST` containing audio and/or silence chunks: data, slnt. Very rare. [RIFF1991](https://wavref.til.cafe/chunk/wavl/)
//!
//! NOTE: Implemented from the spec only, because I couldn't find any files actually
//! containing this chunk.

use core::fmt::Debug;

use binrw::{binrw, helpers};
use itertools::Itertools;

use crate::chunk::data::DataChunk;
use crate::{ChunkID, FourCC, KnownChunk, KnownChunkID, Summarizable};

#[binrw]
#[br(little)]
#[br(import(_size: u32))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// `LIST-wavl` contains a sequence of [`DataChunk`] or [`SlntChunk`] chunks.
pub struct ListWavl {
    /// A four-character code that identifies the contents of the list.
    #[brw(assert(list_type == ListWavl::LIST_TYPE))]
    pub list_type: FourCC,

    /// Sub chunks contained within this LIST
    #[br(parse_with = helpers::until_eof)]
    #[bw()]
    pub chunks: Vec<WavlEnum>,
}

impl ListWavl {
    /// Chunk id constant: `wavl`
    pub const LIST_TYPE: FourCC = FourCC(*b"wavl");
}

impl KnownChunkID for ListWavl {
    const ID: FourCC = FourCC(*b"LIST");
}

impl Summarizable for ListWavl {
    fn summary(&self) -> String {
        self.chunks
            .iter()
            .into_grouping_map_by(|c| c.id())
            .fold(0, |acc, _key, _value| acc + 1)
            .iter()
            .map(|(g, c)| format!("{}({})", g, c))
            .sorted_unstable()
            .join(", ")
    }

    fn name(&self) -> String {
        format!("{}-{}", self.id(), self.list_type)
    }

    fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        Box::new(
            self.chunks
                .iter()
                .map(|c| (c.id().to_string(), c.summary())),
        )
    }
}

/// `LIST-wavl` contains a sequence of [`DataChunk`] or [`SlntChunk`] chunks.
///
/// NOTE: Implemented from the spec only, because I couldn't find any files actually
/// containing this chunk.
pub type ListWavlChunk = KnownChunk<ListWavl>;

#[binrw]
#[br(little)]
#[br(import(_size: u32))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// ‘slnt’ represents silence, not necessarily a repeated zero volume.
pub struct Slnt {
    /// Number of samples of silence in this chunk.
    pub samples: u32,
}

impl KnownChunkID for Slnt {
    const ID: FourCC = FourCC(*b"slnt");
}

impl Summarizable for Slnt {
    fn summary(&self) -> String {
        format!("{} samples", self.samples)
    }
}

/// ‘slnt’ represents silence, not necessarily a repeated zero volume.
pub type SlntChunk = KnownChunk<Slnt>;

/// All `LIST-wavl` chunk structs as an enum
#[allow(missing_docs)]
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WavlEnum {
    Data(DataChunk),
    Slnt(SlntChunk),
    Unknown {
        id: FourCC,
        size: u32,
        #[brw(align_after = 2)]
        #[br(count = size, pad_size_to= size.to_owned())]
        raw: Vec<u8>,
    },
}

impl ChunkID for WavlEnum {
    fn id(&self) -> FourCC {
        match self {
            WavlEnum::Data(e) => e.id(),
            WavlEnum::Slnt(e) => e.id(),
            WavlEnum::Unknown { id, .. } => *id,
        }
    }
}

impl Summarizable for WavlEnum {
    fn summary(&self) -> String {
        match self {
            WavlEnum::Data(e) => e.summary(),
            WavlEnum::Slnt(e) => e.summary(),
            WavlEnum::Unknown { .. } => "...".to_string(),
        }
    }
}

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {
    use binrw::{BinRead, BinWrite};
    use hexdump::hexdump;

    use super::*;
    use crate::chunk::data::Data;

    // couldn't find slnt usage in file collection, so just doing a roundtrip test
    #[test]
    fn slnt_roundtrip() {
        let mut slnt = SlntChunk {
            offset: Some(0),
            size: 4,
            data: Slnt { samples: 12345 },
            extra_bytes: Vec::new(),
        };
        println!("{slnt:?}");
        let mut buff = std::io::Cursor::new(Vec::<u8>::new());
        slnt.write(&mut buff).unwrap();
        println!("{:?}", hexdump(buff.get_ref()));
        buff.set_position(0);
        let after = SlntChunk::read(&mut buff).unwrap();
        assert_eq!(after, slnt);
        assert_eq!(after.data.samples, 12345);
        assert_eq!(after.summary(), "12345 samples");

        // now in a wavl LIST
        slnt.offset = Some(12);
        let wavl = ListWavlChunk {
            offset: Some(0),
            size: 16,
            data: ListWavl {
                list_type: ListWavl::LIST_TYPE,
                chunks: vec![WavlEnum::Slnt(slnt)],
            },
            extra_bytes: Vec::new(),
        };
        let mut buff = std::io::Cursor::new(Vec::<u8>::new());
        wavl.write(&mut buff).unwrap();
        println!("{:?}", hexdump(buff.get_ref()));
        buff.set_position(0);
        let after = ListWavlChunk::read(&mut buff).unwrap();
        assert_eq!(after, wavl);
    }

    #[test]
    fn data_in_wavl() {
        // validate data roundtrip
        let mut data = DataChunk {
            offset: Some(0),
            size: 0,
            data: Data {
                data: [8_u8; 0].to_vec(),
            },
            extra_bytes: Vec::new(),
        };
        let mut buff = std::io::Cursor::new(Vec::<u8>::new());
        data.write(&mut buff).unwrap();
        println!("{:?}", hexdump(buff.get_ref()));
        buff.set_position(0);
        let after = DataChunk::read(&mut buff).unwrap();
        assert_eq!(after, data);
        println!("length of data as bytes: {}", buff.into_inner().len());

        data.offset = Some(12);
        // finally validate via wavl
        let wavl = ListWavlChunk {
            offset: Some(0),
            size: 12,
            data: ListWavl {
                list_type: ListWavl::LIST_TYPE,
                chunks: vec![WavlEnum::Data(data)],
            },
            extra_bytes: Vec::new(),
        };
        let mut buff = std::io::Cursor::new(Vec::<u8>::new());
        wavl.write(&mut buff).unwrap();
        println!("{:?}", hexdump(buff.get_ref()));
        buff.set_position(0);
        let after = ListWavlChunk::read(&mut buff).unwrap();
        assert_eq!(after, wavl);
    }
}
