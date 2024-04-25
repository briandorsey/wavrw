use core::fmt::Debug;

use binrw::binrw;

use crate::{ChunkID, FourCC, KnownChunk, KnownChunkID, Summarizable};

/// A position in the waveform `data` chunk.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CuePoint {
    /// Specifies the cue point name. Each `CuePoint` must have a unique name field.
    pub name: u32,
    /// Specifies the sample position of the `CuePoint`. This is the sequential sample number within the play order.
    pub position: u32,
    /// Specifies the name or chunk ID of the chunk containing the cue point.
    pub chunk_id: FourCC,
    /// Specifies the file position of the start of the chunk containing the cue point. This is a byte offset relative to the start of the data section of the 'wavl' LIST chunk.
    pub chunk_start: u32,
    /// Specifies the file position of the start of the block containing the position. This is a byte offset relative to the start of the data section of the ‘wavl’ LIST chunk.
    pub block_start: u32,
    /// Specifies the sample offset of the cue point relative to the start of the block.
    pub sample_offset: u32,
}

impl CuePoint {
    fn summary(&self) -> String {
        format!(
            "{:10}, {}, {:10}, {:10}, {:10}",
            self.position, self.chunk_id, self.chunk_start, self.block_start, self.sample_offset,
        )
    }
}

#[binrw]
#[brw(little)]
#[br(import(_size: u32))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// `cue ` A series of positions in the waveform `data` chunk. [RIFF1991](https://wavref.til.cafe/chunk/cue/)
pub struct Cue {
    /// Count of cue points. The number of times the cue-point struct repeats within this chunk.
    pub cue_points: u32, // dwCuePoints

    /// A series of [`CuePoint`]s.
    #[br(count = cue_points)]
    pub points: Vec<CuePoint>,
}

impl KnownChunkID for Cue {
    const ID: FourCC = FourCC(*b"cue ");
}

impl Cue {
    fn new() -> Self {
        Cue {
            cue_points: 0,
            points: Vec::new(),
        }
    }
}

impl Default for Cue {
    fn default() -> Self {
        Self::new()
    }
}

/// `cue ` A series of positions in the waveform `data` chunk. [RIFF1991](https://wavref.til.cafe/chunk/cue/)
pub type CueChunk = KnownChunk<Cue>;

impl Summarizable for Cue {
    fn summary(&self) -> String {
        let label = match self.cue_points {
            1 => "cue point",
            _ => "cue points",
        };
        format!("{} {label}", self.cue_points)
    }

    fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        let mut items: Vec<(String, String)> = Vec::new();
        for point in &self.points {
            items.push((format!("{}", point.name), point.summary()));
        }
        Box::new(items.into_iter())
    }

    fn item_summary_header(&self) -> String {
        "name: position, chunk_id, chunk_start, block_start, sample_offset".to_string()
    }

    fn name(&self) -> String {
        self.id().to_string().trim().to_string()
    }
}

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {
    use binrw::BinRead;

    use super::*;
    use crate::testing::hex_to_cursor;

    #[test]
    fn parse_cue() {
        // example bext chunk data from BWF MetaEdit
        let mut buff = hex_to_cursor(
            r#"63756520 4C000000 03000000 01000000 00000000 64617461 00000000 00000000 00000000 02000000 F0000000 64617461 00000000 00000000 F0000000 03000000 68010000 64617461 00000000 00000000 68010000"#,
        );
        let cue = CueChunk::read(&mut buff).expect("error parsing cue chunk");
        print!("{:?}", cue);
        assert_eq!(cue.data.cue_points, 3);
        assert_eq!(
            cue.data.points[1],
            CuePoint {
                name: 2,
                position: 240,
                chunk_id: FourCC(*b"data"),
                chunk_start: 0,
                block_start: 0,
                sample_offset: 240,
            },
        );
        assert_eq!(cue.extra_bytes.len(), 0);
    }
}
