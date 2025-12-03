//! `inst` Pitch, volume, and velocity for playback by sampler. [RIFF1994](https://wavref.til.cafe/chunk/inst/)

use binrw::binrw;

use crate::{FourCC, KnownChunk, KnownChunkID, Summarizable};

/// `inst` Pitch, volume, and velocity for playback by sampler. [RIFF1994](https://wavref.til.cafe/chunk/inst/)
#[binrw]
#[brw(little)]
#[br(import(_size: u32))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Inst {
    /// MIDI note of the sample.
    ///
    /// MIDI note number that corresponds to the unshifted pitch of the sample.
    /// Valid values range from 0 to 127.
    pub unshifted_note: u8,

    /// Pitch shift adjustment in cents.
    ///
    /// Pitch shift adjustment in cents. (or 100ths of a semitone) needed to
    /// hit `unshifted_note` value exactly.  `fine_tune` can be used to compensate
    /// for tuning errors in the sampling process. Valid values range from -50
    /// to 50.
    pub fine_tune: i8,

    /// Suggested volume setting for the sample in decibels.
    ///
    /// A value of zero decibels suggests no change in the volume. A value of -6
    /// decibels suggests reducing the amplitude of the sample by two.
    pub gain: i8,

    /// Suggested lowest usable MIDI note number range of the sample.
    ///
    /// Valid values range from 0 to 127.
    pub low_note: u8,

    /// Suggested highest usable MIDI note number range of the sample.
    ///
    /// Valid values range from 0 to 127.
    pub high_note: u8,

    /// Suggested lowest usable MIDI velocity range of the sample.
    ///
    ///  Valid values range from 0 to 127.
    pub low_velocity: u8,

    /// Suggested highest usable MIDI velocity range of the sample.
    ///
    ///  Valid values range from 0 to 127.
    pub high_velocity: u8,
}

impl KnownChunkID for Inst {
    const ID: FourCC = FourCC(*b"inst");
}

impl Summarizable for Inst {
    fn summary(&self) -> String {
        format!(
            "note: {} ({}-{}), gain: {}, velocity: {}-{}",
            self.unshifted_note,
            self.low_note,
            self.high_note,
            self.gain,
            self.low_velocity,
            self.high_velocity
        )
    }

    fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        Box::new(self.into_iter())
    }

    fn item_summary_header(&self) -> String {
        "pitch, volume, and velocity for playback by sampler".into()
    }
}

impl<'a> IntoIterator for &'a Inst {
    type Item = (String, String);
    type IntoIter = InstIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        InstIterator {
            data: self,
            index: 0,
        }
    }
}

/// Iterate over fields as tuple of Strings (name, value).
#[derive(Debug)]
pub struct InstIterator<'a> {
    data: &'a Inst,
    index: usize,
}

impl Iterator for InstIterator<'_> {
    type Item = (String, String);
    fn next(&mut self) -> Option<(String, String)> {
        self.index += 1;
        match self.index {
            1 => Some((
                "unshifted_note".to_string(),
                self.data.unshifted_note.to_string(),
            )),
            2 => Some(("fine_tune".to_string(), self.data.fine_tune.to_string())),
            3 => Some(("gain".to_string(), self.data.gain.to_string())),
            4 => Some(("low_note".to_string(), self.data.low_note.to_string())),
            5 => Some(("high_note".to_string(), self.data.high_note.to_string())),
            6 => Some((
                "low_velocity".to_string(),
                self.data.low_velocity.to_string(),
            )),
            7 => Some((
                "high_velocity".to_string(),
                self.data.high_velocity.to_string(),
            )),
            _ => None,
        }
    }
}

/// `inst` Pitch, volume, and velocity for playback by sampler. [RIFF1994](https://wavref.til.cafe/chunk/inst/)
pub type InstChunk = KnownChunk<Inst>;

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {
    use binrw::BinRead;

    use super::*;
    use crate::testing::hex_to_cursor;

    #[test]
    fn parse_inst() {
        let mut buff = hex_to_cursor("696E7374 07000000 0C00000C 0C017F");
        let expected = InstChunk {
            offset: Some(0),
            size: 7,
            data: Inst {
                unshifted_note: 12,
                fine_tune: 0,
                gain: 0,
                low_note: 12,
                high_note: 12,
                low_velocity: 1,
                high_velocity: 127,
            },
            extra_bytes: vec![],
        };

        let chunk = InstChunk::read(&mut buff).expect("error parsing inst chunk");
        assert_eq!(chunk, expected);
    }
}
