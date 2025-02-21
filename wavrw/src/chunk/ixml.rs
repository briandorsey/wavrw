//! `iXML` Production workflow file & project metadata.  [IXML2021](https://wavref.til.cafe/spec/ixml2021/)

use core::fmt::Debug;

use binrw::{binrw, helpers};

use crate::{FourCC, KnownChunk, KnownChunkID, Summarizable};

// iXML, based on http://www.gallery.co.uk/ixml/

/// `iXML` Production workflow file & project metadata.  [IXML2021](https://wavref.til.cafe/spec/ixml2021/)
#[binrw]
#[brw(little)]
#[br(import(_size: u32))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ixml {
    /// temporary....  TODO
    #[br(parse_with = helpers::until_eof)]
    pub raw_bytes: Vec<u8>,
}

impl KnownChunkID for Ixml {
    const ID: FourCC = FourCC(*b"iXML");
}

impl Ixml {
    fn new() -> Ixml {
        Ixml { raw_bytes: vec![] }
    }
}

impl Default for Ixml {
    fn default() -> Self {
        Ixml::new()
    }
}

/// `iXML` Production workflow file & project metadata.  [IXML2021](https://wavref.til.cafe/spec/ixml2021/)
pub type IxmlChunk = KnownChunk<Ixml>;

impl Summarizable for Ixml {
    fn summary(&self) -> String {
        format!("{} bytes of data", self.raw_bytes.len())
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
        }
    }
}

/// Iterate over fields as tuple of Strings (name, value).
#[derive(Debug)]
pub struct IxmlDataIterator<'a> {
    data: &'a Ixml,
    index: usize,
}

impl Iterator for IxmlDataIterator<'_> {
    type Item = (String, String);
    fn next(&mut self) -> Option<(String, String)> {
        self.index += 1;
        match self.index {
            1 => Some((
                "raw_bytes".to_string(),
                format!("{} bytes of data", self.data.raw_bytes.len()),
            )),
            _ => None,
        }
    }
}

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {
    // use binrw::BinRead;

    // use super::*;
    use crate::testing::hex_to_cursor;

    #[test]
    fn parse_ixml() {
        // example bext chunk data from BWF MetaEdit
        let mut _buff = hex_to_cursor(
            r#"62657874 67020000 44657363 72697074 696F6E00 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 4F726967 696E6174 6F720000 
            00000000 00000000 00000000 00000000 00000000 4F726967 696E6174 
            6F725265 66657265 6E636500 00000000 00000000 00000000 32303036 
            2F30312F 30323033 3A30343A 30353930 00000000 00000200 060A2B34 
            01010101 01010210 13000000 00FF122A 69370580 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 6400C800 2C019001 F4010000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 0000436F 
            64696E67 48697374 6F7279"#,
        );
        // TODO: thread 'chunk::ixml::test::parse_ixml' panicked at wavrw/src/chunk/ixml.rs:124:47:
        // error parsing ixmlchunk: assertion failed: `id == T :: ID` at 0x0
        // let ixml = IxmlChunk::read(&mut buff).expect("error parsing ixmlchunk");
        // print!("{:?}", ixml);
    }
}
