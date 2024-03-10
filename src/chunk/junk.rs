use binrw::{binrw, helpers};

use crate::{fourcc, FourCC, KnownChunk, KnownChunkID, Summarizable};

/// `data` chunk parser which skips all audio data
#[binrw]
#[brw(little)]
#[br(import(_size: u32))]
#[derive(Debug, PartialEq, Eq)]
pub struct PaddingData<const I: u32> {
    #[br(parse_with = helpers::until_eof)]
    data: Vec<u8>,
}

impl<const I: u32> KnownChunkID for PaddingData<I> {
    const ID: FourCC = FourCC(I.to_le_bytes());
}

impl<const I: u32> Summarizable for PaddingData<I> {
    fn summary(&self) -> String {
        "padding, filler or outdated information".to_string()
    }
}

pub type JunkData = PaddingData<{ fourcc(b"JUNK") }>;
pub type PadData = PaddingData<{ fourcc(b"PAD ") }>;
pub type FllrData = PaddingData<{ fourcc(b"FLLR") }>;

pub type Junk = KnownChunk<JunkData>;
pub type Pad = KnownChunk<PadData>;
pub type Fllr = KnownChunk<FllrData>;
