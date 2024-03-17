use std::fmt::Debug;

use binrw::{binrw, helpers};

use crate::{FixedStr, FourCC, KnownChunk, KnownChunkID, Summarizable};

// BEXT, based on https://tech.ebu.ch/docs/tech/tech3285.pdf
// BEXT is specified to use ASCII, but we're parsing it as utf8, since
// that is a superset of ASCII and many WAV files contain utf8 strings here
#[binrw]
#[brw(little)]
#[br(import(_size: u32))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BextData {
    /// Description of the sound sequence
    pub description: FixedStr<256>, // Description
    /// Name of the originator
    pub originator: FixedStr<32>, // Originator
    /// Reference of the originator
    pub originator_reference: FixedStr<32>, // OriginatorReference
    /// yyyy:mm:dd
    pub origination_date: FixedStr<10>, // OriginationDate
    /// hh:mm:ss
    pub origination_time: FixedStr<8>, // OriginationTime
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
    /// History coding
    // interpret the remaining bytes as string
    #[br(parse_with = helpers::until_eof, map = |v: Vec<u8>| String::from_utf8_lossy(&v).to_string())]
    #[bw(map = |s: &String| s.as_bytes())]
    pub coding_history: String, // CodingHistory
}

impl KnownChunkID for BextData {
    const ID: FourCC = FourCC(*b"bext");
}

impl BextData {
    fn new() -> BextData {
        BextData {
            description: FixedStr::<256>::new(),
            originator: FixedStr::<32>::new(),
            originator_reference: FixedStr::<32>::new(),
            origination_date: FixedStr::<10>::new(),
            origination_time: FixedStr::<8>::new(),
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

impl Default for BextData {
    fn default() -> Self {
        BextData::new()
    }
}

pub type Bext = KnownChunk<BextData>;

impl Summarizable for BextData {
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

impl<'a> IntoIterator for &'a BextData {
    type Item = (String, String);
    type IntoIter = BextDataIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BextDataIterator {
            data: self,
            index: 0,
        }
    }
}

#[derive(Debug)]
pub struct BextDataIterator<'a> {
    data: &'a BextData,
    index: usize,
}

impl<'a> Iterator for BextDataIterator<'a> {
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
    use hex::decode;
    use std::str::FromStr;

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
        let bext = Bext::read(&mut buff).expect("error parsing bext chunk");
        print!("{:?}", bext);
        assert_eq!(
            bext.data.description,
            FixedStr::<256>::from_str("Description").unwrap(),
            "description"
        );
        assert_eq!(
            bext.data.originator,
            FixedStr::<32>::from_str("Originator").unwrap(),
            "originator"
        );
        assert_eq!(
            bext.data.originator_reference,
            FixedStr::<32>::from_str("OriginatorReference").unwrap(),
            "originator_reference"
        );
        assert_eq!(
            bext.data.origination_date,
            FixedStr::<10>::from_str("2006/01/02").unwrap(),
            "origination_date"
        );
        assert_eq!(
            bext.data.origination_time,
            FixedStr::<8>::from_str("03:04:05").unwrap(),
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
}
