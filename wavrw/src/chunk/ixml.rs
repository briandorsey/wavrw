//! `iXML` Production workflow file & project metadata.  [IXML2021](https://wavref.til.cafe/spec/ixml2021/)
//!
//! The general approach in this module is to map specified items to specific
//! fields where possible and store unspecified (outside of iXML spec) items as
//! (key, value) in an `extra` field.
//!
//! The implimentation is defaulting to being fairly picky about following
//! the iXML spec... open to loosing up to meet common in-use patterns.
//!
//! Since XML tags could have any string content, some fields are defined with
//! enums of specified values, plus `Custom(String)`.

use alloc::collections::BTreeMap;
use alloc::collections::btree_map::IntoIter;
use core::fmt::{Debug, Display, Formatter};
use core::{error, fmt};
use itertools::Itertools;
use std::io;

use binrw::binrw;
use binrw::io::TakeSeekExt;
use xml::reader::{EventReader, XmlEvent};

use crate::{ChunkID, FourCC, KnownChunkID, SizedChunk, Summarizable};

// TODO: decide on a pattern for represending missing iXML tags in the structs. Maybe use default values, maybe wrap all in Options?
// TODO: review strct String values. Ex: numbers, time, bits... which ones should have conversions and typed values? But... because XML, everything needs a string backup for safety/round-tripping?
// TODO: determine how to deal with string case variations in tags
// TODO: meta to all of the above: is round-trip consistency a goal of the library, or OK to write logically consistent, but perhaps with differening text or XML nodes?

/// Ixml errors.
#[derive(Debug)]
#[non_exhaustive]
pub enum IxmlError {
    /// An error occurred in the underlying reader while reading or seeking to data.
    ///
    /// Contains an [`std::io::Error`]
    Io(std::io::Error),

    /// An error occured while parsing XML data.
    ///
    /// A string representation of the underlying error. Explicitly not
    /// returning underlying xml library error types to insulate from potential
    /// future XML parsing library changes.
    Parse {
        /// Parsing error message.
        message: String,
    },
}
impl error::Error for IxmlError {}

impl Display for IxmlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            IxmlError::Io(err) => write!(f, "Io: {}", err),
            IxmlError::Parse { message, .. } => write!(f, "Parse: {}", message),
        }
    }
}

/// Represents iXML `TAKE_TYPE` dictionary
///
/// the `TAKE_TYPE` tag contains a comma delimited list of values from the
/// dictionary below, following recommendations by AFSI. This tag supercedes
/// the older explicit tags for `WILD_TRACK`, `NO_GOOD`, `FALSE_START` and
/// replaces them with a comma delimited list from the dictionary below
/// which can be expanded in the future. If this tag is absent or empty
/// or contains just the word `DEFAULT`, the take should be considered to be
/// standard `TAKE`.
#[allow(missing_docs)] // TODO: remove this after finding AFSI reference.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TakeType {
    Default,
    NoGood,
    FalseStart,
    WildTrack,
    PickUp,
    Rehearsal,
    Announcement,
    SoundGuide,
    Custom(String),
}

impl TakeType {
    fn from_multiple(value: String) -> Vec<Self> {
        let pat = ',';
        if value.contains(pat) {
            value
                .split(pat)
                .map(|i| TakeType::from(i.trim().to_string()))
                .collect()
        } else {
            vec![value.into()]
        }
    }
}

impl Display for TakeType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let out = match self {
            TakeType::Default => "DEFAULT",
            TakeType::NoGood => "NO_GOOD",
            TakeType::FalseStart => "FALSE_START",
            TakeType::WildTrack => "WILD_TRACK",
            TakeType::PickUp => "PICKUP",
            TakeType::Rehearsal => "REHEARSAL",
            TakeType::Announcement => "ANNOUNCEMENT",
            TakeType::SoundGuide => "SOUND_GUIDE",
            TakeType::Custom(value) => value,
        };
        f.write_str(out)?;
        Ok(())
    }
}

impl From<String> for TakeType {
    // from https://www.gallery.co.uk/ixml/taketype_dictionary.html
    fn from(value: String) -> Self {
        match value.as_str() {
            "" | "DEFAULT" => Self::Default,
            "NO_GOOD" => Self::NoGood,
            "FALSE_START" => Self::FalseStart,
            "WILD_TRACK" => Self::WildTrack,
            "PICKUP" => Self::PickUp,
            "REHEARSAL" => Self::Rehearsal,
            "ANNOUNCEMENT" => Self::Announcement,
            "SOUND_GUIDE" => Self::SoundGuide,
            &_ => Self::Custom(value),
        }
    }
}

// iXML, based on http://www.gallery.co.uk/ixml/

/// `iXML` Production workflow file & project metadata.  [IXML2021](https://wavref.til.cafe/spec/ixml2021/)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ixml {
    /// The version number of the iXML specification used to prepare
    /// the iXML audio file. This version appears in the front page at
    /// [http://www.ixml.info](http://www.ixml.info), and takes the form of x.y
    /// where x and y are whole numbers, for example 1.51
    pub ixml_version: String,

    /// The name of the project to which this file belongs. This might typically
    /// be the name of the motion picture or program which is in production.
    pub project: String,

    /// The name of the scene / slate being recorded. For US system this might
    /// typically be 32, 32A, 32B, A32B, 32AB etc. For UK system this might
    /// typically be a incrementing number with no letters.
    pub scene: String,

    /// The `SoundRoll` which identifies a group of recordings. Normally, the
    /// `SoundRoll` is a vital component of workflow to differentiate audio
    /// recorded with time of day on different days. In other words for 2
    /// (completely different) recordings each covering a period around 11am,
    /// the soundroll would differentiate them by (typically) telling you which
    /// shooting day this recording applies to. Some projects may turnover sound
    /// more than once per day, and increment the soundroll at this point. In
    /// any event, the soundroll should change at least once in any 24 hour
    /// period. Some systems change the soundroll for every recording which
    /// is also a valid option, in effect using the soundroll as a unique file
    /// identifer (although this function is explicitly provided with the iXML
    /// `file_uid` parameter).
    pub tape: String,

    /// The number of the take in the current scene or slate. Usually this will
    /// be a simple number, although variations for things like wild tracks may
    /// yield takes like 1, 2, 3, WT1, WT2 etc.
    pub take: String,

    /// (New in iXML v2.0) A dictionary based tag allowing selection
    /// from a defined list of values to explicitly categorise the
    /// type/purpose/function of the current take. This tag overlaps with the
    /// existing `NO_GOOD` / `FALSE_START` / `WILD_TRACK` which are deprecated
    /// in iXML 2.0, This tag can contain multiple entries, separated by commas
    /// and can be expanded in the future with additional dictionary entries,
    /// detailed in the `TAKE_TYPE` dictionary.
    pub take_type: Vec<TakeType>,

    /// This parameter allows a recorder to mark this recording as a
    /// circle-take. The value should be TRUE or FALSE. If absent, this should be
    /// assumed FALSE.
    pub circled: bool,

    /// A unique number which identifies this physical FILE, regardless of the
    /// number of channels etc. If your system employs a unique `SoundRoll` per
    /// recording, your `FILE_UID` and TAPE parameters should be the same.
    pub file_uid: String,

    /// The userbits associated with this recording. This may have been
    /// extracted from incoming timecode when the file was recorded, or
    /// generated by the recorder from the date, or any other metadata.
    /// Typically the userbits are rarely used now because other more explicit
    /// metadata supercedes this function.
    pub ubits: String,

    /// A free text note to add user metadata to the recording. This might
    /// typically used to communicate information such as TAIL SLATE, NO SLATE,
    /// or to warn of noise interruptions - PLANE OVERHEAD etc.
    pub note: String,

    /// Additional tags found in the XML document beyond those listed in
    /// the iXML spec.
    pub extra: BTreeMap<String, String>,
}

impl KnownChunkID for Ixml {
    const ID: FourCC = FourCC(*b"iXML");
}

impl Ixml {
    /// Create a new Ixml struct.
    pub fn new() -> Ixml {
        Ixml {
            ixml_version: String::new(),
            scene: String::new(),
            project: String::new(),
            extra: BTreeMap::new(),
            tape: String::new(),
            take: String::new(),
            take_type: Vec::new(),
            circled: false,
            file_uid: String::new(),
            ubits: String::new(),
            note: String::new(),
        }
    }

    /// Helper to set an IXML value by path.
    ///
    /// ```
    /// use wavrw::chunk::ixml::Ixml;
    /// let mut ixml = Ixml::new();
    /// assert_eq!("", ixml.project);
    /// ixml.set(&["BWFXML".to_string(), "PROJECT".to_string()], "My Project".to_string());
    /// assert_eq!("My Project", ixml.project);
    /// ```
    pub fn set(&mut self, path: &[String], value: String) {
        if let Some(last) = path.last() {
            match last.as_str() {
                "IXML_VERSION" => self.ixml_version = value,
                "PROJECT" => self.project = value,
                "SCENE" => self.scene = value,
                "TAPE" => self.tape = value,
                "TAKE" => self.take = value,
                "TAKE_TYPE" => self.take_type.append(&mut TakeType::from_multiple(value)),
                "NO_GOOD" => {
                    if value == "TRUE" {
                        self.take_type.push(TakeType::NoGood);
                    }
                }
                "FALSE_START" => {
                    if value == "TRUE" {
                        self.take_type.push(TakeType::FalseStart);
                    }
                }
                "WILD_TRACK" => {
                    if value == "TRUE" {
                        self.take_type.push(TakeType::WildTrack);
                    }
                }
                "CIRCLED" => self.circled = true,
                "FILE_UID" => self.file_uid = value,
                "UBITS" => self.ubits = value,
                "NOTE" => self.note = value,
                &_ => {
                    self.extra.insert(path[1..].join("/"), value);
                }
            }
        }
    }

    /// Create Ixml struct from iXML data
    pub fn from_reader(reader: impl io::Read) -> Result<Ixml, IxmlError> {
        let mut ixml = Ixml::new();
        let parser = EventReader::new(reader);
        let mut path = Vec::<String>::new();
        for e in parser {
            match e {
                Ok(XmlEvent::StartElement { name, .. }) => {
                    path.push(name.local_name);
                }
                Ok(XmlEvent::Characters(chars)) => {
                    ixml.set(&path, chars);
                }
                Ok(XmlEvent::EndElement { name: _ }) => {
                    path.pop();
                }
                Err(e) => {
                    // TODO: actual error handling
                    eprintln!("Error: {e}");
                    break;
                }
                // There's more: https://docs.rs/xml-rs/latest/xml/reader/enum.XmlEvent.html
                _ => {}
            }
        }
        Ok(ixml)
    }
}

impl Default for Ixml {
    fn default() -> Self {
        Ixml::new()
    }
}

impl Summarizable for Ixml {
    fn summary(&self) -> String {
        format!("{}, {}", self.ixml_version, self.project)
    }

    fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        Box::new(self.into_iter())
    }

    fn item_summary_header(&self) -> String {
        String::new()
    }
}

impl<'a> IntoIterator for &'a Ixml {
    type Item = (String, String);
    type IntoIter = IxmlDataIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        IxmlDataIterator {
            data: self,
            index: 0,
            extra_iter: self.extra.clone().into_iter(),
        }
    }
}

/// Iterate over fields as tuple of Strings (name, value).
#[derive(Debug)]
pub struct IxmlDataIterator<'a> {
    data: &'a Ixml,
    index: usize,
    extra_iter: IntoIter<String, String>,
}

impl Iterator for IxmlDataIterator<'_> {
    type Item = (String, String);

    fn next(&mut self) -> Option<(String, String)> {
        self.index += 1;
        match self.index {
            1 => Some(("ixml_version".to_string(), self.data.ixml_version.clone())),
            2 => Some(("project".to_string(), self.data.project.clone())),
            3 => Some(("scene".to_string(), self.data.scene.clone())),
            4 => Some(("tape".to_string(), self.data.tape.clone())),
            5 => Some(("take".to_string(), self.data.take.clone())),
            6 => Some((
                "take_type".to_string(),
                self.data
                    .take_type
                    .iter()
                    .map(|tt| tt.to_string())
                    .join(", "),
            )),
            7 => Some(("circled".to_string(), self.data.circled.to_string())),
            8 => Some(("file_uid".to_string(), self.data.file_uid.clone())),
            9 => Some(("ubits".to_string(), self.data.ubits.clone())),
            10 => Some(("note".to_string(), self.data.note.clone())),
            _ => self.extra_iter.next(),
        }
    }
}

#[binrw::parser(reader: r)]
fn parse_ixml() -> binrw::BinResult<Ixml> {
    let ixml = Ixml::from_reader(r);
    match ixml {
        Ok(ixml) => binrw::BinResult::Ok(ixml),
        // Err(err) => Err(binrw::Error::AssertFail {
        //     pos: 0,
        //     message: err.to_string(),
        // }),
        Err(err) => Err(binrw::Error::Custom {
            pos: 0,
            err: Box::new(err),
        }),
    }
}

#[binrw::writer(writer: _w)]
fn write_ixml(_ixml: &Ixml) -> binrw::BinResult<()> {
    unreachable!("iXML writing not yet implemented (and wavrw itsel doesn't yet support writing");
}

/// `iXML` Production workflow file & project metadata.  [IXML2021](https://wavref.til.cafe/spec/ixml2021/)
#[binrw]
#[br(little)]
#[br(stream = r)]
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct IxmlChunk {
    /// Calculated offset from the beginning of the data stream this chunk is from or None.
    ///
    /// Ignored when writing chunks.
    #[br(try_calc = Some(r.stream_position()).transpose())]
    #[bw(ignore)]
    pub offset: Option<u64>,

    /// RIFF chunk id.
    #[br(temp, assert(id == Ixml::ID))]
    #[bw(calc = Ixml::ID)]
    pub id: FourCC,

    /// RIFF chunk size in bytes.
    pub size: u32,

    // take_seek() to ensure that we don't read outside the bounds for this chunk
    /// Ixml data struct.
    #[br(map_stream = |r| r.take_seek(size as u64))]
    #[br(parse_with = parse_ixml)]
    #[bw(write_with = write_ixml)]
    #[brw(align_after = 2)]
    pub data: Ixml,
}

impl Display for IxmlChunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} {}", self.name(), self.data.summary())
    }
}

impl ChunkID for IxmlChunk {
    fn id(&self) -> FourCC {
        Ixml::ID
    }
}

impl SizedChunk for IxmlChunk {
    fn size(&self) -> u32 {
        self.size
    }

    fn offset(&self) -> Option<u64> {
        self.offset
    }
}

impl Summarizable for IxmlChunk {
    fn summary(&self) -> String {
        self.data.summary()
    }

    fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        self.data.items()
    }

    fn item_summary_header(&self) -> String {
        self.data.item_summary_header()
    }
}

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {
    use binrw::BinRead;

    use super::*;
    use crate::testing::hex_to_cursor;

    #[test]
    fn take_type_from() {
        let variations = vec![
            ("DEFAULT", TakeType::Default),
            ("NO_GOOD", TakeType::NoGood),
            ("UNSPECIFIED", TakeType::Custom("UNSPECIFIED".to_string())),
        ];
        for (input, expected) in variations {
            assert_eq!(expected, TakeType::from(input.to_string()));
            // round trip
            assert_eq!(input, TakeType::from(input.to_string()).to_string());
        }
    }

    #[test]
    fn take_type_from_multiple() {
        let variations = vec![
            ("DEFAULT", vec![TakeType::Default]),
            ("NO_GOOD", vec![TakeType::NoGood]),
            // this one from iXML spec example
            (
                "FALSE_START,PICKUP",
                vec![TakeType::FalseStart, TakeType::PickUp],
            ),
            (
                "DEFAULT, NO_GOOD",
                vec![TakeType::Default, TakeType::NoGood],
            ),
            (
                "DEFAULT, UNSPECIFIED",
                vec![
                    TakeType::Default,
                    TakeType::Custom("UNSPECIFIED".to_string()),
                ],
            ),
        ];
        for (input, expected) in variations {
            assert_eq!(expected, TakeType::from_multiple(input.to_string()));
        }
    }

    #[test]
    fn parse_ixml_deprecated_take_types() {
        let variations = vec![
            (
                "<BWFXML>
                    <NO_GOOD>TRUE</NO_GOOD>
                    <FALSE_START>TRUE</FALSE_START>
                    <WILD_TRACK>TRUE</WILD_TRACK>
                </BWFXML>",
                vec![TakeType::NoGood, TakeType::FalseStart, TakeType::WildTrack],
            ),
            (
                "<BWFXML>
                    <TAKE_TYPE>DEFAULT</TAKE_TYPE>
                    <NO_GOOD>not valid input</NO_GOOD>
                    <FALSE_START>FALSE</FALSE_START>
                </BWFXML>", // WILD_TRACK not included
                vec![TakeType::Default],
            ),
        ];

        for (input, expected) in variations {
            let ixml = Ixml::from_reader(input.as_bytes()).expect("error parsing xml");
            println!("{:?}", ixml);
            assert_eq!(expected, ixml.take_type);
        }
    }

    #[test]
    fn parse_ixml() {
        // example bext chunk data from BWF MetaEdit
        let mut buff = hex_to_cursor(
            r#"
69584D4C A4110000 3C425746 584D4C3E 0A20203C 49584D4C 5F564552 53494F4E 3E312E36
313C2F49 584D4C5F 56455253 494F4E3E 0A20203C 5343454E 453E5363 656E653C 2F534345
4E453E0A 20203C50 524F4A45 43543E69 786D6C50 726F6A65 63743C2F 50524F4A 4543543E
0A20203C 54414B45 3E54616B 653C2F54 414B453E 0A20203C 42455854 3E0A2020 20203C42
57465F4F 52494749 4E41544F 525F5245 46455245 4E43453E 42574F72 6967696E 61746F72
5265663C 2F425746 5F4F5249 47494E41 544F525F 52454645 52454E43 453E0A20 2020203C
4257465F 44455343 52495054 494F4E3E 42574465 73637269 7074696F 6E3C2F42 57465F44
45534352 49505449 4F4E3E0A 20202020 3C425746 5F54494D 455F5245 46455245 4E43455F
48494748 3E303C2F 4257465F 54494D45 5F524546 4552454E 43455F48 4947483E 0A202020
203C4257 465F5449 4D455F52 45464552 454E4345 5F4C4F57 3E303C2F 4257465F 54494D45
5F524546 4552454E 43455F4C 4F573E0A 20202020 3C425746 5F4F5249 47494E41 544F523E
42574F72 6967696E 61746F72 3C2F4257 465F4F52 4947494E 41544F52 3E0A2020 3C2F4245
58543E0A 20203C4E 4F54453E 69786D6C 4E6F7465 3C2F4E4F 54453E0A 20203C54 5241434B
5F4C4953 543E0A20 2020203C 54524143 4B3E0A20 20202020 203C494E 5445524C 45415645
5F494E44 45583E31 3C2F494E 5445524C 45415645 5F494E44 45583E0A 20202020 20203C4E
414D453E 69786D6C 54726163 6B4C6179 6F75743C 2F4E414D 453E0A20 20202020 203C4348
414E4E45 4C5F494E 4445583E 313C2F43 48414E4E 454C5F49 4E444558 3E0A2020 20203C2F
54524143 4B3E0A20 2020203C 54524143 4B5F434F 554E543E 313C2F54 5241434B 5F434F55
4E543E0A 20203C2F 54524143 4B5F4C49 53543E0A 20203C54 4150453E 54617065 3C2F5441
50453E0A 20203C53 5445494E 42455247 3E0A2020 20203C41 5454525F 4C495354 3E0A2020
20202020 3C415454 523E0A20 20202020 2020203C 4E414D45 3E4D6564 69614C69 62726172
793C2F4E 414D453E 0A202020 20202020 203C5459 50453E73 7472696E 673C2F54 5950453E
0A202020 20202020 203C5641 4C55453E 4C696272 6172793C 2F56414C 55453E0A 20202020
20203C2F 41545452 3E0A2020 20202020 3C415454 523E0A20 20202020 2020203C 4E414D45
3E4D6564 69614361 7465676F 7279506F 73743C2F 4E414D45 3E0A2020 20202020 20203C54
5950453E 73747269 6E673C2F 54595045 3E0A2020 20202020 20203C56 414C5545 3E436174
65676F72 793C2F56 414C5545 3E0A2020 20202020 3C2F4154 54523E0A 20202020 20203C41
5454523E 0A202020 20202020 203C4E41 4D453E4D 75736963 616C4368 61726163 7465723C
2F4E414D 453E0A20 20202020 2020203C 54595045 3E737472 696E673C 2F545950 453E0A20
20202020 2020203C 56414C55 453E4D6F 6F643C2F 56414C55 453E0A20 20202020 203C2F41
5454523E 0A202020 2020203C 41545452 3E0A2020 20202020 20203C4E 414D453E 4D656469
61526563 6F726469 6E674D65 74686F64 3C2F4E41 4D453E0A 20202020 20202020 3C545950
453E7374 72696E67 3C2F5459 50453E0A 20202020 20202020 3C56414C 55453E4D 6963726F
70686F6E 653C2F56 414C5545 3E0A2020 20202020 3C2F4154 54523E0A 20202020 20203C41
5454523E 0A202020 20202020 203C4E41 4D453E4D 65646961 436F6D6D 656E743C 2F4E414D
453E0A20 20202020 2020203C 54595045 3E737472 696E673C 2F545950 453E0A20 20202020
2020203C 56414C55 453E4465 73637269 7074696F 6E3C2F56 414C5545 3E0A2020 20202020
3C2F4154 54523E0A 20202020 20203C41 5454523E 0A202020 20202020 203C4E41 4D453E53
6D66536F 6E674E61 6D653C2F 4E414D45 3E0A2020 20202020 20203C54 5950453E 73747269
6E673C2F 54595045 3E0A2020 20202020 20203C56 414C5545 3E547261 636B5469 746C653C
2F56414C 55453E0A 20202020 20203C2F 41545452 3E0A2020 20202020 3C415454 523E0A20
20202020 2020203C 4E414D45 3E4D6564 69615265 636F7264 696E674C 6F636174 696F6E3C
2F4E414D 453E0A20 20202020 2020203C 54595045 3E737472 696E673C 2F545950 453E0A20
20202020 2020203C 56414C55 453E4C6F 63617469 6F6E3C2F 56414C55 453E0A20 20202020
203C2F41 5454523E 0A202020 2020203C 41545452 3E0A2020 20202020 20203C4E 414D453E
4D757369 63616C49 6E737472 756D656E 743C2F4E 414D453E 0A202020 20202020 203C5459
50453E73 7472696E 673C2F54 5950453E 0A202020 20202020 203C5641 4C55453E 4B657977
6F726473 3C2F5641 4C55453E 0A202020 2020203C 2F415454 523E0A20 20202020 203C4154
54523E0A 20202020 20202020 3C4E414D 453E4D75 73696361 6C436174 65676F72 793C2F4E
414D453E 0A202020 20202020 203C5459 50453E73 7472696E 673C2F54 5950453E 0A202020
20202020 203C5641 4C55453E 53756243 61746567 6F72793C 2F56414C 55453E0A 20202020
20203C2F 41545452 3E0A2020 20202020 3C415454 523E0A20 20202020 2020203C 4E414D45
3E4D6564 6961436F 6D70616E 793C2F4E 414D453E 0A202020 20202020 203C5459 50453E73
7472696E 673C2F54 5950453E 0A202020 20202020 203C5641 4C55453E 5075626C 69736865
723C2F56 414C5545 3E0A2020 20202020 3C2F4154 54523E0A 20202020 20203C41 5454523E
0A202020 20202020 203C4E41 4D453E4D 65646961 4C696272 6172794D 616E7566 61637475
7265724E 616D653C 2F4E414D 453E0A20 20202020 2020203C 54595045 3E737472 696E673C
2F545950 453E0A20 20202020 2020203C 56414C55 453E4D61 6E756661 63747572 65723C2F
56414C55 453E0A20 20202020 203C2F41 5454523E 0A202020 2020203C 41545452 3E0A2020
20202020 20203C4E 414D453E 41756469 6F536F75 6E644564 69746F72 3C2F4E41 4D453E0A
20202020 20202020 3C545950 453E7374 72696E67 3C2F5459 50453E0A 20202020 20202020
3C56414C 55453E44 65736967 6E65723C 2F56414C 55453E0A 20202020 20203C2F 41545452
3E0A2020 20202020 3C415454 523E0A20 20202020 2020203C 4E414D45 3E4D6564 69615472
61636B4E 756D6265 723C2F4E 414D453E 0A202020 20202020 203C5459 50453E73 7472696E
673C2F54 5950453E 0A202020 20202020 203C5641 4C55453E 303C2F56 414C5545 3E0A2020
20202020 3C2F4154 54523E0A 20202020 20203C41 5454523E 0A202020 20202020 203C4E41
4D453E4D 65646961 416C6275 6D3C2F4E 414D453E 0A202020 20202020 203C5459 50453E73
7472696E 673C2F54 5950453E 0A202020 20202020 203C5641 4C55453E 43445469 746C653C
2F56414C 55453E0A 20202020 20203C2F 41545452 3E0A2020 20202020 3C415454 523E0A20
20202020 2020203C 4E414D45 3E417564 696F536F 756E644D 69786572 3C2F4E41 4D453E0A
20202020 20202020 3C545950 453E7374 72696E67 3C2F5459 50453E0A 20202020 20202020
3C56414C 55453E52 65636F72 64697374 3C2F5641 4C55453E 0A202020 2020203C 2F415454
523E0A20 20202020 203C4154 54523E0A 20202020 20202020 3C4E414D 453E4D65 64696141
72746973 743C2F4E 414D453E 0A202020 20202020 203C5459 50453E73 7472696E 673C2F54
5950453E 0A202020 20202020 203C5641 4C55453E 41727469 73743C2F 56414C55 453E0A20
20202020 203C2F41 5454523E 0A202020 203C2F41 5454525F 4C495354 3E0A2020 3C2F5354
45494E42 4552473E 0A20203C 55534552 3E0A2020 20203C50 55424C49 53484552 3E507562
6C697368 65723C2F 5055424C 49534845 523E0A20 2020203C 44455349 474E4552 3E446573
69676E65 723C2F44 45534947 4E45523E 0A202020 203C5348 4F4F5444 4154453E 53686F6F
74446174 653C2F53 484F4F54 44415445 3E0A2020 20203C53 484F573E 53686F77 3C2F5348
4F573E0A 20202020 3C545241 434B5449 544C453E 54726163 6B546974 6C653C2F 54524143
4B544954 4C453E0A 20202020 3C524543 54595045 3E526563 54797065 3C2F5245 43545950
453E0A20 2020203C 434F4D50 4F534552 3E436F6D 706F7365 723C2F43 4F4D504F 5345523E
0A202020 203C4341 5445474F 52593E43 61746567 6F72793C 2F434154 45474F52 593E0A20
2020203C 534F5552 43453E53 6F757263 653C2F53 4F555243 453E0A20 2020203C 4C4F4341
54494F4E 3E4C6F63 6174696F 6E3C2F4C 4F434154 494F4E3E 0A202020 203C4250 4D3E4250
4D3C2F42 504D3E0A 20202020 3C46584E 414D453E 46584E61 6D653C2F 46584E41 4D453E0A
20202020 3C564F4C 554D453E 536F7572 63653C2F 564F4C55 4D453E0A 20202020 3C4F5045
4E544945 523E4F70 656E5469 65723C2F 4F50454E 54494552 3E0A2020 20203C53 55424341
5445474F 52593E53 75624361 7465676F 72793C2F 53554243 41544547 4F52593E 0A202020
203C5245 434D4544 49554D3E 5265634D 65646975 6D3C2F52 45434D45 4449554D 3E0A2020
20203C43 41544944 3E436174 49443C2F 43415449 443E0A20 2020203C 55524C3E 55524C3C
2F55524C 3E0A2020 20203C55 53455243 4F4D4D45 4E54533E 55736572 436F6D6D 656E7473
3C2F5553 4552434F 4D4D454E 54533E0A 20202020 3C434154 45474F52 5946554C 4C3E4361
7465676F 72794675 6C6C3C2F 43415445 474F5259 46554C4C 3E0A2020 20203C56 454E444F
52434154 45474F52 593E5665 6E646F72 43617465 676F7279 3C2F5645 4E444F52 43415445
474F5259 3E0A2020 20203C44 45534352 49505449 4F4E3E44 65736372 69707469 6F6E3C2F
44455343 52495054 494F4E3E 0A202020 203C5348 4F525449 443E5368 6F727449 443C2F53
484F5254 49443E0A 20202020 3C415254 4953543E 41727469 73743C2F 41525449 53543E0A
20202020 3C545241 434B5945 41523E54 7261636B 59656172 3C2F5452 41434B59 4541523E
0A202020 203C454D 42454444 45523E53 6F756E64 6D696E65 723C2F45 4D424544 4445523E
0A202020 203C4D49 43504552 53504543 54495645 3E4D6963 50657273 70656374 6976653C
2F4D4943 50455253 50454354 4956453E 0A202020 203C5241 54494E47 3E526174 696E673C
2F524154 494E473E 0A202020 203C4C4F 4E474944 3E4C6F6E 6749443C 2F4C4F4E 4749443E
0A202020 203C4E4F 5445533E 4E6F7465 733C2F4E 4F544553 3E0A2020 20203C4D 4943524F
50484F4E 453E4D69 63726F70 686F6E65 3C2F4D49 43524F50 484F4E45 3E0A2020 20203C4C
49425241 52593E4C 69627261 72793C2F 4C494252 4152593E 0A202020 203C4D41 4E554641
43545552 45523E4D 616E7566 61637475 7265723C 2F4D414E 55464143 54555245 523E0A20
2020203C 54524143 4B3E303C 2F545241 434B3E0A 20202020 3C4B4559 574F5244 533E4B65
79776F72 64733C2F 4B455957 4F524453 3E0A2020 20203C55 53455243 41544547 4F52593E
55736572 43617465 676F7279 3C2F5553 45524341 5445474F 52593E0A 20202020 3C434454
49544C45 3E434454 69746C65 3C2F4344 5449544C 453E0A20 203C2F55 5345523E 0A20203C
41535747 3E0A2020 20203C6E 6F746573 3E4E6F74 65733C2F 6E6F7465 733E0A20 2020203C
696E4B65 793E4B65 793C2F69 6E4B6579 3E0A2020 20203C6F 72696769 6E61746F 723E4465
7369676E 65723C2F 6F726967 696E6174 6F723E0A 20202020 3C757365 72436174 65676F72
793E5573 65724361 7465676F 72793C2F 75736572 43617465 676F7279 3E0A2020 20203C63
61744964 3E436174 49443C2F 63617449 643E0A20 2020203C 6D757369 63507562 6C697368
65723E50 75626C69 73686572 3C2F6D75 73696350 75626C69 73686572 3E0A2020 20203C63
6F6D706F 7365723E 436F6D70 6F736572 3C2F636F 6D706F73 65723E0A 20202020 3C6D6963
54797065 3E4D6963 726F7068 6F6E653C 2F6D6963 54797065 3E0A2020 20203C73 75624361
7465676F 72793E53 75624361 7465676F 72793C2F 73756243 61746567 6F72793E 0A202020
203C6C69 62726172 793E4C69 62726172 793C2F6C 69627261 72793E0A 20202020 3C636174
65676F72 793E4361 7465676F 72793C2F 63617465 676F7279 3E0A2020 20203C69 73726349
643E4953 52433C2F 69737263 49643E0A 20202020 3C736F6E 67546974 6C653E54 7261636B
5469746C 653C2F73 6F6E6754 69746C65 3E0A2020 20203C74 656D706F 3E42504D 3C2F7465
6D706F3E 0A20203C 2F415357 473E0A3C 2F425746 584D4C3E
            "#,
        );
        let ixml = IxmlChunk::read(&mut buff).expect("error parsing ixmlchunk");
        print!("{:?}", ixml);
        // assert!(false);
    }
}
