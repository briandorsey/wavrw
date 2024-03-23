use binrw::binrw;
use binrw::io::SeekFrom;

use crate::{FourCC, KnownChunk, KnownChunkID, Summarizable};

/// `data` Audio samples. This parser skips all audio data (for now). [RIFF1991](https://wavref.til.cafe/spec/riff1991/)
#[binrw]
#[brw(little)]
#[br(import(size: u32))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataData {
    // Not public until we figure out design for loading data. (issue #72)
    //
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

/// `data` Audio samples. This parser skips all audio data (for now). [RIFF1991](https://wavref.til.cafe/spec/riff1991/)
pub type Data = KnownChunk<DataData>;
