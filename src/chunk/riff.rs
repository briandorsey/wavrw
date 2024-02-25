use binrw::binrw;

use crate::FourCC;

#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
pub struct RiffChunk {
    pub id: FourCC,
    pub size: u32,
    pub form_type: FourCC,
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
        let wavfile = RiffChunk::read(&mut data).unwrap();
        assert_eq!(
            wavfile,
            RiffChunk {
                id: FourCC(*b"RIFF"),
                size: 2398,
                form_type: FourCC(*b"WAVE"),
            }
        );
    }
}
