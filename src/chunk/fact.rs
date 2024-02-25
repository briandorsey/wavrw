use binrw::binrw;

use crate::{FourCC, KnownChunk, KnownChunkID, SizedChunk, Summarizable};

#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
pub struct FactData {
    pub samples: u32,
}

impl KnownChunkID for FactData {
    const ID: FourCC = FourCC(*b"fact");
}

impl SizedChunk for FactData {
    fn size(&self) -> u32 {
        4
    }
}

impl Summarizable for FactData {
    fn summary(&self) -> String {
        format!("{} samples", self.samples)
    }
}

pub type Fact = KnownChunk<FactData>;

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {
    use binrw::BinRead;

    use super::*;
    use crate::{testing::hex_to_cursor, ChunkEnum, ChunkID};

    #[test]
    fn factchunk_small_valid() {
        // buff contains ICMT chunk with an odd length
        // handling the WORD padding incorrectly can break parsing
        let mut buff = hex_to_cursor("66616374 04000000 E0010000");
        // parse via explicit chunk type
        let fact = Fact::read(&mut buff).unwrap();
        dbg!(&fact);
        assert_eq!(fact.id(), FourCC(*b"fact"));
        assert_eq!(fact.data.samples, 480);

        // parse via enum wrapper this time
        buff.set_position(0);
        let en = ChunkEnum::read(&mut buff).unwrap();
        dbg!(&en);
        assert_eq!(en.id(), FourCC(*b"fact"));
        let ChunkEnum::Fact(fact) = en else {
            unreachable!("should have been fact")
        };
        assert_eq!(fact.data.samples, 480);
    }
}
