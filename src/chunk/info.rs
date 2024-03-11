use std::fmt::{Debug, Formatter};

use binrw::{binrw, helpers, NullString};
use itertools::Itertools;

use crate::{fourcc, ChunkID, FourCC, KnownChunk, KnownChunkID, Summarizable};

#[binrw]
#[br(little)]
#[br(import(_size: u32))]
#[derive(Debug, PartialEq, Eq)]
pub struct ListInfoData {
    #[brw(assert(list_type == ListInfoData::LIST_TYPE))]
    pub list_type: FourCC,
    #[br(parse_with = helpers::until_eof)]
    #[bw()]
    pub chunks: Vec<InfoEnum>,
}

impl ListInfoData {
    pub const LIST_TYPE: FourCC = FourCC(*b"INFO");
}

impl KnownChunkID for ListInfoData {
    const ID: FourCC = FourCC(*b"LIST");
}

impl Summarizable for ListInfoData {
    fn summary(&self) -> String {
        self.chunks.iter().map(|c| c.id()).join(", ")
    }

    fn name(&self) -> String {
        format!("{}-{}", self.id(), self.list_type)
    }

    fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        Box::new(self.chunks.iter().map(|c| (c.id().to_string(), c.value())))
    }

    fn item_summary_header(&self) -> String {
        "chunk: text".to_string()
    }
}

pub type ListInfo = KnownChunk<ListInfoData>;

/// InfoData is a genericised container for LIST INFO chunks
///
/// A type alias is devfined for all of the INFO types from the initial 1991
/// WAV spec.
///
/// # Examples:
///
/// ```
/// # use wavrw::chunk::info::IcmtData;
/// let icmt = IcmtData::new("comment");
/// ```
///
#[binrw]
#[br(little)]
#[br(import(_size: u32))]
#[derive(PartialEq, Eq)]
pub struct InfoData<const I: u32> {
    pub value: NullString,
}

impl<const I: u32> KnownChunkID for InfoData<I> {
    const ID: FourCC = FourCC(I.to_le_bytes());
}

impl<const I: u32> Debug for InfoData<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "InfoData<{}> {{ value: {:?} }}",
            String::from_utf8_lossy(I.to_le_bytes().as_slice()),
            self.value,
        )?;
        Ok(())
    }
}

impl<const I: u32> Summarizable for InfoData<I> {
    fn summary(&self) -> String {
        self.value.to_string()
    }
}

impl<const I: u32> InfoData<I> {
    pub fn new(value: &str) -> Self {
        InfoData::<I> {
            value: value.into(),
        }
    }
}

pub type IarlData = InfoData<{ fourcc(b"IARL") }>;
pub type IgnrData = InfoData<{ fourcc(b"IGNR") }>;
pub type IkeyData = InfoData<{ fourcc(b"IKEY") }>;
pub type IlgtData = InfoData<{ fourcc(b"ILGT") }>;
pub type ImedData = InfoData<{ fourcc(b"IMED") }>;
pub type InamData = InfoData<{ fourcc(b"INAM") }>;
pub type IpltData = InfoData<{ fourcc(b"IPLT") }>;
pub type IprdData = InfoData<{ fourcc(b"IPRD") }>;
pub type IsbjData = InfoData<{ fourcc(b"ISBJ") }>;
pub type IsftData = InfoData<{ fourcc(b"ISFT") }>;
pub type IshpData = InfoData<{ fourcc(b"ISHP") }>;
pub type IartData = InfoData<{ fourcc(b"IART") }>;
pub type IsrcData = InfoData<{ fourcc(b"ISRC") }>;
pub type IsrfData = InfoData<{ fourcc(b"ISRF") }>;
pub type ItchData = InfoData<{ fourcc(b"ITCH") }>;
pub type IcmsData = InfoData<{ fourcc(b"ICMS") }>;
pub type IcmtData = InfoData<{ fourcc(b"ICMT") }>;
pub type IcopData = InfoData<{ fourcc(b"ICOP") }>;
pub type IcrdData = InfoData<{ fourcc(b"ICRD") }>;
pub type IcrpData = InfoData<{ fourcc(b"ICRP") }>;
pub type IdpiData = InfoData<{ fourcc(b"IDPI") }>;
pub type IengData = InfoData<{ fourcc(b"IENG") }>;

pub type Iarl = KnownChunk<IarlData>;
pub type Ignr = KnownChunk<IgnrData>;
pub type Ikey = KnownChunk<IkeyData>;
pub type Ilgt = KnownChunk<IlgtData>;
pub type Imed = KnownChunk<ImedData>;
pub type Inam = KnownChunk<InamData>;
pub type Iplt = KnownChunk<IpltData>;
pub type Iprd = KnownChunk<IprdData>;
pub type Isbj = KnownChunk<IsbjData>;
pub type Isft = KnownChunk<IsftData>;
pub type Ishp = KnownChunk<IshpData>;
pub type Iart = KnownChunk<IartData>;
pub type Isrc = KnownChunk<IsrcData>;
pub type Isrf = KnownChunk<IsrfData>;
pub type Itch = KnownChunk<ItchData>;
pub type Icms = KnownChunk<IcmsData>;
pub type Icmt = KnownChunk<IcmtData>;
pub type Icop = KnownChunk<IcopData>;
pub type Icrd = KnownChunk<IcrdData>;
pub type Icrp = KnownChunk<IcrpData>;
pub type Idpi = KnownChunk<IdpiData>;
pub type Ieng = KnownChunk<IengData>;

#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
pub enum InfoEnum {
    Iarl(Iarl),
    Ignr(Ignr),
    Ikey(Ikey),
    Ilgt(Ilgt),
    Imed(Imed),
    Inam(Inam),
    Iplt(Iplt),
    Iprd(Iprd),
    Isbj(Isbj),
    Isft(Isft),
    Ishp(Ishp),
    Iart(Iart),
    Isrc(Isrc),
    Isrf(Isrf),
    Itch(Itch),
    Icms(Icms),
    Icmt(Icmt),
    Icop(Icop),
    Icrd(Icrd),
    Icrp(Icrp),
    Idpi(Idpi),
    Ieng(Ieng),
    Unknown {
        id: FourCC,
        size: u32,
        #[brw(align_after=2, pad_size_to= size.to_owned())]
        value: NullString,
    },
}

impl InfoEnum {
    pub fn id(&self) -> FourCC {
        match self {
            InfoEnum::Iarl(e) => e.id(),
            InfoEnum::Ignr(e) => e.id(),
            InfoEnum::Ikey(e) => e.id(),
            InfoEnum::Ilgt(e) => e.id(),
            InfoEnum::Imed(e) => e.id(),
            InfoEnum::Inam(e) => e.id(),
            InfoEnum::Iplt(e) => e.id(),
            InfoEnum::Iprd(e) => e.id(),
            InfoEnum::Isbj(e) => e.id(),
            InfoEnum::Isft(e) => e.id(),
            InfoEnum::Ishp(e) => e.id(),
            InfoEnum::Iart(e) => e.id(),
            InfoEnum::Isrc(e) => e.id(),
            InfoEnum::Isrf(e) => e.id(),
            InfoEnum::Itch(e) => e.id(),
            InfoEnum::Icms(e) => e.id(),
            InfoEnum::Icmt(e) => e.id(),
            InfoEnum::Icop(e) => e.id(),
            InfoEnum::Icrd(e) => e.id(),
            InfoEnum::Icrp(e) => e.id(),
            InfoEnum::Idpi(e) => e.id(),
            InfoEnum::Ieng(e) => e.id(),
            InfoEnum::Unknown { id, .. } => *id,
        }
    }

    pub fn value(&self) -> String {
        match self {
            InfoEnum::Iarl(e) => e.data.value.to_string(),
            InfoEnum::Ignr(e) => e.data.value.to_string(),
            InfoEnum::Ikey(e) => e.data.value.to_string(),
            InfoEnum::Ilgt(e) => e.data.value.to_string(),
            InfoEnum::Imed(e) => e.data.value.to_string(),
            InfoEnum::Inam(e) => e.data.value.to_string(),
            InfoEnum::Iplt(e) => e.data.value.to_string(),
            InfoEnum::Iprd(e) => e.data.value.to_string(),
            InfoEnum::Isbj(e) => e.data.value.to_string(),
            InfoEnum::Isft(e) => e.data.value.to_string(),
            InfoEnum::Ishp(e) => e.data.value.to_string(),
            InfoEnum::Iart(e) => e.data.value.to_string(),
            InfoEnum::Isrc(e) => e.data.value.to_string(),
            InfoEnum::Isrf(e) => e.data.value.to_string(),
            InfoEnum::Itch(e) => e.data.value.to_string(),
            InfoEnum::Icms(e) => e.data.value.to_string(),
            InfoEnum::Icmt(e) => e.data.value.to_string(),
            InfoEnum::Icop(e) => e.data.value.to_string(),
            InfoEnum::Icrd(e) => e.data.value.to_string(),
            InfoEnum::Icrp(e) => e.data.value.to_string(),
            InfoEnum::Idpi(e) => e.data.value.to_string(),
            InfoEnum::Ieng(e) => e.data.value.to_string(),
            InfoEnum::Unknown { value, .. } => format!("Unknown(\"{}\")", *value),
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
        let icmt = InfoEnum::Icmt(Icmt {
            size: 8,
            data: IcmtData {
                value: NullString("comment".into()),
            },
            extra_bytes: vec![],
        });
        println!("{icmt:?}");
        let mut buff = std::io::Cursor::new(Vec::<u8>::new());
        icmt.write(&mut buff).unwrap();
        println!("{:?}", hexdump(buff.get_ref()));
        buff.set_position(0);
        let after = InfoEnum::read(&mut buff).unwrap();
        assert_eq!(after, icmt);
    }

    #[test]
    fn infochunk_small_valid() {
        // buff contains ICMT chunk with an odd length
        // handling the WORD padding incorrectly can break parsing
        let mut buff =
            hex_to_cursor("49434D54 15000000 62657874 20636875 6E6B2074 65737420 66696C65 00");
        // parse via explicit chunk type
        let icmt = Icmt::read(&mut buff).unwrap();
        dbg!(&icmt);
        assert_eq!(icmt.id(), FourCC(*b"ICMT"));
        assert_eq!(icmt.data.value, "bext chunk test file".into());

        // parse via enum wrapper this time
        buff.set_position(0);
        let en = InfoEnum::read(&mut buff).unwrap();
        dbg!(&en);
        assert_eq!(en.id(), FourCC(*b"ICMT"));
        let InfoEnum::Icmt(icmt) = en else {
            unreachable!("should have been ICMT")
        };
        assert_eq!(icmt.data.value, "bext chunk test file".into());
    }

    #[test]
    fn listinfochunk_small_valid() {
        // buff contains INFO chunk with two odd length'd inner chunks
        // handling the WORD padding incorrectly can break parsing
        // if infochunk_small_valid() passes, but this fails, error is
        // likely in the ListInfo wrapping
        let mut buff = hex_to_cursor(
        "4C495354 38000000 494E464F 49534654 0D000000 42574620 4D657461 45646974 00004943 4D541500 00006265 78742063 68756E6B 20746573 74206669 6C6500"
            );
        let list = ListInfo::read(&mut buff).unwrap();
        assert_eq!(list.id(), FourCC(*b"LIST"));

        // parse via enum wrapper this time
        buff.set_position(0);
        let chunk = ChunkEnum::read(&mut buff).map(box_chunk).unwrap();
        assert_eq!(chunk.id(), FourCC(*b"LIST"));

        // let list = ListInfo::read(&mut buff).unwrap();
        // assert_eq!(
    }

    #[test]
    fn infochunk_debug_string() {
        let icmt = IcmtData {
            value: NullString("comment".into()),
        };
        assert!(format!("{icmt:?}").starts_with("InfoData<ICMT>"));
    }

    #[test]
    fn icmtchunk_as_trait() {
        let icmt = Icmt {
            size: 8,
            data: IcmtData::new("comment"),
            extra_bytes: vec![],
        };
        // ensure trait bounds are satisfied
        let mut _trt: Box<dyn Chunk> = Box::new(icmt);
    }
}
