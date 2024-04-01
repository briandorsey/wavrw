use core::fmt::{Debug, Display, Formatter};
use std::collections::HashMap;
use std::sync::OnceLock;

use binrw::binrw;
use num_enum::{FromPrimitive, IntoPrimitive};

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
    pub country_code: RiffCountryCode,

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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, IntoPrimitive, FromPrimitive)]
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
    #[num_enum(catch_all)]
    Unknown(u16),
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
            Unknown(_) => "Unknown Country Code",
        };
        write!(f, "{}({})", output, u16::from(*self))?;
        Ok(())
    }
}

// num_enum: Attribute `catch_all` is mutually exclusive with `default`
#[allow(clippy::derivable_impls)]
impl Default for RiffCountryCode {
    fn default() -> Self {
        RiffCountryCode::None
    }
}

impl TryFrom<&RiffCountryCode> for u16 {
    // infalible, but binrw seems to need TryFrom?
    type Error = binrw::io::Error;

    fn try_from(value: &RiffCountryCode) -> Result<Self, Self::Error> {
        Ok(u16::from(*value))
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
        assert_eq!(u16::from(country), 2);

        let mut buff = hex_to_cursor("0200");
        let canada: RiffCountryCode = RiffCountryCode::read(&mut buff).unwrap();
        assert_eq!(canada, RiffCountryCode::Canada);
    }

    // couldn't find CSET usage in file collection, so just doing a roundtrip test
    #[test]
    fn cset_roundtrip() {
        let cset = Cset {
            offset: Some(0),
            size: 8,
            data: CsetData {
                code_page: 1,
                country_code: RiffCountryCode::Canada,
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
        assert_eq!(after.data.country_code, RiffCountryCode::Canada);
        assert_eq!(after.data.language, 12);
        assert_eq!(after.data.dialect, 3);
        assert_eq!(
            after.summary(),
            "code_page: (1), Canada(2), French(12), Canadian(3)"
        );
    }

    #[test]
    fn cset_primitive() {
        let none = RiffCountryCode::from(0u16);
        assert_eq!(none, RiffCountryCode::None);

        let unknown = RiffCountryCode::from(4242u16);
        assert_eq!(unknown, RiffCountryCode::Unknown(4242_u16));
        assert_eq!(unknown.to_string(), "Unknown Country Code(4242)");

        // make sure it works via BinRead as well
        let mut buff = hex_to_cursor("0042");
        let unknown: RiffCountryCode = RiffCountryCode::read(&mut buff).unwrap();
        assert_eq!(unknown, RiffCountryCode::Unknown(0x4200));
    }
}
