#![allow(dead_code)]
use std::io::Cursor;

use hex::decode;

pub(crate) fn hex_to_cursor(data: &str) -> Cursor<Vec<u8>> {
    let data = data.replace(' ', "");
    let data = data.replace('\n', "");
    let data = decode(data).expect("while decoding hex data from string");
    Cursor::new(data)
}
