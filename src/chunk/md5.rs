use binrw::binrw;

use crate::{FourCC, KnownChunk, KnownChunkID, SizedChunk, Summarizable};

// based on https://mediaarea.net/BWFMetaEdit/md5
#[binrw]
#[brw(little)]
#[br(import(_size: u32))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Md5Data {
    pub md5: u128,
}

impl KnownChunkID for Md5Data {
    const ID: FourCC = FourCC(*b"MD5 ");
}

impl SizedChunk for Md5Data {
    fn size(&self) -> u32 {
        16
    }
}

impl Summarizable for Md5Data {
    fn summary(&self) -> String {
        format!("0x{:X}", self.md5)
    }
}

pub type Md5 = KnownChunk<Md5Data>;

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {
    use binrw::BinRead;

    use super::*;
    use crate::testing::hex_to_cursor;

    #[test]
    fn parse_md5() {
        let mut buff = hex_to_cursor("4D443520 10000000 83F4C759 5E3F9608 378F3B39 D4BEA537");
        let expected = Md5 {
            size: 16,
            data: Md5Data {
                md5: 0x37A5BED4393B8F3708963F5E59C7F483,
            },
            extra_bytes: vec![],
        };

        let chunk = Md5::read(&mut buff).expect("error parsing WAV chunks");
        println!("chunk   : 0x{:X}", chunk.data.md5);
        println!("expected: 0x{:X}", expected.data.md5);
        assert_eq!(chunk, expected);
        // hexdump(remaining_input);
    }
}
