use binrw::binrw;
use binrw::io::SeekFrom;

use crate::{FourCC, KnownChunk, KnownChunkID, Summarizable};

/// `data` Audio samples. This parser skips all audio data (for now). [RIFF1991](https://wavref.til.cafe/spec/riff1991/)
#[binrw]
#[brw(little)]
#[br(import(size: u32))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Data {
    // Not public until we figure out design for loading data. (issue #72)
    //
    // For some reason SeekFrom::End(0) takes us beyond this chunk, even though
    // KnownChunk uses take_seek to limit the reader. I don't know why.
    // Instead, we're passing in the chunk size and using that.
    // The reason for this dance is to seek past the (often very large)
    // audio data entirely, so we don't read it into KnownChunk.extra_bytes.
    #[br(count = 0, seek_before(SeekFrom::Current(size.into())))]
    #[bw()]
    pub(crate) data: Vec<u8>,
}

impl KnownChunkID for Data {
    const ID: FourCC = FourCC(*b"data");
}

impl Summarizable for Data {
    fn summary(&self) -> String {
        "audio data".to_string()
    }
}

/// `data` Audio samples. This parser skips all audio data (for now). [RIFF1991](https://wavref.til.cafe/spec/riff1991/)
pub type DataChunk = KnownChunk<Data>;

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {
    use binrw::{BinRead, BinWrite};
    use hexdump::hexdump;

    use super::*;

    #[test]
    fn data_roundtrip() {
        // validate inner DataData roundtrip. Only works for 0 length since data
        // chunks aren't read yet.
        let dd = Data {
            data: [8_u8; 0].to_vec(),
        };
        let mut buff = std::io::Cursor::new(Vec::<u8>::new());
        dd.write(&mut buff).unwrap();
        println!("{:?}", hexdump(buff.get_ref()));
        buff.set_position(0);
        let after = Data::read(&mut buff).unwrap();
        assert_eq!(after, dd,);

        // validate data roundtrip
        let data = DataChunk {
            offset: Some(0),
            size: 0,
            data: Data {
                data: [8_u8; 0].to_vec(),
            },
            extra_bytes: Vec::new(),
        };
        let mut buff = std::io::Cursor::new(Vec::<u8>::new());
        data.write(&mut buff).unwrap();
        println!("{:?}", hexdump(buff.get_ref()));
        buff.set_position(0);
        let after = DataChunk::read(&mut buff).unwrap();
        assert_eq!(after, data);
        println!("length of data as bytes: {}", buff.into_inner().len());
    }
}
