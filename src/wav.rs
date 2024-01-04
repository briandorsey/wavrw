use binrw::{binrw, helpers, io::SeekFrom};
use std::cmp::min;
use std::ffi::CStr;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

// TODO: test  offset += chunk.chunk_size(); equals actual chunk_id locaiton
// TODO: ensure chunk sizes are always an even number, per RIFF specs. Probably use align_* args on bw attributes.
// consider refactoring Chunk to hold id, size and raw data, with enum for parsed data
// TODO: chunk_id -> id, chunk_size -> size

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
        write!(f, "FourCC({})", String::from_utf8_lossy(&self.0),)?;
        Ok(())
    }
}

#[derive(Debug)]
struct FixedStrErr;

#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
/// FixedStr holds Null terminated fixed length strings (from BEXT for example)
struct FixedStr<const N: usize>([u8; N]);

impl<const N: usize> Display for FixedStr<N> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        let s: String = {
            if N > 0 && self.0[N - 1] == 0u8 {
                // TODO: fix unwrap()
                let cstr = CStr::from_bytes_until_nul(&self.0).unwrap();
                cstr.to_string_lossy().to_string()
            } else {
                String::from_utf8_lossy(&self.0).to_string()
            }
        };
        write!(f, "{}", s)?;
        Ok(())
    }
}

impl<const N: usize> FromStr for FixedStr<N> {
    type Err = FixedStrErr;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        let mut array_tmp = [0u8; N];
        array_tmp[..min(s.len(), N)].copy_from_slice(&s.as_bytes()[..min(s.len(), N)]);
        Ok(FixedStr::<N>(array_tmp))
    }
}

// parsing structs
// ----

#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
// http://www.tactilemedia.com/info/MCI_Control_Info.html
pub struct Wav {
    pub chunk_id: FourCC,
    pub chunk_size: u32,
    pub form_type: FourCC,
    #[br(parse_with = helpers::until_eof)]
    pub chunks: Vec<Chunk>,
}

// based on http://soundfile.sapp.org/doc/WaveFormat/
#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
pub struct FmtChunk {
    #[brw(seek_before = SeekFrom::Current(-4))]
    chunk_id: FourCC,
    chunk_size: u32,
    audio_format: u16,
    num_channels: u16,
    sample_rate: u32,
    byte_rate: u32,
    block_align: u16,
    bits_per_sample: u16,
}

impl FmtChunk {
    pub fn summary(&self) -> String {
        format!(
            "{} chan, {}/{}",
            self.num_channels,
            self.bits_per_sample,
            self.sample_rate,
            // TODO: audio_format
        )
    }
}

#[binrw]
#[br(little)]
#[derive(Debug, PartialEq, Eq)]
pub struct ListChunk {
    #[brw(seek_before = SeekFrom::Current(-4))]
    chunk_id: FourCC,
    chunk_size: u32,
    list_type: FourCC,
    // need to add magic here to choose the right enum
    // items: ListType,
    #[br(count = chunk_size -4 )]
    #[bw()]
    raw: Vec<u8>,
}

impl ListChunk {
    pub fn summary(&self) -> String {
        format!("{}", self.list_type)
    }
}

// BEXT, based on https://tech.ebu.ch/docs/tech/tech3285.pdf
#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
pub struct BextChunk {
    #[brw(seek_before = SeekFrom::Current(-4))]
    chunk_id: FourCC,
    chunk_size: u32,
    // #[br(map = |x: [u8;256]| String::from_utf8_lossy(&x).to_string())]
    // #[bw(map = |x| {
    //     let mut array_tmp = [0u8; 256];
    //     array_tmp[..x.len()].copy_from_slice(x.as_bytes())
    // })]
    description: FixedStr<256>,
    #[br(count = chunk_size - 256 )]
    #[bw()]
    raw: Vec<u8>,
}

impl BextChunk {
    pub fn summary(&self) -> String {
        let s = format!("{}", self.description);
        format!("... BEXT\n{} {}", self.description, s.len())
    }
}

// based on https://mediaarea.net/BWFMetaEdit/md5
#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
pub struct Md5Chunk {
    #[brw(seek_before = SeekFrom::Current(-4))]
    chunk_id: FourCC,
    chunk_size: u32,
    md5: u128,
}

impl Md5Chunk {
    pub fn summary(&self) -> String {
        format!("MD5: {:X}", self.md5)
    }
}

#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
pub enum Chunk {
    // TODO: add DATA parsing which skips actual data
    #[brw(magic = b"fmt ")]
    Fmt(FmtChunk),
    #[brw(magic = b"LIST")]
    List(ListChunk),
    #[brw(magic = b"bext")]
    Bext(BextChunk),
    #[brw(magic = b"MD5 ")]
    Md5(Md5Chunk),
    Unknown {
        chunk_id: FourCC,
        chunk_size: u32,
        #[br(count = chunk_size )]
        #[bw()]
        raw: Vec<u8>,
    },
}

impl Chunk {
    pub fn chunk_id(&self) -> FourCC {
        match self {
            Chunk::Fmt(e) => e.chunk_id,
            Chunk::List(e) => e.chunk_id,
            Chunk::Bext(e) => e.chunk_id,
            Chunk::Md5(e) => e.chunk_id,
            Chunk::Unknown { chunk_id, .. } => *chunk_id,
        }
    }

    pub fn chunk_size(&self) -> u32 {
        match self {
            Chunk::Fmt(e) => e.chunk_size,
            Chunk::List(e) => e.chunk_size,
            Chunk::Bext(e) => e.chunk_size,
            Chunk::Md5(e) => e.chunk_size,
            Chunk::Unknown { chunk_size, .. } => *chunk_size,
        }
    }

    pub fn summary(&self) -> String {
        match self {
            Chunk::Fmt(e) => e.summary(),
            Chunk::List(e) => e.summary(),
            Chunk::Bext(e) => e.summary(),
            Chunk::Md5(e) => e.summary(),
            Chunk::Unknown { .. } => "...".to_owned(),
        }
    }
}

#[cfg(test)]
mod test {
    use binrw::BinRead; // don't understand why this is needed in this scope
    use std::io::Cursor;

    use super::*;
    use hex::decode;

    fn hex_to_cursor(input: &str) -> Cursor<Vec<u8>> {
        let data = decode(input.replace(' ', "")).unwrap();
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
        let wavfile = Wav::read(&mut data).unwrap();
        assert_eq!(
            Wav {
                chunk_id: FourCC(*b"RIFF"),
                chunk_size: 2398,
                form_type: FourCC(*b"WAVE"),
                chunks: vec![],
            },
            wavfile
        );
    }

    // #[test]
    // fn parse_chunk_length() {
    //     let tests = [(
    //         &decode("666D7420 10000000 01000100 80BB0000 80320200 03001800".replace(' ', ""))
    //             .unwrap(),
    //         UnknownChunk {
    //             chunk_id: "fmt ".as_bytes().try_into().unwrap(),
    //             chunk_size: 16,
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
            Wav {
                chunk_id: FourCC(*b"RIFF"),
                chunk_size: 2398,
                form_type: FourCC(*b"WAVE"),
                chunks: vec![Chunk::Fmt(FmtChunk {
                    chunk_id: FourCC(*b"fmt "),
                    chunk_size: 16,
                    audio_format: 1,
                    num_channels: 1,
                    sample_rate: 48000,
                    byte_rate: 144000,
                    block_align: 3,
                    bits_per_sample: 24,
                })],
            },
        )];
        for (mut input, expected_output) in tests {
            // hexdump(input);
            let output = Wav::read(&mut input).expect("error parsing wav");
            assert_eq!(expected_output, output);
            // hexdump(remaining_input);
        }
    }

    #[test]
    fn decode_spaces() {
        let a = &decode("666D7420 10000000 01000100 80BB0000 80320200 03001800".replace(' ', ""))
            .unwrap();
        let b = &decode("666D7420100000000100010080BB00008032020003001800").unwrap();
        assert_eq!(a, b);
    }
}
