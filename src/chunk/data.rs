use binrw::binrw;

use crate::{FourCC, KnownChunk, KnownChunkID, Summarizable};

/// `data` chunk parser which skips all audio data
#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
pub struct DataChunkData {}

impl KnownChunkID for DataChunkData {
    const ID: FourCC = FourCC(*b"data");
}

impl Summarizable for DataChunkData {
    fn summary(&self) -> String {
        "audio data".to_string()
    }
}

pub type DataChunk = KnownChunk<DataChunkData>;
