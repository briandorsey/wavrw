use std::io::SeekFrom;

use binrw::binrw;

use crate::{FourCC, KnownChunk, KnownChunkID, Summarizable};

/// `data` chunk parser which skips all audio data
#[binrw]
#[brw(little)]
#[br(import(size: u32))]
#[derive(Debug, PartialEq, Eq)]
pub struct DataData {
    // For some reason SeekFrom::End(0) takes us beyond this chunk, even though
    // KnownChunk uses take_seek to limit the reader. I don't know why.
    // Instead, we're passing in the chunk size and using that.
    // The reason for this dance is to seek past the (often very large)
    // audio data entirely, so we don't read it into KnownChunk.extra_bytes.
    #[br(count = 0, seek_before(SeekFrom::Start(size.into())))]
    data: Vec<u8>,
}

impl KnownChunkID for DataData {
    const ID: FourCC = FourCC(*b"data");
}

impl Summarizable for DataData {
    fn summary(&self) -> String {
        "audio data".to_string()
    }
}

pub type Data = KnownChunk<DataData>;
