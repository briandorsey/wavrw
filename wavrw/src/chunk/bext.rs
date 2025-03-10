//! `bext` Broadcast Extension for motion picture, radio and television production. [BEXT1996](https://wavref.til.cafe/spec/bext1996/)

use core::fmt::Debug;

use binrw::{binrw, helpers};

use crate::{FourCC, KnownChunk, KnownChunkID, Summarizable, fixedstring::FixedString};

// BEXT, based on https://tech.ebu.ch/docs/tech/tech3285.pdf
// BEXT is specified to use ASCII for strings, but we're parsing it as utf8,
// since that is a superset of ASCII and many WAV files contain utf8 strings
// in `bext` chunks.
//
// Technically, this struct implements a BWF Version 2 parser. Previous versions
// are specified to always pad extra data with NULL bytes, so loudness fields
// added in V2 will usually default to 0s when reading a V1 or V0 chunk.
// However, it is possible to have incorrect data in those fields when `version`
// field is less than 2.

/// `bext` Broadcast Extension for motion picture, radio and television production. [BEXT1996](https://wavref.til.cafe/spec/bext1996/)
#[binrw]
#[brw(little)]
#[br(import(_size: u32))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Bext {
    /// Description of the sound sequence
    pub description: FixedString<256>, // Description
    /// Name of the originator
    pub originator: FixedString<32>, // Originator
    /// Reference of the originator
    pub originator_reference: FixedString<32>, // OriginatorReference
    /// yyyy:mm:dd
    pub origination_date: FixedString<10>, // OriginationDate
    /// hh:mm:ss
    pub origination_time: FixedString<8>, // OriginationTime
    /// First sample count since midnight
    pub time_reference: u64, // TimeReference
    /// Version of the BWF; unsigned binary number
    pub version: u16, // Version
    /// SMPTE UMID, raw unparsed data
    pub umid: [u8; 64], // UMID
    /// Integrated Loudness Value of the file in LUFS (multiplied by 100)
    pub loudness_value: i16, // LoudnessValue
    /// Integrated Loudness Range of the file in LUFS (multiplied by 100)
    pub loudness_range: i16, // LoudnessRange
    /// Maximum True Peak Level of the file expressed as dBTP (multiplied by 100)
    pub max_true_peak_level: i16, // MaxTruePeakLevel
    /// Highest value of the Momentary Loudness Level of the file in LUFS (multiplied by 100)
    pub max_momentary_loudness: i16, // MaxMomentaryLoudness
    /// Highest value of the Short-Term Loudness Level of the file in LUFS (multiplied by 100)
    pub max_short_term_loudness: i16, // MaxShortTermLoudness
    /// 180 bytes, reserved for future use, set to “NULL”
    pub reserved: [u8; 180], // Reserved

    // Interpret the remaining bytes as string, ignoring any trailing \x00 bytes
    // Some recorders write `bext` chunks with trailing 0x0 padding. Perhaps to
    // allow later writing coding_history data without needing move chunks?
    #[br(parse_with = helpers::until_eof,
        try_map = |v: Vec<u8>| {
            match String::from_utf8(v) {
                Ok(s) =>  Ok(s.trim_end_matches('\0').to_string()),
                Err(e) => Err(e),
            }
        })]
    #[bw(map = |s: &String| s.as_bytes())]
    /// History coding
    pub coding_history: String, // CodingHistory
}

impl KnownChunkID for Bext {
    const ID: FourCC = FourCC(*b"bext");
}

impl Bext {
    fn new() -> Bext {
        Bext {
            description: FixedString::<256>::default(),
            originator: FixedString::<32>::default(),
            originator_reference: FixedString::<32>::default(),
            origination_date: FixedString::<10>::default(),
            origination_time: FixedString::<8>::default(),
            time_reference: 0,
            version: 0,
            umid: [0_u8; 64],
            loudness_value: 0,
            loudness_range: 0,
            max_true_peak_level: 0,
            max_momentary_loudness: 0,
            max_short_term_loudness: 0,
            reserved: [0u8; 180],
            coding_history: String::new(),
        }
    }
}

impl Default for Bext {
    fn default() -> Self {
        Bext::new()
    }
}

/// `bext` Broadcast Extension for motion picture, radio and television production. [BEXT1996](https://wavref.til.cafe/spec/bext1996/)
pub type BextChunk = KnownChunk<Bext>;

impl Summarizable for Bext {
    fn summary(&self) -> String {
        format!(
            "{}, {}, {}",
            self.origination_date, self.origination_time, self.description
        )
    }

    fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        Box::new(self.into_iter())
    }

    fn item_summary_header(&self) -> String {
        String::new()
    }
}

impl<'a> IntoIterator for &'a Bext {
    type Item = (String, String);
    type IntoIter = BextDataIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BextDataIterator {
            data: self,
            index: 0,
        }
    }
}

/// Iterate over fields as tuple of Strings (name, value).
#[derive(Debug)]
pub struct BextDataIterator<'a> {
    data: &'a Bext,
    index: usize,
}

impl Iterator for BextDataIterator<'_> {
    type Item = (String, String);
    fn next(&mut self) -> Option<(String, String)> {
        self.index += 1;
        match self.index {
            1 => Some(("description".to_string(), self.data.description.to_string())),
            2 => Some(("originator".to_string(), self.data.originator.to_string())),
            3 => Some((
                "originator_reference".to_string(),
                self.data.originator_reference.to_string(),
            )),
            4 => Some((
                "origination_date".to_string(),
                self.data.origination_date.to_string(),
            )),
            5 => Some((
                "origination_time".to_string(),
                self.data.origination_time.to_string(),
            )),
            6 => Some((
                "time_reference".to_string(),
                self.data.time_reference.to_string(),
            )),
            7 => Some(("version".to_string(), self.data.version.to_string())),
            8 => Some(("umid".to_string(), hex::encode(self.data.umid))),
            9 => Some((
                "loudness_value".to_string(),
                self.data.loudness_value.to_string(),
            )),
            10 => Some((
                "loudness_range".to_string(),
                self.data.loudness_range.to_string(),
            )),
            11 => Some((
                "max_true_peak_level".to_string(),
                self.data.max_true_peak_level.to_string(),
            )),
            12 => Some((
                "max_momentary_loudness".to_string(),
                self.data.max_momentary_loudness.to_string(),
            )),
            13 => Some((
                "max_short_term_loudness".to_string(),
                self.data.max_short_term_loudness.to_string(),
            )),
            14 => Some((
                "coding_history".to_string(),
                self.data.coding_history.clone(),
            )),
            _ => None,
        }
    }
}

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {
    use binrw::BinRead;
    use core::str::FromStr;
    use hex::decode;

    use super::*;
    use crate::testing::hex_to_cursor;

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
        let bext = BextChunk::read(&mut buff).expect("error parsing bext chunk");
        print!("{:?}", bext);
        assert_eq!(
            bext.data.description,
            FixedString::<256>::from_str("Description").unwrap(),
            "description"
        );
        assert_eq!(
            bext.data.originator,
            FixedString::<32>::from_str("Originator").unwrap(),
            "originator"
        );
        assert_eq!(
            bext.data.originator_reference,
            FixedString::<32>::from_str("OriginatorReference").unwrap(),
            "originator_reference"
        );
        assert_eq!(
            bext.data.origination_date,
            FixedString::<10>::from_str("2006/01/02").unwrap(),
            "origination_date"
        );
        assert_eq!(
            bext.data.origination_time,
            FixedString::<8>::from_str("03:04:05").unwrap(),
            "origination_time"
        );
        assert_eq!(bext.data.time_reference, 12345, "time_reference");
        assert_eq!(bext.data.version, 2);
        assert_eq!(
            bext.data.umid,
            <Vec<u8> as TryInto<[u8; 64]>>::try_into(
                decode("060A2B3401010101010102101300000000FF122A6937058000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap()
            )
            .unwrap(),
            "version"
        );
        assert_eq!(bext.data.loudness_value, 100, "loudness_value");
        assert_eq!(bext.data.loudness_range, 200, "loudness_range");
        assert_eq!(bext.data.max_true_peak_level, 300, "max_true_peak_level");
        assert_eq!(
            bext.data.max_momentary_loudness, 400,
            "max_momentary_loudness"
        );
        assert_eq!(
            bext.data.max_short_term_loudness, 500,
            "max_short_term_loudness"
        );
        assert_eq!(bext.data.reserved.len(), 180, "reserved");
        assert_eq!(bext.data.coding_history, "CodingHistory", "coding_history");
    }

    #[test]
    fn parse_h1e_coding_history() {
        // bext chunk from an h1e recorder. Extra \x0 data at the end of coding history
        let mut buff = hex_to_cursor(
            r#"62657874 5A030000 0D0A0D0A 7A534345 4E453D32 34303330 322D3230
            35353236 0D0A7A54 524B313D 54724D69 634C0D0A 7A54524B 323D5472 4D696352 0D0A7A4E
            4F54453D 0D0A0000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
            00000000 00000000 00000000 00000000 00000000 5A4F4F4D 20483165 7373656E 7469616C
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
            00000000 00000000 00000000 32303234 2D30332D 30323230 3A35353A 323600D2 04AF0100
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
            00000000 0000413D 50434D2C 463D3936 3030302C 573D3332 2C4D3D73 74657265 6F2C543D
            5A4F4F4D 20483165 7373656E 7469616C 3B564552 53494F4E 3D312E31 303B0000 00000000
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
            00000000 00000000 0000"#,
        );
        let bext = BextChunk::read(&mut buff).expect("error parsing bext chunk");
        print!("{:?}", bext);
        assert_eq!(
            bext.data.coding_history,
            "A=PCM,F=96000,W=32,M=stereo,T=ZOOM H1essential;VERSION=1.10;",
            "coding_history"
        );
    }
}
