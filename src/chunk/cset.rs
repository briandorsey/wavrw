use std::{
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
    sync::OnceLock,
};

use binrw::binrw;

use crate::{FourCC, KnownChunk, KnownChunkID, Summarizable};

#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
pub struct CsetData {
    code_page: u16,
    country_code: CsetCountryCode,
    language: u16,
    dialect: u16,
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
                    format!("{language}({})", self.data.language.to_string()),
                ))
            }
            4 => {
                let (_, dialect) = cset_ld_map()
                    .get(&(self.data.language, self.data.dialect))
                    .unwrap_or(&("Unknown", "Unknown"));
                Some((
                    "dialect".to_string(),
                    format!("{dialect}({})", self.data.dialect.to_string()),
                ))
            }
            _ => None,
        }
    }
}

/// `Cset` (CSET) stores character set information. Defined in RIFF1991.
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

#[allow(dead_code)]
#[binrw]
#[brw(little, repr = u16)]
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RiffCountryCode {
    USA = 0x1,
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
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        use RiffCountryCode::*;
        let output = match self {
            USA => "USA",
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

#[binrw]
#[brw(little)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnknownCountryCode {
    country_code: u16,
}

impl Display for UnknownCountryCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown (0x{:x})", self.country_code)?;
        Ok(())
    }
}

#[allow(dead_code)]
#[binrw]
#[brw(little)]
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CsetCountryCode {
    Known(RiffCountryCode),
    Unknown(UnknownCountryCode),
}

impl Display for CsetCountryCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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

    #[test]
    fn unknowncountrycode() {
        let country = UnknownCountryCode {
            country_code: 0xFFFF,
        };
        assert_eq!(country.country_code, 0xFFFF);

        let mut buff = hex_to_cursor("FFFF");
        let unknown = UnknownCountryCode::read(&mut buff).unwrap();
        assert_eq!(
            unknown,
            UnknownCountryCode {
                country_code: 0xFFFF
            }
        );
    }

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
