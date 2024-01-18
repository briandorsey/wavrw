use binrw::io::TakeSeekExt;
use binrw::NullString;
use binrw::{binrw, helpers, io::SeekFrom};
use itertools::Itertools;
use std::cmp::min;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

// TODO: enum for fmt chunk
// TODO: test  offset += chunk.size(); equals actual id locaiton
// TODO: ensure chunk sizes are always an even number, per RIFF specs. Probably use align_* args on brw attributes.
// consider refactoring Chunk to hold id, size and raw data, with enum for parsed data

// helper types
// ----

#[binrw]
#[brw(big)]
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct FourCC(pub [u8; 4]);

impl Display for FourCC {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", String::from_utf8_lossy(&self.0),)?;
        Ok(())
    }
}

impl Debug for FourCC {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "FourCC({}=", String::from_utf8_lossy(&self.0),)?;
        write!(f, "{:?})", &self.0)?;
        Ok(())
    }
}

#[derive(Debug)]
struct FixedStrErr;

#[binrw]
#[brw(little)]
#[derive(PartialEq, Eq)]
/// FixedStr holds Null terminated fixed length strings (from BEXT for example)
struct FixedStr<const N: usize>([u8; N]);

// FixedStr display design question. RIFF spec uses ""Z notation for fixed strings. Should we do the same?

impl<const N: usize> Debug for FixedStr<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "FixedStr<{}>(\"{}\")", N, self)
    }
}

impl<const N: usize> Display for FixedStr<N> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            String::from_utf8_lossy(&self.0).trim_end_matches('\0')
        )
    }
}

impl<const N: usize> FromStr for FixedStr<N> {
    type Err = FixedStrErr;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        let mut array_tmp = [0u8; N];
        let l = min(s.len(), N);
        array_tmp[..l].copy_from_slice(&s.as_bytes()[..l]);
        Ok(FixedStr::<N>(array_tmp))
    }
}

// parsing structs
// ----

#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
// http://www.tactilemedia.com/info/MCI_Control_Info.html
pub struct WavMetadata {
    pub id: FourCC,
    pub size: u32,
    pub form_type: FourCC,
    #[br(parse_with = helpers::until_eof)]
    // TODO: revisit this enum design... chunks are large... maybe it should be a Vec of Traits instead? Research how binrw parsing might work in that case. Maybe go back to parsing each chunk while manually iterating through the file?
    pub chunks: Vec<Chunk>,
}

// based on http://soundfile.sapp.org/doc/WaveFormat/
#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
pub struct FmtChunk {
    #[brw(seek_before = SeekFrom::Current(-4))]
    id: FourCC,
    size: u32,
    format_tag: u16,
    channels: u16,
    samples_per_sec: u32,
    avg_bytes_per_sec: u32,
    block_align: u16,
    bits_per_sample: u16,
}
// TODO: properly handle different fmt chunk additions from later specs

impl FmtChunk {
    pub fn summary(&self) -> String {
        format!(
            "{} chan, {}/{}",
            self.channels,
            self.bits_per_sample,
            self.samples_per_sec,
            // TODO: format_tag
        )
    }

    pub fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        Box::new(self.into_iter())
    }
}

// Iteration based on pattern from https://stackoverflow.com/questions/30218886/how-to-implement-iterator-and-intoiterator-for-a-simple-struct

impl<'a> IntoIterator for &'a FmtChunk {
    type Item = (String, String);
    type IntoIter = FmtChunkIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        FmtChunkIterator {
            fmt: self,
            index: 0,
        }
    }
}

pub struct FmtChunkIterator<'a> {
    fmt: &'a FmtChunk,
    index: usize,
}

impl<'a> Iterator for FmtChunkIterator<'a> {
    type Item = (String, String);
    fn next(&mut self) -> Option<(String, String)> {
        self.index += 1;
        match self.index {
            1 => Some(("format_tag".to_string(), self.fmt.format_tag.to_string())),
            2 => Some(("channels".to_string(), self.fmt.channels.to_string())),
            3 => Some((
                "samples_per_sec".to_string(),
                self.fmt.samples_per_sec.to_string(),
            )),
            4 => Some((
                "avg_bytes_per_sec".to_string(),
                self.fmt.avg_bytes_per_sec.to_string(),
            )),
            5 => Some(("block_align".to_string(), self.fmt.block_align.to_string())),
            6 => Some((
                "bits_per_sample".to_string(),
                self.fmt.bits_per_sample.to_string(),
            )),
            _ => None,
        }
    }
}

// based on http://soundfile.sapp.org/doc/WaveFormat/
/// `data` chunk parser which skips all audio data
#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
pub struct DataChunk {
    #[brw(seek_before = SeekFrom::Current(-4))]
    id: FourCC,
    size: u32,
    #[brw(seek_before = SeekFrom::Current(size.clone().into()), ignore)]
    end_of_chunk: [u8; 0],
}

impl DataChunk {
    pub fn summary(&self) -> String {
        format!("audio data, len: {} bytes", self.size)
    }
}

#[binrw]
#[br(little)]
#[derive(Debug, PartialEq, Eq)]
pub struct ListInfoChunk {
    #[brw(seek_before = SeekFrom::Current(-4))]
    id: FourCC,
    size: u32,
    #[brw(magic = b"INFO", seek_before = SeekFrom::Current(-4))]
    list_type: FourCC,
    #[br(map_stream = |reader| reader.take_seek(size as u64 - 4u64), parse_with = helpers::until_eof)]
    #[bw()]
    chunks: Vec<InfoChunk>,
}

impl ListInfoChunk {
    pub fn summary(&self) -> String {
        format!(
            "{}: {}",
            self.list_type,
            self.chunks.iter().map(|c| c.id()).join(", ")
        )
    }

    pub fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        Box::new(self.chunks.iter().map(|c| (c.id().to_string(), c.value())))
    }
}
#[binrw]
#[br(little)]
#[derive(Debug, PartialEq, Eq)]
pub enum InfoChunk {
    // TODO: the rest of the INFO chunks
    #[brw(magic = b"ISFT")]
    Isft {
        #[brw(seek_before = SeekFrom::Current(-4))]
        id: FourCC,
        size: u32,
        #[brw(align_after = 2, pad_size_to= size.to_owned())]
        value: NullString,
    },
    #[brw(magic = b"ICMT")]
    Icmt {
        #[brw(seek_before = SeekFrom::Current(-4))]
        id: FourCC,
        size: u32,
        #[brw(align_after = 2, pad_size_to= size.to_owned())]
        value: NullString,
    },
    Unknown {
        // #[br(dbg)]
        id: FourCC,
        // #[br(dbg)]
        size: u32,
        #[brw(align_after=2, pad_size_to= size.to_owned())]
        value: NullString,
    },
}

impl InfoChunk {
    pub fn id(&self) -> FourCC {
        match self {
            InfoChunk::Isft { id, .. } => *id,
            InfoChunk::Icmt { id, .. } => *id,
            InfoChunk::Unknown { id, .. } => *id,
        }
    }

    pub fn value(&self) -> String {
        match self {
            InfoChunk::Isft { value, .. } => (*value).to_string(),
            InfoChunk::Icmt { value, .. } => (*value).to_string(),
            InfoChunk::Unknown { value, .. } => format!("Unknown(\"{}\")", *value),
        }
    }

    pub fn _summary(&self) -> String {
        format!("{}: {}", self.id(), self.value())
    }
}

#[binrw]
#[br(little)]
#[derive(Debug, PartialEq, Eq)]
pub struct ListAdtlChunk {
    #[brw(seek_before = SeekFrom::Current(-4))]
    id: FourCC,
    size: u32,
    #[brw(magic = b"adtl", seek_before = SeekFrom::Current(-4))]
    list_type: FourCC,
    // need to add magic here to choose the right enum
    // items: ListType,
    #[br(count = size -4 )]
    #[bw()]
    raw: Vec<u8>,
}

impl ListAdtlChunk {
    pub fn summary(&self) -> String {
        format!("{}", self.list_type)
    }
}

// BEXT, based on https://tech.ebu.ch/docs/tech/tech3285.pdf
// BEXT is specified to use ASCII, but we're parsing it as utf8, since
// that is a superset of ASCII and many WAV files contain utf8 strings here
#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
pub struct BextChunk {
    #[brw(seek_before = SeekFrom::Current(-4))]
    id: FourCC,
    size: u32,
    /// Description of the sound sequence
    description: FixedStr<256>, // Description
    /// Name of the originator
    originator: FixedStr<32>, // Originator
    /// Reference of the originator
    originator_reference: FixedStr<32>, // OriginatorReference
    /// yyyy:mm:dd
    origination_date: FixedStr<10>, // OriginationDate
    /// hh:mm:ss
    origination_time: FixedStr<8>, // OriginationTime
    // TODO: validate endianness, spec has DWORD high then DWORD low
    /// First sample count since midnight
    time_reference: u64, // TimeReference
    /// Version of the BWF; unsigned binary number
    version: u16, // Version
    /// SMPTE UMID
    // TODO: write UMID parser, based on: SMPTE 330M
    umid: [u8; 64], // UMID
    /// Integrated Loudness Value of the file in LUFS (multiplied by 100)
    loudness_value: i16, // LoudnessValue
    /// Integrated Loudness Range of the file in LUFS (multiplied by 100)
    loudness_range: i16, // LoudnessRange
    /// Maximum True Peak Level of the file expressed as dBTP (multiplied by 100)
    max_true_peak_level: i16, // MaxTruePeakLevel
    /// Highest value of the Momentary Loudness Level of the file in LUFS (multiplied by 100)
    max_momentary_loudness: i16, // MaxMomentaryLoudness
    /// Highest value of the Short-Term Loudness Level of the file in LUFS (multiplied by 100)
    max_short_term_loudness: i16, // MaxShortTermLoudness
    /// 180 bytes, reserved for future use, set to “NULL”
    reserved: [u8; 180], // Reserved
    /// History coding
    // interpret the remaining bytes as string
    #[br(align_after = 2, count = size -256 -32 -32 -10 -8 -8 -2 -64 -2 -2 -2 -2 -2 -180, map = |v: Vec<u8>| String::from_utf8_lossy(&v).to_string())]
    #[bw(align_after = 2, map = |s: &String| s.as_bytes())]
    coding_history: String, // CodingHistory
}

impl BextChunk {
    pub fn summary(&self) -> String {
        format!(
            "{}, {}, {}",
            self.origination_date, self.origination_time, self.description
        )
    }

    pub fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        Box::new(self.into_iter())
    }
}

impl<'a> IntoIterator for &'a BextChunk {
    type Item = (String, String);
    type IntoIter = BextChunkIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BextChunkIterator {
            bext: self,
            index: 0,
        }
    }
}

pub struct BextChunkIterator<'a> {
    bext: &'a BextChunk,
    index: usize,
}

impl<'a> Iterator for BextChunkIterator<'a> {
    type Item = (String, String);
    fn next(&mut self) -> Option<(String, String)> {
        self.index += 1;
        match self.index {
            1 => Some(("description".to_string(), self.bext.description.to_string())),
            2 => Some(("originator".to_string(), self.bext.originator.to_string())),
            3 => Some((
                "origination_date".to_string(),
                self.bext.origination_date.to_string(),
            )),
            4 => Some((
                "origination_time".to_string(),
                self.bext.origination_time.to_string(),
            )),
            5 => Some((
                "time_reference".to_string(),
                self.bext.time_reference.to_string(),
            )),
            6 => Some(("version".to_string(), self.bext.version.to_string())),
            7 => Some(("umid".to_string(), hex::encode(self.bext.umid))),
            8 => Some((
                "loudness_value".to_string(),
                self.bext.loudness_value.to_string(),
            )),
            9 => Some((
                "loudness_range".to_string(),
                self.bext.loudness_range.to_string(),
            )),
            10 => Some((
                "max_true_peak_level".to_string(),
                self.bext.max_true_peak_level.to_string(),
            )),
            11 => Some((
                "max_momentary_loudness".to_string(),
                self.bext.max_momentary_loudness.to_string(),
            )),
            12 => Some((
                "max_short_term_loudness".to_string(),
                self.bext.max_short_term_loudness.to_string(),
            )),
            13 => Some((
                "coding_history".to_string(),
                self.bext.coding_history.to_string(),
            )),
            _ => None,
        }
    }
}

// based on https://mediaarea.net/BWFMetaEdit/md5
#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
pub struct Md5Chunk {
    #[brw(seek_before = SeekFrom::Current(-4))]
    id: FourCC,
    size: u32,
    md5: u128,
}

impl Md5Chunk {
    pub fn summary(&self) -> String {
        format!("0x{:X}", self.md5)
    }
}

#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
pub enum Chunk {
    // TODO: add DATA parsing which skips actual data
    #[brw(magic = b"fmt ")]
    Fmt(FmtChunk),
    #[brw(magic = b"data")]
    Data(DataChunk),
    #[brw(magic = b"LIST")]
    Info(ListInfoChunk),
    #[brw(magic = b"LIST")]
    Adtl(ListAdtlChunk),
    #[brw(magic = b"bext")]
    Bext(Box<BextChunk>),
    #[brw(magic = b"MD5 ")]
    Md5(Md5Chunk),
    Unknown {
        id: FourCC,
        size: u32,
        #[br(count = size )]
        #[bw()]
        raw: Vec<u8>,
    },
}

impl Chunk {
    /// Returns the [FourCC] (chunk id) for the contained chunk.
    pub fn id(&self) -> FourCC {
        // TODO: research: is it possible to match on contained structs with a specific trait to reduce repetition?
        match self {
            Chunk::Fmt(e) => e.id,
            Chunk::Data(e) => e.id,
            Chunk::Info(e) => e.id,
            Chunk::Adtl(e) => e.id,
            Chunk::Bext(e) => e.id,
            Chunk::Md5(e) => e.id,
            Chunk::Unknown { id, .. } => *id,
        }
    }

    /// Returns the logical (used) size in bytes of the contained chunk.
    pub fn size(&self) -> u32 {
        match self {
            Chunk::Fmt(e) => e.size,
            Chunk::Data(e) => e.size,
            Chunk::Info(e) => e.size,
            Chunk::Adtl(e) => e.size,
            Chunk::Bext(e) => e.size,
            Chunk::Md5(e) => e.size,
            Chunk::Unknown { size, .. } => *size,
        }
    }

    /// Returns a short text summary of the contents of the contained chunk.
    pub fn summary(&self) -> String {
        match self {
            Chunk::Fmt(e) => e.summary(),
            Chunk::Data(e) => e.summary(),
            Chunk::Info(e) => e.summary(),
            Chunk::Adtl(e) => e.summary(),
            Chunk::Bext(e) => e.summary(),
            Chunk::Md5(e) => e.summary(),
            Chunk::Unknown { .. } => "...".to_owned(),
        }
    }

    /// Returns an iterator over a sequence of contents of the contained
    /// chunk as (field, value).
    pub fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        match self {
            Chunk::Fmt(e) => Box::new(e.into_iter()),
            Chunk::Info(e) => Box::new(e.items()),
            Chunk::Bext(e) => Box::new(e.items()),
            _ => Box::new(std::iter::empty()),
        }
    }
}

#[cfg(test)]
mod test {
    use binrw::BinRead; // don't understand why this is needed in this scope
    use std::io::Cursor;

    use super::*;
    use hex::decode;

    fn hex_to_cursor(data: &str) -> Cursor<Vec<u8>> {
        let data = data.replace(' ', "");
        let data = data.replace('\n', "");
        let data = decode(data).unwrap();
        Cursor::new(data)
    }

    #[test]
    fn fixed_string() {
        let fs = FixedStr::<6>(*b"abc\0\0\0");
        assert_eq!(6, fs.0.len());
        let s = fs.to_string();
        assert_eq!("abc".to_string(), s);
        assert_eq!(3, s.len());
        let new_fs = FixedStr::<6>::from_str(&s).unwrap();
        assert_eq!(fs, new_fs);
        assert_eq!(6, fs.0.len());
        assert_eq!(
            b"\0\0\0"[..],
            new_fs.0[3..6],
            "extra space after the string should be null bytes"
        );

        // strings longer than fixed size should get truncated
        let long_str = "this is a longer str";
        let fs = FixedStr::<6>::from_str(long_str).unwrap();
        assert_eq!("this i", fs.to_string());
    }

    #[test]
    fn riff_header() {
        // RIFF 2398 WAVE
        let header = "524946465E09000057415645";
        let mut data = hex_to_cursor(header);
        println!("{header:?}");
        let wavfile = WavMetadata::read(&mut data).unwrap();
        assert_eq!(
            WavMetadata {
                id: FourCC(*b"RIFF"),
                size: 2398,
                form_type: FourCC(*b"WAVE"),
                chunks: vec![],
            },
            wavfile
        );
    }

    // #[test]
    // fn parse_length() {
    //     let tests = [(
    //         &decode("666D7420 10000000 01000100 80BB0000 80320200 03001800".replace(' ', ""))
    //             .unwrap(),
    //         UnknownChunk {
    //             id: "fmt ".as_bytes().try_into().unwrap(),
    //             size: 16,
    //         },
    //         &[] as &[u8],
    //     )];
    //     for (input, expected_output, expected_remaining_input) in tests {
    //         hexdump(input);
    //         let (remaining_input, output) = parse_chunk(input).unwrap();
    //         assert_eq!(expected_output, output);
    //         assert_eq!(expected_remaining_input, remaining_input);
    //     }
    // }

    #[test]
    fn parse_header_fmt() {
        let data = hex_to_cursor(
            "52494646 5E090000 57415645 666D7420 10000000 01000100 80BB0000 80320200 03001800",
        );
        let tests = [(
            data,
            WavMetadata {
                id: FourCC(*b"RIFF"),
                size: 2398,
                form_type: FourCC(*b"WAVE"),
                chunks: vec![Chunk::Fmt(FmtChunk {
                    id: FourCC(*b"fmt "),
                    size: 16,
                    format_tag: 1,
                    channels: 1,
                    samples_per_sec: 48000,
                    avg_bytes_per_sec: 144000,
                    block_align: 3,
                    bits_per_sample: 24,
                })],
            },
        )];
        for (mut input, expected_output) in tests {
            // hexdump(input);
            let output = WavMetadata::read(&mut input).expect("error parsing wav");
            assert_eq!(expected_output, output);
            // hexdump(remaining_input);
        }
    }

    #[test]
    fn parse_bext() {
        // example bext chunk data from BWF MetaEdit
        let mut buff = hex_to_cursor(
            r#"62657874 67020000 44657363 72697074 696F6E00 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 4F726967 696E6174 6F720000 
            00000000 00000000 00000000 00000000 00000000 4F726967 696E6174 
            6F725265 66657265 6E636500 00000000 00000000 00000000 32303036 
            2F30312F 30323033 3A30343A 30353930 00000000 00000200 060A2B34 
            01010101 01010210 13000000 00FF122A 69370580 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 6400C800 2C019001 F4010000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 0000436F 
            64696E67 48697374 6F7279"#,
        );
        // work around for FourCC parsing location... TODO: can we move this seek to enclosing enum?
        buff.set_position(4);
        let bext = BextChunk::read(&mut buff).expect("error parsing bext chunk");
        print!("{:?}", bext);
        assert_eq!(
            bext.description,
            FixedStr::<256>::from_str("Description").unwrap(),
            "description"
        );
        assert_eq!(
            bext.originator,
            FixedStr::<32>::from_str("Originator").unwrap(),
            "originator"
        );
        assert_eq!(
            bext.originator_reference,
            FixedStr::<32>::from_str("OriginatorReference").unwrap(),
            "originator_reference"
        );
        assert_eq!(
            bext.origination_date,
            FixedStr::<10>::from_str("2006/01/02").unwrap(),
            "origination_date"
        );
        assert_eq!(
            bext.origination_time,
            FixedStr::<8>::from_str("03:04:05").unwrap(),
            "origination_time"
        );
        assert_eq!(bext.time_reference, 12345, "time_reference");
        assert_eq!(bext.version, 2);
        assert_eq!(
            bext.umid,
            <Vec<u8> as TryInto<[u8; 64]>>::try_into(
                decode("060A2B3401010101010102101300000000FF122A6937058000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap()
            )
            .unwrap(),
            "version"
        );
        assert_eq!(bext.loudness_value, 100, "loudness_value");
        assert_eq!(bext.loudness_range, 200, "loudness_range");
        assert_eq!(bext.max_true_peak_level, 300, "max_true_peak_level");
        assert_eq!(bext.max_momentary_loudness, 400, "max_momentary_loudness");
        assert_eq!(bext.max_short_term_loudness, 500, "max_short_term_loudness");
        assert_eq!(bext.reserved.len(), 180, "reserved");
        assert_eq!(bext.coding_history, "CodingHistory", "coding_history");
    }

    #[test]
    fn decode_spaces() {
        let a = &decode("666D7420 10000000 01000100 80BB0000 80320200 03001800".replace(' ', ""))
            .unwrap();
        let b = &decode("666D7420100000000100010080BB00008032020003001800").unwrap();
        assert_eq!(a, b);
    }
}
