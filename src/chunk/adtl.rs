use std::fmt::Debug;

use binrw::{binrw, helpers};

use crate::{FourCC, KnownChunk, KnownChunkID, Summarizable};

#[binrw]
#[br(little)]
#[derive(Debug, PartialEq, Eq)]
pub struct ListAdtlChunkData {
    #[brw(assert(list_type == ListAdtlChunkData::LIST_TYPE))]
    pub list_type: FourCC,
    #[br(parse_with = helpers::until_eof)]
    #[bw()]
    // chunks: Vec<AdtlChunk>,
    raw: Vec<u8>,
}

impl ListAdtlChunkData {
    pub const LIST_TYPE: FourCC = FourCC(*b"adtl");
}

impl KnownChunkID for ListAdtlChunkData {
    const ID: FourCC = FourCC(*b"LIST");
}

impl Summarizable for ListAdtlChunkData {
    fn summary(&self) -> String {
        format!("{} ...", self.list_type)
    }

    fn name(&self) -> String {
        self.list_type.to_string().trim().to_string()
    }
}

pub type ListAdtlChunk = KnownChunk<ListAdtlChunkData>;
