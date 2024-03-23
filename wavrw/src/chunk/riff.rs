use core::fmt::Display;

use binrw::binrw;

use crate::FourCC;

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Riff {
    pub id: FourCC,
    pub size: u32,
    pub form_type: FourCC,
}

impl Display for Riff {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for Riff {
    fn default() -> Self {
        Riff {
            id: FourCC(*b"RIFF"),
            size: 0,
            form_type: FourCC(*b"WAVE"),
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
    fn riff_header() {
        // RIFF 2398 WAVE
        let header = "524946465E09000057415645";
        let mut data = hex_to_cursor(header);
        println!("{header:?}");
        let wavfile = Riff::read(&mut data).unwrap();
        assert_eq!(
            wavfile,
            Riff {
                id: FourCC(*b"RIFF"),
                size: 2398,
                form_type: FourCC(*b"WAVE"),
            }
        );
    }

    #[test]
    fn parse_riff_large() {
        // example large chunk size, double checking endian
        let header = r#"52494646 0000FEFF 57415645 00000000"#;
        let mut buff = hex_to_cursor(header);
        println!("{header:?}");
        let riff = Riff::read(&mut buff).expect("error parsing large data chunk");
        print!("{:?}", riff);
        assert_eq!(
            riff,
            Riff {
                id: FourCC(*b"RIFF"),
                size: 4294836224,
                form_type: FourCC(*b"WAVE"),
            }
        );
    }
}
