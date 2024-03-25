//! `INFO` A `LIST` containing descriptive text chunks: IARL, IGNR, IKEY, ILGT, IMED, INAM, IPLT, IPRD, ISBJ, ISFT, ISHP, IART, ISRC, ISRF, ITCH, ICMS, ICMT, ICOP, ICRD, ICRP, IDPI, IENG. [RIFF1991](https://wavref.til.cafe/chunk/info/)

use core::fmt::{Debug, Formatter};

use binrw::{binrw, helpers, NullString};
use itertools::Itertools;

use crate::{fourcc, ChunkID, FourCC, KnownChunk, KnownChunkID, Summarizable};

/// `LIST-INFO` holds subchunks of strings describing the WAVE.
#[binrw]
#[br(little)]
#[br(import(_size: u32))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ListInfoData {
    /// A four-character code that identifies the contents of the list.
    #[brw(assert(list_type == ListInfoData::LIST_TYPE))]
    pub list_type: FourCC,

    /// Sub chunks contained within this LIST
    #[br(parse_with = helpers::until_eof)]
    #[bw()]
    pub chunks: Vec<InfoEnum>,
}

impl ListInfoData {
    /// Chunk id constant: `INFO`
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
        Box::new(self.chunks.iter().map(|c| (c.id().to_string(), c.text())))
    }

    fn item_summary_header(&self) -> String {
        "chunk: text".to_string()
    }
}

/// `LIST-INFO` holds subchunks of strings describing the WAVE.
pub type ListInfo = KnownChunk<ListInfoData>;

/// A genericised container for LIST-INFO subchunks.
///
/// A type alias is defined for each of the INFO types from the initial
/// [RIFF1991](https://wavref.til.cafe/chunk/info/) WAV spec.
///
/// # Examples:
/// Parsing chunk data from a buffer:
/// ```
/// # use wavrw::chunk::info::Icmt;
/// # use wavrw::testing::hex_to_cursor;
/// # use binrw::BinRead;
/// # let mut buff = hex_to_cursor("49434D54 15000000 62657874 20636875 6E6B2074 65737420 66696C65 00");
/// let icmt = Icmt::read(&mut buff).unwrap();
/// ```
///
/// Creating a new chunk from scratch:
/// ```
/// # use wavrw::chunk::info::IcmtData;
/// let icmt = IcmtData::new("comment");
/// ```
///
#[binrw]
#[br(little)]
#[br(import(_size: u32))]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct InfoData<const I: u32> {
    /// Generic container for `info` chunk text.
    #[br(map= |ns: NullString| ns.to_string())]
    #[bw(map= |s: &String| NullString::from(s.clone()))]
    pub text: String,
}

impl<const I: u32> KnownChunkID for InfoData<I> {
    const ID: FourCC = FourCC(I.to_le_bytes());
}

impl<const I: u32> Debug for InfoData<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        f.debug_struct(&format!("InfoData<{}>", Self::ID))
            .field("text", &self.text)
            .finish()
    }
}

impl<const I: u32> Summarizable for InfoData<I> {
    fn summary(&self) -> String {
        self.text.clone()
    }
}

impl<const I: u32> InfoData<I> {
    /// Creates a new [`InfoData<I>`] chunk.
    pub fn new(text: &str) -> Self {
        InfoData::<I> { text: text.into() }
    }
}

/// Archival Location. Indicates where the subject of the file is archived.
pub type IarlData = InfoData<{ fourcc(b"IARL") }>;
/// Genre. Describes the original work, such as "landscape", "portrait", "still
/// life", etc.
pub type IgnrData = InfoData<{ fourcc(b"IGNR") }>;
/// Keywords. Provides a list of keywords that refer to the file or subject
/// of the file. Separate multiple keywords with a semicolon and a blank. For
/// example, "Seattle; aerial view; scenery".
pub type IkeyData = InfoData<{ fourcc(b"IKEY") }>;
/// Lightness. Describes the changes in lightness settings on the digitizer
/// required to produce the file. Note that the format of this information depends
/// on hardware used.
pub type IlgtData = InfoData<{ fourcc(b"ILGT") }>;
/// Medium. Describes the original subject of the file, such as "computer
/// image", "drawing", "lithograph", and so forth.
pub type ImedData = InfoData<{ fourcc(b"IMED") }>;
/// Name. Stores the title of the subject of the file, such as "Seattle From Above".
pub type InamData = InfoData<{ fourcc(b"INAM") }>;
/// Palette Setting. Specifies the number of colors requested when digitizing an
/// image, such as "256".
pub type IpltData = InfoData<{ fourcc(b"IPLT") }>;
/// Product. Specifies the name of the title the file was originally intended
/// for, such as "Encyclopedia of Pacific Northwest Geography".
pub type IprdData = InfoData<{ fourcc(b"IPRD") }>;
/// Subject. Describes the contents of the file, such as "Aerial view of Seattle".
pub type IsbjData = InfoData<{ fourcc(b"ISBJ") }>;
/// Software. Identifies the name of the software package used to create the
/// file, such as "Microsoft WaveEdit".
pub type IsftData = InfoData<{ fourcc(b"ISFT") }>;
/// Sharpness. Identifies the changes in sharpness for the digitizer required to
/// produce the file (the format depends on the hardware used).
pub type IshpData = InfoData<{ fourcc(b"ISHP") }>;
/// Artist. Lists the artist of the original subject of the file. For example,
/// "Michaelangelo".
pub type IartData = InfoData<{ fourcc(b"IART") }>;
/// Source. Identifies the name of the person or organization who supplied the
/// original subject of the file. For example, "Trey Research".
pub type IsrcData = InfoData<{ fourcc(b"ISRC") }>;
/// Source Form. Identifies the original form of the material that was
/// digitized, such as "slide", "paper", "map", and so forth. This is not
/// necessarily the same as IMED.
pub type IsrfData = InfoData<{ fourcc(b"ISRF") }>;
/// Technician. Identifies the technician who digitized the subject file. For
/// example, "Smith, John."
pub type ItchData = InfoData<{ fourcc(b"ITCH") }>;
/// Commissioned. Lists the name of the person or organization that commissioned
/// the subject of the file. For example, "Pope Julian II".
pub type IcmsData = InfoData<{ fourcc(b"ICMS") }>;
/// Comments. Provides general comments about the file or the subject of the
/// file. If the comment is several sentences long, end each sentence with a
/// period. Do not include newline characters.
pub type IcmtData = InfoData<{ fourcc(b"ICMT") }>;
/// Copyright. Records the copyright information for the file. For example,
/// "Copyright Encyclopedia International 1991." If there are multiple
/// copyrights, separate them by a semicolon followed by a space.
pub type IcopData = InfoData<{ fourcc(b"ICOP") }>;
/// Creation date. Specifies the date the subject of the file was created. List
/// dates in year-month-day format, padding one-digit months and days with a
/// zero on the left. For example, "1553-05-03" for May 3, 1553.
pub type IcrdData = InfoData<{ fourcc(b"ICRD") }>;
/// Cropped. Describes whether an image has been cropped and, if so, how it was
/// cropped. For example, "lower right corner". IDIM Dimensions. Specifies the
/// size of the original subject of the file. For example, "8.5 in h, 11 in w".
pub type IcrpData = InfoData<{ fourcc(b"ICRP") }>;
/// Dots Per Inch. Stores dots per inch setting of the digitizer used to produce
/// the file, such as "300".
pub type IdpiData = InfoData<{ fourcc(b"IDPI") }>;
/// Engineer. Stores the name of the engineer who worked on the file. If there
/// are multiple engineers, separate the names by a semicolon and a blank. For
/// example, "Smith, John; Adams, Joe".
pub type IengData = InfoData<{ fourcc(b"IENG") }>;

/// Archival Location. Indicates where the subject of the file is archived.
pub type Iarl = KnownChunk<IarlData>;
/// Genre. Describes the original work, such as "landscape", "portrait", "still
/// life", etc.
pub type Ignr = KnownChunk<IgnrData>;
/// Keywords. Provides a list of keywords that refer to the file or subject
/// of the file. Separate multiple keywords with a semicolon and a blank. For
/// example, "Seattle; aerial view; scenery".
pub type Ikey = KnownChunk<IkeyData>;
/// Lightness. Describes the changes in lightness settings on the digitizer
/// required to produce the file. Note that the format of this information depends
/// on hardware used.
pub type Ilgt = KnownChunk<IlgtData>;
/// Medium. Describes the original subject of the file, such as "computer
/// image", "drawing", "lithograph", and so forth.
pub type Imed = KnownChunk<ImedData>;
/// Name. Stores the title of the subject of the file, such as "Seattle From Above".
pub type Inam = KnownChunk<InamData>;
/// Palette Setting. Specifies the number of colors requested when digitizing an
/// image, such as "256".
pub type Iplt = KnownChunk<IpltData>;
/// Product. Specifies the name of the title the file was originally intended
/// for, such as "Encyclopedia of Pacific Northwest Geography".
pub type Iprd = KnownChunk<IprdData>;
/// Subject. Describes the contents of the file, such as "Aerial view of Seattle".
pub type Isbj = KnownChunk<IsbjData>;
/// Software. Identifies the name of the software package used to create the
/// file, such as "Microsoft WaveEdit".
pub type Isft = KnownChunk<IsftData>;
/// Sharpness. Identifies the changes in sharpness for the digitizer required to
/// produce the file (the format depends on the hardware used).
pub type Ishp = KnownChunk<IshpData>;
/// Artist. Lists the artist of the original subject of the file. For example,
/// "Michaelangelo".
pub type Iart = KnownChunk<IartData>;
/// Source. Identifies the name of the person or organization who supplied the
/// original subject of the file. For example, "Trey Research".
pub type Isrc = KnownChunk<IsrcData>;
/// Source Form. Identifies the original form of the material that was
/// digitized, such as "slide", "paper", "map", and so forth. This is not
/// necessarily the same as IMED.
pub type Isrf = KnownChunk<IsrfData>;
/// Technician. Identifies the technician who digitized the subject file. For
/// example, "Smith, John."
pub type Itch = KnownChunk<ItchData>;
/// Commissioned. Lists the name of the person or organization that commissioned
/// the subject of the file. For example, "Pope Julian II".
pub type Icms = KnownChunk<IcmsData>;
/// Comments. Provides general comments about the file or the subject of the
/// file. If the comment is several sentences long, end each sentence with a
/// period. Do not include newline characters.
pub type Icmt = KnownChunk<IcmtData>;
/// Copyright. Records the copyright information for the file. For example,
/// "Copyright Encyclopedia International 1991." If there are multiple
/// copyrights, separate them by a semicolon followed by a space.
pub type Icop = KnownChunk<IcopData>;
/// Creation date. Specifies the date the subject of the file was created. List
/// dates in year-month-day format, padding one-digit months and days with a
/// zero on the left. For example, "1553-05-03" for May 3, 1553.
pub type Icrd = KnownChunk<IcrdData>;
/// Cropped. Describes whether an image has been cropped and, if so, how it was
/// cropped. For example, "lower right corner". IDIM Dimensions. Specifies the
/// size of the original subject of the file. For example, "8.5 in h, 11 in w".
pub type Icrp = KnownChunk<IcrpData>;
/// Dots Per Inch. Stores dots per inch setting of the digitizer used to produce
/// the file, such as "300".
pub type Idpi = KnownChunk<IdpiData>;
/// Engineer. Stores the name of the engineer who worked on the file. If there
/// are multiple engineers, separate the names by a semicolon and a blank. For
/// example, "Smith, John; Adams, Joe".
pub type Ieng = KnownChunk<IengData>;

/// All `LIST-INFO` chunk structs as an enum
#[allow(missing_docs)]
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
        #[br(map= |ns: NullString| ns.to_string())]
        #[bw(map= |s: &String| NullString::from(s.clone()))]
        text: String,
    },
}

impl ChunkID for InfoEnum {
    fn id(&self) -> FourCC {
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
}

impl InfoEnum {
    /// Return a clone of the inner chunks' text field.
    pub fn text(&self) -> String {
        match self {
            InfoEnum::Iarl(e) => e.data.text.clone(),
            InfoEnum::Ignr(e) => e.data.text.clone(),
            InfoEnum::Ikey(e) => e.data.text.clone(),
            InfoEnum::Ilgt(e) => e.data.text.clone(),
            InfoEnum::Imed(e) => e.data.text.clone(),
            InfoEnum::Inam(e) => e.data.text.clone(),
            InfoEnum::Iplt(e) => e.data.text.clone(),
            InfoEnum::Iprd(e) => e.data.text.clone(),
            InfoEnum::Isbj(e) => e.data.text.clone(),
            InfoEnum::Isft(e) => e.data.text.clone(),
            InfoEnum::Ishp(e) => e.data.text.clone(),
            InfoEnum::Iart(e) => e.data.text.clone(),
            InfoEnum::Isrc(e) => e.data.text.clone(),
            InfoEnum::Isrf(e) => e.data.text.clone(),
            InfoEnum::Itch(e) => e.data.text.clone(),
            InfoEnum::Icms(e) => e.data.text.clone(),
            InfoEnum::Icmt(e) => e.data.text.clone(),
            InfoEnum::Icop(e) => e.data.text.clone(),
            InfoEnum::Icrd(e) => e.data.text.clone(),
            InfoEnum::Icrp(e) => e.data.text.clone(),
            InfoEnum::Idpi(e) => e.data.text.clone(),
            InfoEnum::Ieng(e) => e.data.text.clone(),
            InfoEnum::Unknown { text, .. } => format!("Unknown(\"{}\")", *text),
        }
    }
}

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {
    use binrw::{BinRead, BinWrite};
    use hexdump::hexdump;

    use super::*;
    use crate::{box_chunk, testing::hex_to_cursor, SizedChunk, SizedChunkEnum};

    #[test]
    fn infochunk_roundtrip() {
        let icmt = InfoEnum::Icmt(Icmt {
            size: 8,
            data: IcmtData {
                text: String::from("comment"),
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
        assert_eq!(icmt.data.text, "bext chunk test file".to_string());

        // parse via enum wrapper this time
        buff.set_position(0);
        let en = InfoEnum::read(&mut buff).unwrap();
        dbg!(&en);
        assert_eq!(en.id(), FourCC(*b"ICMT"));
        let InfoEnum::Icmt(icmt) = en else {
            unreachable!("should have been ICMT")
        };
        assert_eq!(icmt.data.text, "bext chunk test file".to_string());
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
        let chunk = SizedChunkEnum::read(&mut buff).map(box_chunk).unwrap();
        assert_eq!(chunk.id(), FourCC(*b"LIST"));

        // let list = ListInfo::read(&mut buff).unwrap();
        // assert_eq!(
    }

    #[test]
    fn infochunk_debug_string() {
        let icmt = IcmtData {
            text: "comment".to_string(),
        };
        println!("{icmt:?}");
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
        let mut _trt: Box<dyn SizedChunk> = Box::new(icmt);
    }
}
