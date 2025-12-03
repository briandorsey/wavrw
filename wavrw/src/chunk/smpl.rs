//! `smpl` Information needed for use as a sampling instrument. [RIFF1994](https://wavref.til.cafe/chunk/smpl/)

use core::fmt::Debug;

use binrw::binrw;

use crate::{FourCC, KnownChunk, KnownChunkID, Summarizable};

/// Loop details from a `smpl` chunk.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SmplLoop {
    /// Name of the loop as [`CuePoint::name`][crate::chunk::cue::CuePoint::name].
    ///
    /// Identifies the unique 'name' of the loop. This field may correspond with
    /// a name stored in the `Cue` chunk. The name data is stored in the `adtl`
    /// chunk.
    pub identifier: u32,

    /// Loop type.
    ///
    /// Specifies the loop type: 0 - Loop forward (normal). 1 - Alternating
    /// loop (forward/backward). 2 - Loop backward. 3-31 - reserved for future
    /// standard types. 32-? - sampler specific types (manufacturer defined).
    pub loop_type: u32,

    /// Specifies the startpoint of the loop in samples.
    pub start: u32,

    /// Specifies the endpoint of the loop in samples
    ///
    /// This sample will also be played.
    pub end: u32,

    /// Fractional area between samples.
    ///
    /// Allows fine-tuning for loop fractional areas between samples. Values
    /// range from 0x00000000 to 0xFFFFFFFF. A value of 0x80000000 represents
    /// 1/2 of a sample length.
    pub fraction: u32,

    /// Specifies the number of times to play the loop.
    ///
    /// A value of 0 specifies an infinite sustain loop.
    pub play_count: u32,
}

/// `smpl` Information needed for use as a sampling instrument. [RIFF1994](https://wavref.til.cafe/chunk/smpl/)
#[binrw]
#[brw(little)]
#[br(import(_size: u32))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Smpl {
    /// MIDI Manufacturer's Association Manufacturer code.
    ///
    /// Specifies the MMA Manufacturer code for the intended target device.
    /// The high byte indicates the number of low order bytes (1 or 3) that
    /// are valid for the manufacturer code. For example, this value will be
    /// 0x01000013 for Digidesign (the MMA Manufacturer code is one byte, 0x13);
    /// whereas 0x03000041 identifies Microsoft (the MMA Manufacturer code
    /// is three bytes, 0x00 0x00 0x41). If the sample is not intended for a
    /// specific manufacturer, then this field should be set to zero.
    pub manufacturer: u32,

    /// Manufacturer product code.
    ///
    /// Specifies the Product code of the intended target device for the
    /// manufacturer. If the sample is not intended for a specific manufacturer's
    /// product, then this field should be set to zero.
    pub product: u32,

    /// Period of one sample in nanoseconds.
    ///
    /// Specifies the period of one sample in nanoseconds (normally 1/
    /// `samples_per_second` from the WAVEFORMAT structure for the RIFF WAVE file
    /// -- however, this field allows fine tuning). For example, 44.1 kHz would
    /// be specified as 22675 (0x00005893).
    pub sample_period: u32,

    /// Pitch of the sample as MIDI note.
    ///
    /// Specifies the MIDI note which will replay the sample at original pitch.
    /// This value ranges from 0 to 127 (a value of 60 represents Middle C as
    /// defined by the MMA).
    pub midi_unity_note: u32,

    /// Fine tune pitch as fraction of a semitone.
    ///
    /// Specifies the fraction of a semitone up from the specified
    /// `midi_unity_note. A value of 0x80000000 is 1/2 semitone (50 cents); a
    /// value of 0x00000000 represents no fine tuning between semitones.
    pub midi_pitch_fraction: u32,

    /// SMPTE time format.
    ///
    /// Specifies the SMPTE time format used in the `smpte_offset` field. Possible
    /// values are (unrecognized formats should be ignored): 0 - specifies
    /// no SMPTE offset (`smpte_offset` should also be zero). 24 - specifies 24
    /// frames per second. 25 - specifies 25 frames per second. 29 - specifies
    /// 30 frames per second with frame dropping ('30 drop'). 30 - specifies 30
    /// frames per second.
    pub smpte_format: u32,

    /// SMTPE time offset.
    ///
    /// Specifies a time offset for the sample if it is to be syncronized or
    /// calibrated according to a start time other than 0. The format of this
    /// value is 0xhhmmssff. hh is a signed Hours value [-23..23]. mm is an
    /// unsigned Minutes value [0..59]. ss is unsigned Seconds value [0..59]. ff
    /// is an unsigned value [0..(`smpte_format` - 1)].
    pub smpte_offset: u32,

    /// Count of sample loops (for serialization/deserialization)
    ///
    /// Specifies the number (count) of <sample-loop> records that are contained
    /// in the <smpl> chunk. The <sample-loop> records are stored immediately
    /// following the sampler_data field.
    #[br(temp)]
    #[bw(try_calc = sample_loops.len().try_into())]
    pub sample_loop_count: u32,

    /// Size of optional sampler data in bytes.
    ///
    /// Specifies the size in bytes of the optional <sampler_data>. Sampler
    /// specific data is stored imediately following the <sample-loop> records.
    /// The sampler_data field will be zero if no extended sampler specific
    /// information is stored in the <smpl> chunk.
    #[br(temp)]
    #[bw(try_calc = sampler_data.len().try_into())]
    pub sampler_data_size: u32,

    /// A series of [`SmplLoop`]s.
    #[br(count = sample_loop_count)]
    pub sample_loops: Vec<SmplLoop>,

    /// Sampler specific data bytes.
    #[br(count = sampler_data_size)]
    pub sampler_data: Vec<u8>,
}

impl KnownChunkID for Smpl {
    const ID: FourCC = FourCC(*b"smpl");
}

impl Smpl {
    fn new() -> Self {
        Smpl {
            manufacturer: 0,
            product: 0,
            sample_period: 0,
            midi_unity_note: 0,
            midi_pitch_fraction: 0,
            smpte_format: 0,
            smpte_offset: 0,
            sample_loops: Vec::new(),
            sampler_data: Vec::new(),
        }
    }
}

impl Default for Smpl {
    fn default() -> Self {
        Self::new()
    }
}

/// `smpl` Information needed for use as a sampling instrument. [RIFF1994](https://wavref.til.cafe/chunk/smpl/)
pub type SmplChunk = KnownChunk<Smpl>;

impl Summarizable for Smpl {
    fn summary(&self) -> String {
        let label = match self.sample_loops.len() {
            1 => "loop",
            _ => "loops",
        };
        format!("{} {label}", self.sample_loops.len())
    }

    fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        let mut items: Vec<(String, String)> = Vec::with_capacity(11);
        items.push(("manufacturer".to_string(), self.manufacturer.to_string()));
        items.push(("product".to_string(), self.product.to_string()));
        // consider adding comparison to expected rate
        items.push(("sample_period".to_string(), self.sample_period.to_string()));
        items.push((
            "midi_unity_note".to_string(),
            self.midi_unity_note.to_string(),
        ));
        items.push((
            "midi_pitch_fraction".to_string(),
            self.midi_pitch_fraction.to_string(),
        ));
        items.push(("smpte_format".to_string(), self.smpte_format.to_string()));
        items.push(("smpte_offset".to_string(), self.smpte_offset.to_string()));
        items.push((
            "sample_loop_count".to_string(),
            self.sample_loops.len().to_string(),
        ));
        items.push((
            "sampler_data_size".to_string(),
            self.sampler_data.len().to_string(),
        ));

        // add summary of each sample loop
        items.push((
            "loop identifier".to_string(),
            format!(
                "{:>5}, {:>10}, {:>10}, {:>10}, {:>5}",
                "type", "start", "end", "fraction", "play"
            ),
        ));
        for sample_loop in &self.sample_loops {
            items.push((
                format!("{}", sample_loop.identifier),
                format!(
                    "{:5}, {:10}, {:10}, {:10}, {:5}",
                    sample_loop.loop_type,
                    sample_loop.start,
                    sample_loop.end,
                    sample_loop.fraction,
                    sample_loop.play_count
                ),
            ));
        }

        Box::new(items.into_iter())
    }

    // fn item_summary_header(&self) -> String {
    //     "name: position, chunk_id, chunk_start, block_start, sample_offset".to_string()
    // }

    // fn name(&self) -> String {
    //     self.id().to_string().trim().to_string()
    // }
}

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {
    use binrw::BinRead;

    use super::*;
    use crate::testing::hex_to_cursor;

    #[test]
    fn parse_smpl() {
        // example bext chunk data from BWF MetaEdit
        let mut buff = hex_to_cursor(
            r#"736D706C 6C000000 00000000 00000000 93580000 00000000 00000000 00000000 00000000 03000000 00000000 01000000 00000000 00000000 08500000 00000000 00000000 02000000 00000000 00000000 00000850 00000000 00000000 03000000 00000000 00000000 00000000 00000000 00000000"#,
        );
        let smpl = SmplChunk::read(&mut buff).expect("error parsing smpl chunk");
        print!("{:?}", smpl);
        assert_eq!(smpl.data.manufacturer, 0);
        assert_eq!(smpl.data.product, 0);
        assert_eq!(smpl.data.sample_period, 22675);
        assert_eq!(smpl.data.sample_loops.len(), 3);
        assert_eq!(
            smpl.data.sample_loops[1],
            SmplLoop {
                identifier: 2,
                loop_type: 0,
                start: 0,
                end: 1342701568,
                fraction: 0,
                play_count: 0
            },
        );
        assert_eq!(smpl.extra_bytes.len(), 0);
    }
}
