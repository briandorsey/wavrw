use std::fmt::Debug;

use binrw::binrw;

use crate::{ChunkID, FourCC, KnownChunk, KnownChunkID, Summarizable};

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlstSegment {
    /// The cue point name. This value must match a [`CuePoint.name`][crate::chunk::cue::CuePoint] in the [`CueData`][crate::chunk::cue::CueData] chunk.
    pub name: u32,

    /// The length of the section in samples.
    pub length: u32,

    /// The number of times to play the section.
    pub loops: u32,
}

impl PlstSegment {
    fn summary(&self) -> String {
        format!("{:7}, {:7}, {:7}", self.name, self.length, self.loops,)
    }
}

#[binrw]
#[brw(little)]
#[br(import(_size: u32))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// `plst` chunk contains character set information. Defined in RIFF1991.
pub struct PlstData {
    /// Count of plst segments. The number of times the `PlstSegment` struct repeats within this chunk.
    pub segment_count: u32, // dwSegments
    #[br(count = segment_count)]
    pub segments: Vec<PlstSegment>,
}

impl KnownChunkID for PlstData {
    const ID: FourCC = FourCC(*b"plst");
}

/// `plst` chunk contains character set information. Defined in RIFF1991.
///
/// NOTE: Implemented from the spec only, because I couldn't find any files actually
/// containing this chunk.
pub type Plst = KnownChunk<PlstData>;

impl Summarizable for PlstData {
    fn summary(&self) -> String {
        let label = match self.segment_count {
            1 => "segment",
            _ => "segments",
        };
        format!("{} {label}", self.segment_count)
    }

    fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        let mut items: Vec<(String, String)> = Vec::new();
        for segment in &self.segments {
            items.push((format!("{}", segment.name), segment.summary()));
        }
        Box::new(items.into_iter())
    }

    fn item_summary_header(&self) -> String {
        "name: length, loops".to_string()
    }

    fn name(&self) -> String {
        self.id().to_string().trim().to_string()
    }
}

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {
    use binrw::{BinRead, BinWrite};
    use hexdump::hexdump;
    use std::io::Read;

    use crate::testing::hex_to_cursor;

    use super::*;

    // couldn't find plst usage in file collection, so just doing a roundtrip test
    #[test]
    fn plst_roundtrip() {
        let plst = Plst {
            size: 0x1c, // u32 + 2x(3x u32)
            data: PlstData {
                segment_count: 2,
                segments: vec![
                    PlstSegment {
                        name: 4,
                        length: 5,
                        loops: 6,
                    },
                    PlstSegment {
                        name: 7,
                        length: 8,
                        loops: 9,
                    },
                ],
            },
            extra_bytes: vec![],
        };
        println!("{plst:?}");
        let mut buff = std::io::Cursor::new(Vec::<u8>::new());
        plst.write(&mut buff).unwrap();
        println!("{:?}", hexdump(buff.get_ref()));
        buff.set_position(0);
        println!("buff length: 0x{:X}", buff.clone().bytes().count());
        buff.set_position(0);
        let after = Plst::read(&mut buff).unwrap();
        assert_eq!(after, plst);
        // assert_eq!(after.data.name, 1);
        assert_eq!(after.data.segments[0].length, 5);
        assert_eq!(after.data.segments[0].loops, 6);
        assert_eq!(after.summary(), "2 segments");
    }

    #[test]
    fn parse_plst() {
        // example bext chunk data from BWF MetaEdit
        let mut buff = hex_to_cursor(r#"706c7374 10000000 01000000 01000000 02000000 03000000"#);
        let plst = Plst::read(&mut buff).expect("error parsing plst chunk");
        print!("{:?}", plst);
        assert_eq!(plst.data.segment_count, 1);
        assert_eq!(
            plst.data.segments[0],
            PlstSegment {
                name: 1,
                length: 2,
                loops: 3,
            },
        );
        assert_eq!(plst.extra_bytes.len(), 0);
    }
}
