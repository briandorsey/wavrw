use binrw::{binrw, helpers};

use crate::{fourcc, FourCC, KnownChunk, KnownChunkID, Summarizable};

/// `data` chunk parser which skips all audio data
#[binrw]
#[brw(little)]
#[br(import(_size: u32))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

/// `JUNK` Padding, filler or outdated information. [RIFF1991](https://wavref.til.cafe/chunk/junk/)
pub type JunkData = PaddingData<{ fourcc(b"JUNK") }>;
/// `PAD ` Padding, filler or outdated information. [UNKNOWN](https://wavref.til.cafe/chunk/pad/)
pub type PadData = PaddingData<{ fourcc(b"PAD ") }>;
/// `FLLR` Padding, filler or outdated information. [UNKNOWN](https://wavref.til.cafe/chunk/fllr/)
pub type FllrData = PaddingData<{ fourcc(b"FLLR") }>;

/// `JUNK` Padding, filler or outdated information. [RIFF1991](https://wavref.til.cafe/chunk/junk/)
pub type JunkChunk = KnownChunk<JunkData>;
/// `PAD ` Padding, filler or outdated information. [UNKNOWN](https://wavref.til.cafe/chunk/pad/)
pub type PadChunk = KnownChunk<PadData>;
/// `FLLR` Padding, filler or outdated information. [UNKNOWN](https://wavref.til.cafe/chunk/fllr/)
pub type FllrChunk = KnownChunk<FllrData>;
