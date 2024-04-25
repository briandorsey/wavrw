use binrw::binrw;

use crate::{FourCC, KnownChunk, KnownChunkID, Summarizable};

/// `fact` Number of samples for compressed audio in `data`. [RIFF1991](https://wavref.til.cafe/chunk/fact/)
///
/// The `fact` chunk is required if the waveform data is contained in a `wavl`
/// LIST chunk and for all compressed audio formats. The chunk is not required
/// for PCM files using the “ data” chunk format.
#[binrw]
#[brw(little)]
#[br(import(_size: u32))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Fact {
    /// Number of samples for audio in `data` chunk.
    pub samples: u32,
}

impl KnownChunkID for Fact {
    const ID: FourCC = FourCC(*b"fact");
}

impl Summarizable for Fact {
    fn summary(&self) -> String {
        format!("{} samples", self.samples)
    }
}

/// `fact` Number of samples for compressed audio in `data`. [RIFF1991](https://wavref.til.cafe/chunk/fact/)
pub type FactChunk = KnownChunk<Fact>;

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {
    use binrw::BinRead;

    use super::*;
    use crate::{testing::hex_to_cursor, ChunkID, SizedChunkEnum};

    #[test]
    fn factchunk_small_valid() {
        // buff contains ICMT chunk with an odd length
        // handling the WORD padding incorrectly can break parsing
        let mut buff = hex_to_cursor("66616374 04000000 E0010000");
        // parse via explicit chunk type
        let fact = FactChunk::read(&mut buff).unwrap();
        dbg!(&fact);
        assert_eq!(fact.id(), FourCC(*b"fact"));
        assert_eq!(fact.data.samples, 480);

        // parse via enum wrapper this time
        buff.set_position(0);
        let en = SizedChunkEnum::read(&mut buff).unwrap();
        dbg!(&en);
        assert_eq!(en.id(), FourCC(*b"fact"));
        let SizedChunkEnum::Fact(fact) = en else {
            unreachable!("should have been fact")
        };
        assert_eq!(fact.data.samples, 480);
    }
}
