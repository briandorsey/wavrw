use std::fmt::{Debug, Formatter};

use binrw::{binrw, helpers, NullString};
use itertools::Itertools;

use crate::{ChunkID, FourCC, KnownChunk, KnownChunkID, Summarizable};

#[binrw]
#[br(little)]
#[br(import(_size: u32))]
#[derive(Debug, PartialEq, Eq)]
pub struct ListAdtlData {
    #[brw(assert(list_type == ListAdtlData::LIST_TYPE))]
    pub list_type: FourCC,
    #[br(parse_with = helpers::until_eof)]
    #[bw()]
    chunks: Vec<AdtlEnum>,
}

impl ListAdtlData {
    pub const LIST_TYPE: FourCC = FourCC(*b"adtl");
}

impl KnownChunkID for ListAdtlData {
    const ID: FourCC = FourCC(*b"LIST");
}

impl Summarizable for ListAdtlData {
    fn summary(&self) -> String {
        format!(
            "{}",
            self.chunks
                .iter()
                .into_grouping_map_by(|c| c.id())
                .fold(0, |acc, _key, _value| acc + 1)
                .iter()
                .map(|(g, c)| format!("{}({})", g, c))
                .join(", ")
        )
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

    fn item_summary_header(&self) -> String {
        "chunk: summary".to_string()
    }
}

pub type ListAdtl = KnownChunk<ListAdtlData>;

#[binrw]
#[br(little)]
#[br(import(_size: u32))]
#[derive(PartialEq, Eq)]
pub struct LablData {
    pub name: u32,
    pub text: NullString,
}

impl KnownChunkID for LablData {
    const ID: FourCC = FourCC(*b"labl");
}

impl Debug for LablData {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "LablData<{}> {{ value: {:?} }}", LablData::ID, self.text,)?;
        Ok(())
    }
}

impl Summarizable for LablData {
    fn summary(&self) -> String {
        format!("{:>3}, {}", self.name, self.text.to_string())
    }
}

pub type Labl = KnownChunk<LablData>;

// impl LablData {
//     pub fn new(text: &str) -> Self {
//         LablData { text: text.into() }
//     }
// }

#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
pub enum AdtlEnum {
    Labl(Labl),
    Unknown {
        id: FourCC,
        size: u32,
        #[br(count = size, align_after=2, pad_size_to= size.to_owned())]
        raw: Vec<u8>,
    },
}
impl AdtlEnum {
    pub fn id(&self) -> FourCC {
        match self {
            AdtlEnum::Labl(e) => e.id(),
            AdtlEnum::Unknown { id, .. } => *id,
        }
    }

    pub fn summary(&self) -> String {
        match self {
            AdtlEnum::Labl(e) => e.summary(),
            AdtlEnum::Unknown { .. } => "...".to_string(),
        }
    }
}
