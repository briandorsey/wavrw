use std::fmt::{Debug, Formatter};

use binrw::{binrw, helpers, NullString};
use itertools::Itertools;

use crate::{fourcc, ChunkID, FourCC, KnownChunk, KnownChunkID, Summarizable};

#[binrw]
#[br(little)]
#[derive(Debug, PartialEq, Eq)]
pub struct ListInfoChunkData {
    #[brw(assert(list_type == ListInfoChunkData::LIST_TYPE))]
    pub list_type: FourCC,
    #[br(parse_with = helpers::until_eof)]
    #[bw()]
    pub chunks: Vec<InfoChunkEnum>,
}

impl ListInfoChunkData {
    pub const LIST_TYPE: FourCC = FourCC(*b"INFO");
}

impl KnownChunkID for ListInfoChunkData {
    const ID: FourCC = FourCC(*b"LIST");
}

impl Summarizable for ListInfoChunkData {
    fn summary(&self) -> String {
        format!(
            "{}: {}",
            self.list_type,
            self.chunks.iter().map(|c| c.id()).join(", ")
        )
    }

    fn name(&self) -> String {
        self.list_type.to_string().trim().to_string()
    }

    fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        Box::new(self.chunks.iter().map(|c| (c.id().to_string(), c.value())))
    }
}

pub type ListInfoChunk = KnownChunk<ListInfoChunkData>;

/// InfoChunkData is a genericised container for LIST INFO chunks
///
/// A type alias is devfined for all of the INFO types from the initial 1991
/// WAV spec.
///
/// # Examples:
///
/// ```
/// # use wavrw::chunk::info::IcmtChunkData;
/// let icmt = IcmtChunkData::new("comment");
/// ```
///
#[binrw]
#[br(little)]
#[derive(PartialEq, Eq)]
pub struct InfoChunkData<const I: u32> {
    pub value: NullString,
}

impl<const I: u32> KnownChunkID for InfoChunkData<I> {
    const ID: FourCC = FourCC(I.to_le_bytes());
}

impl<const I: u32> Debug for InfoChunkData<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "InfoChunkData<{}> {{ value: {:?} }}",
            String::from_utf8_lossy(I.to_le_bytes().as_slice()),
            self.value,
        )?;
        Ok(())
    }
}

impl<const I: u32> Summarizable for InfoChunkData<I> {
    fn summary(&self) -> String {
        self.value.to_string()
    }
}

impl<const I: u32> InfoChunkData<I> {
    pub fn new(value: &str) -> Self {
        InfoChunkData::<I> {
            value: value.into(),
        }
    }
}

pub type IarlChunkData = InfoChunkData<{ fourcc(b"IARL") }>;
pub type IgnrChunkData = InfoChunkData<{ fourcc(b"IGNR") }>;
pub type IkeyChunkData = InfoChunkData<{ fourcc(b"IKEY") }>;
pub type IlgtChunkData = InfoChunkData<{ fourcc(b"ILGT") }>;
pub type ImedChunkData = InfoChunkData<{ fourcc(b"IMED") }>;
pub type InamChunkData = InfoChunkData<{ fourcc(b"INAM") }>;
pub type IpltChunkData = InfoChunkData<{ fourcc(b"IPLT") }>;
pub type IprdChunkData = InfoChunkData<{ fourcc(b"IPRD") }>;
pub type IsbjChunkData = InfoChunkData<{ fourcc(b"ISBJ") }>;
pub type IsftChunkData = InfoChunkData<{ fourcc(b"ISFT") }>;
pub type IshpChunkData = InfoChunkData<{ fourcc(b"ISHP") }>;
pub type IartChunkData = InfoChunkData<{ fourcc(b"IART") }>;
pub type IsrcChunkData = InfoChunkData<{ fourcc(b"ISRC") }>;
pub type IsrfChunkData = InfoChunkData<{ fourcc(b"ISRF") }>;
pub type ItchChunkData = InfoChunkData<{ fourcc(b"ITCH") }>;
pub type IcmsChunkData = InfoChunkData<{ fourcc(b"ICMS") }>;
pub type IcmtChunkData = InfoChunkData<{ fourcc(b"ICMT") }>;
pub type IcopChunkData = InfoChunkData<{ fourcc(b"ICOP") }>;
pub type IcrdChunkData = InfoChunkData<{ fourcc(b"ICRD") }>;
pub type IcrpChunkData = InfoChunkData<{ fourcc(b"ICRP") }>;
pub type IdpiChunkData = InfoChunkData<{ fourcc(b"IDPI") }>;
pub type IengChunkData = InfoChunkData<{ fourcc(b"IENG") }>;

pub type IarlChunk = KnownChunk<IarlChunkData>;
pub type IgnrChunk = KnownChunk<IgnrChunkData>;
pub type IkeyChunk = KnownChunk<IkeyChunkData>;
pub type IlgtChunk = KnownChunk<IlgtChunkData>;
pub type ImedChunk = KnownChunk<ImedChunkData>;
pub type InamChunk = KnownChunk<InamChunkData>;
pub type IpltChunk = KnownChunk<IpltChunkData>;
pub type IprdChunk = KnownChunk<IprdChunkData>;
pub type IsbjChunk = KnownChunk<IsbjChunkData>;
pub type IsftChunk = KnownChunk<IsftChunkData>;
pub type IshpChunk = KnownChunk<IshpChunkData>;
pub type IartChunk = KnownChunk<IartChunkData>;
pub type IsrcChunk = KnownChunk<IsrcChunkData>;
pub type IsrfChunk = KnownChunk<IsrfChunkData>;
pub type ItchChunk = KnownChunk<ItchChunkData>;
pub type IcmsChunk = KnownChunk<IcmsChunkData>;
pub type IcmtChunk = KnownChunk<IcmtChunkData>;
pub type IcopChunk = KnownChunk<IcopChunkData>;
pub type IcrdChunk = KnownChunk<IcrdChunkData>;
pub type IcrpChunk = KnownChunk<IcrpChunkData>;
pub type IdpiChunk = KnownChunk<IdpiChunkData>;
pub type IengChunk = KnownChunk<IengChunkData>;

#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
pub enum InfoChunkEnum {
    Iarl(IarlChunk),
    Ignr(IgnrChunk),
    Ikey(IkeyChunk),
    Ilgt(IlgtChunk),
    Imed(ImedChunk),
    Inam(InamChunk),
    Iplt(IpltChunk),
    Iprd(IprdChunk),
    Isbj(IsbjChunk),
    Isft(IsftChunk),
    Ishp(IshpChunk),
    Iart(IartChunk),
    Isrc(IsrcChunk),
    Isrf(IsrfChunk),
    Itch(ItchChunk),
    Icms(IcmsChunk),
    Icmt(IcmtChunk),
    Icop(IcopChunk),
    Icrd(IcrdChunk),
    Icrp(IcrpChunk),
    Idpi(IdpiChunk),
    Ieng(IengChunk),
    Unknown {
        id: FourCC,
        size: u32,
        #[brw(align_after=2, pad_size_to= size.to_owned())]
        value: NullString,
    },
}

impl InfoChunkEnum {
    pub fn id(&self) -> FourCC {
        match self {
            InfoChunkEnum::Iarl(e) => e.id(),
            InfoChunkEnum::Ignr(e) => e.id(),
            InfoChunkEnum::Ikey(e) => e.id(),
            InfoChunkEnum::Ilgt(e) => e.id(),
            InfoChunkEnum::Imed(e) => e.id(),
            InfoChunkEnum::Inam(e) => e.id(),
            InfoChunkEnum::Iplt(e) => e.id(),
            InfoChunkEnum::Iprd(e) => e.id(),
            InfoChunkEnum::Isbj(e) => e.id(),
            InfoChunkEnum::Isft(e) => e.id(),
            InfoChunkEnum::Ishp(e) => e.id(),
            InfoChunkEnum::Iart(e) => e.id(),
            InfoChunkEnum::Isrc(e) => e.id(),
            InfoChunkEnum::Isrf(e) => e.id(),
            InfoChunkEnum::Itch(e) => e.id(),
            InfoChunkEnum::Icms(e) => e.id(),
            InfoChunkEnum::Icmt(e) => e.id(),
            InfoChunkEnum::Icop(e) => e.id(),
            InfoChunkEnum::Icrd(e) => e.id(),
            InfoChunkEnum::Icrp(e) => e.id(),
            InfoChunkEnum::Idpi(e) => e.id(),
            InfoChunkEnum::Ieng(e) => e.id(),
            InfoChunkEnum::Unknown { id, .. } => *id,
        }
    }

    pub fn value(&self) -> String {
        match self {
            InfoChunkEnum::Iarl(e) => e.data.value.to_string(),
            InfoChunkEnum::Ignr(e) => e.data.value.to_string(),
            InfoChunkEnum::Ikey(e) => e.data.value.to_string(),
            InfoChunkEnum::Ilgt(e) => e.data.value.to_string(),
            InfoChunkEnum::Imed(e) => e.data.value.to_string(),
            InfoChunkEnum::Inam(e) => e.data.value.to_string(),
            InfoChunkEnum::Iplt(e) => e.data.value.to_string(),
            InfoChunkEnum::Iprd(e) => e.data.value.to_string(),
            InfoChunkEnum::Isbj(e) => e.data.value.to_string(),
            InfoChunkEnum::Isft(e) => e.data.value.to_string(),
            InfoChunkEnum::Ishp(e) => e.data.value.to_string(),
            InfoChunkEnum::Iart(e) => e.data.value.to_string(),
            InfoChunkEnum::Isrc(e) => e.data.value.to_string(),
            InfoChunkEnum::Isrf(e) => e.data.value.to_string(),
            InfoChunkEnum::Itch(e) => e.data.value.to_string(),
            InfoChunkEnum::Icms(e) => e.data.value.to_string(),
            InfoChunkEnum::Icmt(e) => e.data.value.to_string(),
            InfoChunkEnum::Icop(e) => e.data.value.to_string(),
            InfoChunkEnum::Icrd(e) => e.data.value.to_string(),
            InfoChunkEnum::Icrp(e) => e.data.value.to_string(),
            InfoChunkEnum::Idpi(e) => e.data.value.to_string(),
            InfoChunkEnum::Ieng(e) => e.data.value.to_string(),
            InfoChunkEnum::Unknown { value, .. } => format!("Unknown(\"{}\")", *value),
        }
    }
}

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {
    use binrw::{BinRead, BinWrite};
    use hexdump::hexdump;

    use super::*;
    use crate::{box_chunk, testing::hex_to_cursor, Chunk, ChunkEnum};

    #[test]
    fn infochunk_roundtrip() {
        let icmt = InfoChunkEnum::Icmt(IcmtChunk {
            size: 8,
            data: IcmtChunkData {
                value: NullString("comment".into()),
            },
            extra_bytes: vec![],
        });
        println!("{icmt:?}");
        let mut buff = std::io::Cursor::new(Vec::<u8>::new());
        icmt.write(&mut buff).unwrap();
        println!("{:?}", hexdump(buff.get_ref()));
        buff.set_position(0);
        let after = InfoChunkEnum::read(&mut buff).unwrap();
        assert_eq!(after, icmt);
    }

    #[test]
    fn infochunk_small_valid() {
        // buff contains ICMT chunk with an odd length
        // handling the WORD padding incorrectly can break parsing
        let mut buff =
            hex_to_cursor("49434D54 15000000 62657874 20636875 6E6B2074 65737420 66696C65 00");
        // parse via explicit chunk type
        let icmt = IcmtChunk::read(&mut buff).unwrap();
        dbg!(&icmt);
        assert_eq!(icmt.id(), FourCC(*b"ICMT"));
        assert_eq!(icmt.data.value, "bext chunk test file".into());

        // parse via enum wrapper this time
        buff.set_position(0);
        let en = InfoChunkEnum::read(&mut buff).unwrap();
        dbg!(&en);
        assert_eq!(en.id(), FourCC(*b"ICMT"));
        let InfoChunkEnum::Icmt(icmt) = en else {
            unreachable!("should have been ICMT")
        };
        assert_eq!(icmt.data.value, "bext chunk test file".into());
    }

    #[test]
    fn listinfochunk_small_valid() {
        // buff contains INFO chunk with two odd length'd inner chunks
        // handling the WORD padding incorrectly can break parsing
        // if infochunk_small_valid() passes, but this fails, error is
        // likely in the ListInfoChunk wrapping
        let mut buff = hex_to_cursor(
        "4C495354 38000000 494E464F 49534654 0D000000 42574620 4D657461 45646974 00004943 4D541500 00006265 78742063 68756E6B 20746573 74206669 6C6500"
            );
        let list = ListInfoChunk::read(&mut buff).unwrap();
        assert_eq!(list.id(), FourCC(*b"LIST"));

        // parse via enum wrapper this time
        buff.set_position(0);
        let chunk = ChunkEnum::read(&mut buff).map(box_chunk).unwrap();
        assert_eq!(chunk.id(), FourCC(*b"LIST"));

        // let list = ListInfoChunk::read(&mut buff).unwrap();
        // assert_eq!(
    }

    #[test]
    fn infochunk_debug_string() {
        let icmt = IcmtChunkData {
            value: NullString("comment".into()),
        };
        assert!(format!("{icmt:?}").starts_with("InfoChunkData<ICMT>"));
    }

    #[test]
    fn icmtchunk_as_trait() {
        let icmt = IcmtChunk {
            size: 8,
            data: IcmtChunkData::new("comment"),
            extra_bytes: vec![],
        };
        // ensure trait bounds are satisfied
        let mut _trt: Box<dyn Chunk> = Box::new(icmt);
    }
}
