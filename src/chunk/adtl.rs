use std::fmt::Debug;

use binrw::{binrw, helpers};

use crate::{FourCC, KnownChunk, KnownChunkID, Summarizable};

#[binrw]
#[br(little)]
#[br(import(_size: u32))]
#[derive(Debug, PartialEq, Eq)]
pub struct ListAdtlData {
    #[brw(assert(list_type == ListAdtlData::LIST_TYPE))]
    pub list_type: FourCC,
    #[br(parse_with = helpers::until_eof)]
    #[bw()]
    // chunks: Vec<ListAdtl>,
    raw: Vec<u8>,
}

impl ListAdtlData {
    pub const LIST_TYPE: FourCC = FourCC(*b"adtl");
}

impl KnownChunkID for ListAdtlData {
    const ID: FourCC = FourCC(*b"LIST");
}

impl Summarizable for ListAdtlData {
    fn summary(&self) -> String {
        format!("{} ...", self.list_type)
    }

    fn name(&self) -> String {
        self.list_type.to_string().trim().to_string()
    }
}

pub type ListAdtl = KnownChunk<ListAdtlData>;
