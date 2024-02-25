use binrw::binrw;

use crate::{FourCC, KnownChunk, KnownChunkID, Summarizable};

/// `data` chunk parser which skips all audio data
#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
pub struct DataData {}

impl KnownChunkID for DataData {
    const ID: FourCC = FourCC(*b"data");
}

impl Summarizable for DataData {
    fn summary(&self) -> String {
        "audio data".to_string()
    }
}

pub type Data = KnownChunk<DataData>;
