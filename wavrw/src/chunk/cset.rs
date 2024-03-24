use core::fmt::{Debug, Display, Formatter};
use std::collections::HashMap;
use std::sync::OnceLock;

use binrw::binrw;

use crate::{FourCC, KnownChunk, KnownChunkID, Summarizable};

#[binrw]
#[brw(little)]
#[br(import(_size: u32))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
/// `CSET` Character set information. Code page, language, etc. Very Rare. [RIFF1991](https://wavref.til.cafe/chunk/cset/)
pub struct CsetData {
    /// Specifies the code page used for file elements.
    ///
    /// If the CSET chunk is not present, or if this field has value zero, assume
    /// standard ISO 8859/1 code page (identical to code page 1004 without code
    /// points defined in hex columns 0, 1, 8, and 9).
    pub code_page: u16,

    /// Specifies the country code used for file elements.
    ///
    /// See [`RiffCountryCode`] for a list of currently
    /// defined country codes. If the CSET chunk is not present, or if this
    /// field has value zero, assume USA (country code 001).
    pub country_code: CsetCountryCode,

    /// Specify the language and dialect used for file elements.
    ///
    /// See cset_ld_map, for a list of language and dialect codes. If the CSET
    /// chunk is not present, or if these fields have value zero, assume US
    /// English (language code 9, dialect code 1).
    pub language: u16,

    /// Specify the language and dialect used for file elements.
    ///
    /// See cset_ld_map, for a list of language and dialect codes. If the CSET
    /// chunk is not present, or if these fields have value zero, assume US
    /// English (language code 9, dialect code 1).
    pub dialect: u16,
}

impl KnownChunkID for CsetData {
    const ID: FourCC = FourCC(*b"CSET");
}

impl Summarizable for CsetData {
    fn summary(&self) -> String {
        let (language, dialect) = cset_ld_map()
            .get(&(self.language, self.dialect))
            .unwrap_or(&("Unknown", "Unknown"));
        format!(
            "code_page: ({}), {}, {language}({}), {dialect}({})",
            self.code_page, self.country_code, self.language, self.dialect,
        )
    }

    fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        Box::new(self.into_iter())
    }
}

// Iteration based on pattern from https://stackoverflow.com/questions/30218886/how-to-implement-iterator-and-intoiterator-for-a-simple-struct

impl<'a> IntoIterator for &'a CsetData {
    type Item = (String, String);
    type IntoIter = CsetDataIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        CsetDataIterator {
            data: self,
            index: 0,
        }
    }
}

#[derive(Debug)]
pub struct CsetDataIterator<'a> {
    data: &'a CsetData,
    index: usize,
}

impl<'a> Iterator for CsetDataIterator<'a> {
    type Item = (String, String);
    fn next(&mut self) -> Option<(String, String)> {
        self.index += 1;
        match self.index {
            1 => Some(("code_page".to_string(), self.data.code_page.to_string())),
            2 => Some((
                "country_code".to_string(),
                self.data.country_code.to_string(),
            )),
            3 => {
                let (language, _) = cset_ld_map()
                    .get(&(self.data.language, self.data.dialect))
                    .unwrap_or(&("Unknown", "Unknown"));
                Some((
                    "language".to_string(),
                    format!("{language}({})", self.data.language),
                ))
            }
            4 => {
                let (_, dialect) = cset_ld_map()
                    .get(&(self.data.language, self.data.dialect))
                    .unwrap_or(&("Unknown", "Unknown"));
                Some((
                    "dialect".to_string(),
                    format!("{dialect}({})", self.data.dialect),
                ))
            }
            _ => None,
        }
    }
}

/// `CSET` Character set information. Code page, language, etc. Very Rare. [RIFF1991](https://wavref.til.cafe/chunk/cset/)
///
/// NOTE: Implemented from the spec only, because I couldn't find any files actually
/// containing this chunk.
pub type Cset = KnownChunk<CsetData>;

#[allow(clippy::type_complexity)]
fn cset_ld_map() -> &'static HashMap<(u16, u16), (&'static str, &'static str)> {
    static MAP: OnceLock<HashMap<(u16, u16), (&'static str, &'static str)>> = OnceLock::new();
    MAP.get_or_init(|| {
        HashMap::from([
            ((0, 0), ("None", "")),
            ((1, 1), ("Arabic", "")),
            ((2, 1), ("Bulgarian", "")),
            ((3, 1), ("Catalan", "")),
            ((4, 1), ("Chinese", "Traditional")),
            ((4, 2), ("Chinese", "Simplified")),
            ((5, 1), ("Czech", "")),
            ((6, 1), ("Danish", "")),
            ((7, 1), ("German", "")),
            ((7, 2), ("German", "Swiss")),
            ((8, 1), ("Greek", "")),
            ((9, 1), ("English", "US")),
            ((9, 2), ("English", "UK")),
            ((10, 1), ("Spanish", "")),
            ((10, 2), ("Spanish", "Mexican")),
            ((11, 1), ("Finnish", "")),
            ((12, 1), ("French", "")),
            ((12, 2), ("French", "Belgian")),
            ((12, 3), ("French", "Canadian")),
            ((12, 4), ("French", "Swiss")),
            ((13, 1), ("Hebrew", "")),
        ])
    })
}

/// The country codes specified in [RIFF1991](https://wavref.til.cafe/chunk/cset/)
#[allow(dead_code, missing_docs)]
#[binrw]
#[brw(little, repr = u16)]
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RiffCountryCode {
    None = 0x0,
    UnitedStates = 0x1,
    Canada = 0x2,
    LatinAmerica = 0x3,
    Greece = 0x30,
    Netherlands = 0x31,
    Belgium = 0x32,
    France = 0x33,
    Spain = 0x34,
    Italy = 0x39,
    Switzerland = 0x41,
    Austria = 0x43,
    UnitedKingdom = 0x44,
    Denmark = 0x45,
    Sweden = 0x46,
    Norway = 0x47,
    WestGermany = 0x49,
    Mexico = 0x52,
    Brazil = 0x55,
    Australia = 0x61,
    NewZealand = 0x64,
    Japan = 0x81,
    Korea = 0x82,
    PeoplesRepublicOfChina = 0x86,
    Taiwan = 0x88,
    Turkey = 0x90,
    Portugal = 0x351,
    Luxembourg = 0x352,
    Iceland = 0x354,
    Finland = 0x358,
}

#[allow(clippy::enum_glob_use)]
impl Display for RiffCountryCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        use RiffCountryCode::*;
        let output = match self {
            None => "None",
            UnitedStates => "United States of America",
            Canada => "Canada",
            LatinAmerica => "Latin America",
            Greece => "Greece",
            Netherlands => "Netherlands",
            Belgium => "Belgium",
            France => "France",
            Spain => "Spain",
            Italy => "Italy",
            Switzerland => "Switzerland",
            Austria => "Austria",
            UnitedKingdom => "United Kingdom",
            Denmark => "Denmark",
            Sweden => "Sweden",
            Norway => "Norway",
            WestGermany => "West Germany",
            Mexico => "Mexico",
            Brazil => "Brazil",
            Australia => "Australia",
            NewZealand => "New Zealand",
            Japan => "Japan",
            Korea => "Korea",
            PeoplesRepublicOfChina => "People’s Republic of China",
            Taiwan => "Taiwan",
            Turkey => "Turkey",
            Portugal => "Portugal",
            Luxembourg => "Luxembourg",
            Iceland => "Iceland",
            Finland => "Finland",
        };
        write!(f, "{}({})", output, *self as u16)?;
        Ok(())
    }
}

/// Store any country code found outside of [`RiffCountryCode`].
#[binrw]
#[brw(little)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UnknownCountryCode(u16);

impl Display for UnknownCountryCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "Unknown (0x{:x})", self.0)?;
        Ok(())
    }
}

impl From<u16> for UnknownCountryCode {
    fn from(value: u16) -> Self {
        UnknownCountryCode(value)
    }
}

impl From<UnknownCountryCode> for u16 {
    fn from(value: UnknownCountryCode) -> Self {
        value.0
    }
}

/// Country code enum combining specified and unknown values.
///
/// This is to handle values outside of the specification, which are assumed to be
/// in at least some files.
#[allow(dead_code, missing_docs)]
#[binrw]
#[brw(little)]
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CsetCountryCode {
    Known(RiffCountryCode),
    Unknown(UnknownCountryCode),
}

impl Display for CsetCountryCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CsetCountryCode::Known(c) => c.to_string(),
                CsetCountryCode::Unknown(c) => c.to_string(),
            }
        )?;
        Ok(())
    }
}

// impl CsetCountryCode {
//     // Constructor depends on try_from, TODO: #86
//     pub fn new(value: u16) -> CsetCountryCode {
//         RiffCountryCode::try_from(value)
//     }
// }

impl Default for CsetCountryCode {
    fn default() -> Self {
        CsetCountryCode::Known(RiffCountryCode::None)
    }
}

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {
    use binrw::{BinRead, BinWrite};
    use hexdump::hexdump;

    use super::*;
    use crate::testing::hex_to_cursor;

    #[test]
    fn riffcountrycode() {
        let country = RiffCountryCode::Canada;
        assert_eq!(country as u16, 2);

        let mut buff = hex_to_cursor("0200");
        let canada: RiffCountryCode = RiffCountryCode::read(&mut buff).unwrap();
        assert_eq!(canada, RiffCountryCode::Canada);
    }

    // TODO: implement try_from() primitive for enums #86
    // #[test]
    // fn riffcountrycode_from_primitive() {
    //     let canada = RiffCountryCode::try_from(2_u16);
    //     assert_eq!(canada, Ok(RiffCountryCode::Canada));

    //     let other = RiffCountryCode::try_from(4242_u16);
    //     assert_eq!(other, Ok(RiffCountryCode::Other(4242_u16)));
    // }

    #[test]
    fn unknowncountrycode() {
        let country = UnknownCountryCode(0xFFFF_u16);
        let raw_country: u16 = country.into();
        assert_eq!(raw_country, 0xFFFF_u16);

        let mut buff = hex_to_cursor("FFFF");
        let unknown = UnknownCountryCode::read(&mut buff).unwrap();
        assert_eq!(unknown, UnknownCountryCode(0xFFFF_u16));
    }

    // couldn't find CSET usage in file collection, so just doing a roundtrip test
    #[test]
    fn cset_roundtrip() {
        let cset = Cset {
            size: 8,
            data: CsetData {
                code_page: 1,
                country_code: CsetCountryCode::Known(RiffCountryCode::Canada),
                language: 12,
                dialect: 3,
            },
            extra_bytes: vec![],
        };
        println!("{cset:?}");
        let mut buff = std::io::Cursor::new(Vec::<u8>::new());
        cset.write(&mut buff).unwrap();
        println!("{:?}", hexdump(buff.get_ref()));
        buff.set_position(0);
        let after = Cset::read(&mut buff).unwrap();
        assert_eq!(after, cset);
        assert_eq!(after.data.code_page, 1);
        assert_eq!(
            after.data.country_code,
            CsetCountryCode::Known(RiffCountryCode::Canada)
        );
        assert_eq!(after.data.language, 12);
        assert_eq!(after.data.dialect, 3);
        assert_eq!(
            after.summary(),
            "code_page: (1), Canada(2), French(12), Canadian(3)"
        );
    }
}
