use alloc::collections::BTreeMap;
use core::fmt::Debug;

/// `SPEED` the ratio of recorded speed to target playback speed
///
/// [IXML2021](https://wavref.til.cafe/spec/ixml2021/)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Speed {
    /// User comments about speed.
    pub note: Option<String>,

    /// Speed at which the material will be replayed.
    pub master_speed: Option<String>,
    /// Speed of this recording.
    ///
    /// Might be different than `master_speed`. In slow motion for
    /// example, `master_speed` might be "24/1", and `current_speed` might be "48/1".
    pub current_speed: Option<String>,

    /// Timecode frame rate used during record. Ex: 24/1, 25/1, 30000/1001, 24000/1001, 30/1
    pub timecode_rate: Option<String>,

    /// Timecode Drop Frame used during record.
    ///
    /// Ex: DF for Drop Frame, NDF for Non Drop Frame. Defaults to NDF.
    /// Useful to calculate H:m:s:f format timecode.
    pub timecode_flag: Option<String>,

    /// Duplicated from
    /// [`fmt::FmtExtended.samples_per_sec`](crate::chunk::fmt::FmtExtended#structfield.samples_per_sec`)
    /// "for convenience".
    pub file_sample_rate: Option<String>,

    /// Duplicated from
    /// [`fmt::FmtExtended.bits_per_sample`](crate::chunk::fmt::FmtExtended#structfield.bits_per_sample`)
    /// "for convenience".
    pub audio_bit_depth: Option<String>,

    /// True wordclock speed of the A-D convertors used during the recording.
    pub digitizer_sample_rate: Option<String>,

    /// Duplicated from
    /// [`bext::Bext.time_reference`](crate::chunk::bext::Bext#structfield.time_reference)
    /// "for convenience".
    pub timestamp_samples_since_midnight_hi: Option<String>,

    /// Duplicated from
    /// [`bext::Bext.time_reference`](crate::chunk::bext::Bext#structfield.time_reference)
    /// "for convenience".
    pub timestamp_samples_since_midnight_lo: Option<String>,

    /// Sample rate used to calculate the timestamp, and which must be used in mathematic calculations to recover the timecode timestamp for the file.
    pub timestamp_sample_rate: Option<String>,

    /// Additional tags found in the XML document beyond those listed in
    /// the iXML spec.
    pub extra: BTreeMap<String, String>,
}

impl Speed {
    /// User friendly name of the element's key
    pub fn name(&self) -> String {
        "SPEED".to_string()
    }

    /// Create a new Aswg struct.
    pub fn new() -> Self {
        Speed {
            note: None,
            master_speed: None,
            current_speed: None,
            timecode_rate: None,
            timecode_flag: None,
            file_sample_rate: None,
            audio_bit_depth: None,
            digitizer_sample_rate: None,
            timestamp_samples_since_midnight_hi: None,
            timestamp_samples_since_midnight_lo: None,
            timestamp_sample_rate: None,
            extra: BTreeMap::new(),
        }
    }

    /// Helper to set value by path.
    pub fn set(&mut self, path: &[String], value: String) {
        if let Some(first) = path.first() {
            match first.as_str() {
                "NOTE" => self.note = Some(value),
                "MASTER_SPEED" => self.master_speed = Some(value),
                "CURRENT_SPEED" => self.current_speed = Some(value),
                "TIMECODE_RATE" => self.timecode_rate = Some(value),
                "TIMECODE_FLAG" => self.timecode_flag = Some(value),
                "FILE_SAMPLE_RATE" => self.file_sample_rate = Some(value),
                "AUDIO_BIT_DEPTH" => self.audio_bit_depth = Some(value),
                "DIGITIZER_SAMPLE_RATE" => self.digitizer_sample_rate = Some(value),
                "TIMESTAMP_SAMPLES_SINCE_MIDNIGHT_HI" => {
                    self.timestamp_samples_since_midnight_hi = Some(value);
                }
                "TIMESTAMP_SAMPLES_SINCE_MIDNIGHT_LO" => {
                    self.timestamp_samples_since_midnight_lo = Some(value);
                }
                "TIMESTAMP_SAMPLE_RATE" => self.timestamp_sample_rate = Some(value),
                &_ => {
                    self.extra.insert(path.join("/"), value);
                }
            }
        }
    }

    /// Iterate over fields and values as (String, String)
    pub fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        let mut items: Vec<(String, String)> = Vec::new();
        let mut push = |key: &str, value: &Option<String>| {
            if let Some(val) = value {
                items.push((key.to_string(), val.clone()));
            }
        };
        push("NOTE", &self.note);
        push("MASTER_SPEED", &self.master_speed);
        push("CURRENT_SPEED", &self.current_speed);
        push("TIMECODE_RATE", &self.timecode_rate);
        push("TIMECODE_FLAG", &self.timecode_flag);
        push("FILE_SAMPLE_RATE", &self.file_sample_rate);
        push("AUDIO_BIT_DEPTH", &self.audio_bit_depth);
        push("DIGITIZER_SAMPLE_RATE", &self.digitizer_sample_rate);
        push(
            "TIMESTAMP_SAMPLES_SINCE_MIDNIGHT_HI",
            &self.timestamp_samples_since_midnight_hi,
        );
        push(
            "TIMESTAMP_SAMPLES_SINCE_MIDNIGHT_LO",
            &self.timestamp_samples_since_midnight_lo,
        );
        push("TIMESTAMP_SAMPLE_RATE", &self.timestamp_sample_rate);

        for (k, v) in &self.extra {
            items.push((format!("{} (extra)", k), v.clone()));
        }

        Box::new(items.into_iter())
    }
}

impl Default for Speed {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {

    // use super::*;
}
