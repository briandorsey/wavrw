//! `adtl` A `LIST` containing [`CuePoint`](crate::chunk::cue::CuePoint) annotation chunks: file, labl, ltxt, note. [RIFF1991](https://wavref.til.cafe/chunk/adtl/)

use core::fmt::Debug;

use binrw::{binrw, helpers, NullString};
use itertools::Itertools;

use crate::{ChunkID, FourCC, KnownChunk, KnownChunkID, Summarizable};

#[binrw]
#[br(little)]
#[br(import(_size: u32))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// `LIST-adtl` Associated data list provides the ability to attach information like labels to sections of the waveform data stream.
pub struct ListAdtlData {
    /// A four-character code that identifies the contents of the list.
    #[brw(assert(list_type == ListAdtlData::LIST_TYPE))]
    pub list_type: FourCC,

    /// Sub chunks contained within this LIST
    #[br(parse_with = helpers::until_eof)]
    #[bw()]
    pub chunks: Vec<AdtlEnum>,
}

impl ListAdtlData {
    /// Chunk id constant: `adtl`
    pub const LIST_TYPE: FourCC = FourCC(*b"adtl");
}

impl KnownChunkID for ListAdtlData {
    const ID: FourCC = FourCC(*b"LIST");
}

impl Summarizable for ListAdtlData {
    fn summary(&self) -> String {
        self.chunks
            .iter()
            .into_grouping_map_by(|c| c.id())
            .fold(0, |acc, _key, _value| acc + 1)
            .iter()
            .map(|(g, c)| format!("{}({})", g, c))
            .sorted_unstable()
            .join(", ")
    }

    fn name(&self) -> String {
        format!("{}-{}", self.id(), self.list_type)
    }

    fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        Box::new(
            self.chunks
                .iter()
                .map(|c| (c.id().to_string(), c.summary())),
        )
    }
}

/// `adtl` Associated data list provides the ability to attach information like labels to sections of the waveform data stream.
pub type ListAdtl = KnownChunk<ListAdtlData>;

#[binrw]
#[br(little)]
#[br(import(_size: u32))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// `labl` A label, or title, to associate with a [`CuePoint`][super::CuePoint].
pub struct LablData {
    /// Specifies the cue point name. This value must match one of the names listed in the `cue` chunk's [CuePoint][super::CuePoint] table.
    pub name: u32,

    /// Specifies a NULL-terminated string containing a text label.
    #[br(map= |ns: NullString| ns.to_string())]
    #[bw(map= |s: &String| NullString::from(s.clone()))]
    pub text: String,
}

impl KnownChunkID for LablData {
    const ID: FourCC = FourCC(*b"labl");
}

impl Summarizable for LablData {
    fn summary(&self) -> String {
        format!("{:>3}, {}", self.name, self.text)
    }
}

/// `labl` A label, or title, to associate with a [`CuePoint`][super::CuePoint].
pub type Labl = KnownChunk<LablData>;

#[binrw]
#[br(little)]
#[br(import(_size: u32))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// `note` Comment text for a [`CuePoint`][super::CuePoint].
pub struct NoteData {
    /// Specifies the cue point name. This value must match one of the names listed in the `cue` chunk's [CuePoint][super::CuePoint] table.
    pub name: u32,

    /// Specifies a NULL-terminated string containing comment text.
    #[br(map= |ns: NullString| ns.to_string())]
    #[bw(map= |s: &String| NullString::from(s.clone()))]
    pub text: String,
}

impl KnownChunkID for NoteData {
    const ID: FourCC = FourCC(*b"note");
}

impl Summarizable for NoteData {
    fn summary(&self) -> String {
        format!("{:>3}, {}", self.name, self.text)
    }
}

/// `note` Comment text for a [`CuePoint`][super::CuePoint].
pub type Note = KnownChunk<NoteData>;

/// `ltxt` Text associated with a range of `data` samples.
#[binrw]
#[br(little)]
#[br(import(size: u32))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LtxtData {
    /// Specifies the cue point name. This value must match one of the names listed in the `cue` chunk's [CuePoint][super::CuePoint] table.
    pub name: u32,

    /// Specifies the number of samples in the segment of waveform data. 	...>sample_length
    pub sample_length: u32,

    /// Specifies the type or purpose of the text. For example, dwPurpose can specify a FOURCC code like `scrp` for script text or `capt` for close-caption text. `rgn ` is commonly used for "region notes"
    pub purpose: u32,

    /// Specifies the country code for the text. See "Country Codes" in CSET chunk, for a current list of country codes.
    pub country_code: u16,

    /// Specify the language for the text. See "Language and Dialect Codes" in CSET chunk, for a current list of language and dialect codes.
    pub language: u16,

    /// Specify the dialect codes for the text. See "Language and Dialect Codes" in CSET chunk, for a current list of language and dialect codes.
    pub dialect: u16,

    ///	Specifies the code page for the text. See CSET chunk for details.
    pub code_page: u16,

    /// The text associated with this range.
    #[br(count = size as u64 -4 -4 -4 -2 -2 -2 -2, 
        try_map = |v: Vec<u8>|  String::from_utf8(v) 
    )]
    #[bw(map = |s: &String| s.as_bytes())]
    pub text: String,
}

impl KnownChunkID for LtxtData {
    const ID: FourCC = FourCC(*b"ltxt");
}

impl Summarizable for LtxtData {
    fn summary(&self) -> String {
        format!(
            "{:>3}, len:{}, purpose:{}, {}",
            self.name,
            self.sample_length,
            FourCC(self.purpose.to_le_bytes()),
            self.text
        )
    }
}

/// `ltxt` Text associated with a range of `data` samples.
pub type Ltxt = KnownChunk<LtxtData>;

/// `file` Information embedded in other file formats.
///
/// Example formats: an 'RDIB' file or an ASCII text file.
///
/// NOTE: Implemented from the spec only, because I couldn’t find any files
/// actually containing this chunk.
#[binrw]
#[br(little)]
#[br(import(size: u32))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileData {
    /// Specifies the cue point name. This value must match one of the names listed in the `cue` chunk's [CuePoint][super::CuePoint] table.
    pub name: u32,

    /// Specifies the file type contained in the `file_data` field. If the fileData section contains a RIFF form, the `media_type` field is the same as the RIFF form type for the file. This field can contain a zero value.
    pub media_type: u32,

    /// Contains the media file.
    #[br(count = size as u64 -4 -4 )]
    pub file_data: Vec<u8>,
}

impl KnownChunkID for FileData {
    const ID: FourCC = FourCC(*b"ltxt");
}

impl Summarizable for FileData {
    fn summary(&self) -> String {
        format!(
            "{:>3}, media_type:{}, {} bytes",
            self.name,
            FourCC(self.media_type.to_le_bytes()),
            self.file_data.len()
        )
    }
}

/// `file` Information embedded in other file formats.
pub type File = KnownChunk<FileData>;

/// All `LIST-adtl` chunk structs as an enum
#[allow(missing_docs)]
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AdtlEnum {
    Labl(Labl),
    Note(Note),
    Ltxt(Ltxt),
    File(File),
    Unknown {
        id: FourCC,
        size: u32,
        #[brw(align_after = 2)]
        #[br(count = size, pad_size_to= size.to_owned())]
        raw: Vec<u8>,
    },
}

impl ChunkID for AdtlEnum {
    fn id(&self) -> FourCC {
        match self {
            AdtlEnum::Labl(e) => e.id(),
            AdtlEnum::Note(e) => e.id(),
            AdtlEnum::Ltxt(e) => e.id(),
            AdtlEnum::File(e) => e.id(),
            AdtlEnum::Unknown { id, .. } => *id,
        }
    }
}

impl Summarizable for AdtlEnum {
    fn summary(&self) -> String {
        match self {
            AdtlEnum::Labl(e) => e.summary(),
            AdtlEnum::Note(e) => e.summary(),
            AdtlEnum::Ltxt(e) => e.summary(),
            AdtlEnum::File(e) => e.summary(),
            AdtlEnum::Unknown { .. } => "...".to_string(),
        }
    }
}

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {
    use binrw::BinRead;

    use super::*;
    use crate::testing::hex_to_cursor;

    #[test]
    fn adtl_valid() {
        // LIST-adtl chunk containing 5 labl, 5 ltxt, and 2 note chunks
        let mut buff = hex_to_cursor(
"4C495354 24010000 6164746C 6C747874 14000000 01000000 81212000 72676E20 00000000 00000000 6C61626C 10000000 01000000 316B2040 202D3130 64420000 6C747874 14000000 02000000 D5A66400 72676E20 00000000 00000000 6C61626C 0E000000 02000000 316B487A 20546573 74006C74 78741400 00000300 00006A23 05007267 6E200000 00000000 00006C61 626C0A00 00000300 00004469 72616300 6C747874 14000000 04000000 22130200 72676E20 00000000 00000000 6C61626C 0A000000 04000000 43686972 70006E6F 74650800 00000400 00004C6F 67006C74 78741400 00000500 0000CF38 3A007267 6E200000 00000000 00006C61 626C0C00 00000500 00005377 65657020 00006E6F 74651600 00000500 00003130 487A2D39 366B487A 20333020 53656300"
            );

        let adtl = ListAdtl::read(&mut buff).unwrap();
        dbg!(&adtl);
        assert_eq!(adtl.id(), FourCC(*b"LIST"));
        assert_eq!(adtl.data.list_type, FourCC(*b"adtl"));
        assert_eq!(adtl.data.chunks.len(), 12);
        assert_eq!(adtl.data.chunks[3].id(), FourCC(*b"labl"));
        assert_eq!(adtl.data.chunks[3].summary(), "  2, 1kHz Test");
    }
}
