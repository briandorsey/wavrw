use alloc::collections::BTreeMap;
use core::fmt;
use core::fmt::{Debug, Display, Formatter};

/// iXML `SYNC_POINT_TYPE` dictionary (enum).
///
/// Can be either RELATIVE or ABSOLUTE, which represents a sample frames count
/// from the start of the file, or an absolute sample count since midnight.
///
/// Enum variants as defined by
/// [the iXML spec](http://www.gallery.co.uk/ixml/)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SyncPointType {
    /// Count sample frames from start of this file.
    Relative,
    /// Count sample frames from midnight.
    Absolute,
    /// Value outside of iXML spec.
    Custom(String),
}

impl Display for SyncPointType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let out = match self {
            SyncPointType::Relative => "RELATIVE",
            SyncPointType::Absolute => "ABSOLUTE",
            SyncPointType::Custom(value) => value,
        };
        f.write_str(out)?;
        Ok(())
    }
}

impl From<String> for SyncPointType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "RELATIVE" => Self::Relative,
            "ABSOLUTE" => Self::Absolute,
            &_ => Self::Custom(value),
        }
    }
}

/// iXML `SYNC_POINT_FUNCTION` dictionary (enum).
///
/// The purpose of the sync point or event.
///
/// "...functions can add value to a simple indication of sample count, offering some functional purpose encoded in a machine readable format, for example, in order to automatically skip over the pre-record, or auto play and stop, for cart-machine type playback situations."
///
/// Enum variants as defined by
/// [the iXML spec](http://www.gallery.co.uk/ixml/function_dictionary.html)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SyncPointFunction {
    /// Enable software to begin playback from the point where the record button
    /// was actually pressed.
    PreRecordSampleCount,

    /// No description in spec.
    SlateGeneric,

    /// No description in spec.
    HeadSlate,

    /// No description in spec.
    MarkerGeneric,

    /// Playback should automatically start when the user selects this marker.
    MarkerAutoplay,

    /// Playback should automatically start when user selects this marker, then stop.
    ///
    /// Playback should start when the user selects this marker, then automatically
    /// stop when reaching [`SyncPoint::event_duration`] samples after the Sync point.
    MarkerAutoplayStop,

    /// Playback should automatically start when user selects this marker, then loop.
    ///
    /// Playback should automatically start when the user selects this marker,
    /// then automatically loop back to the start when reaching [`SyncPoint::event_duration`]
    /// samples after the Sync point.
    MarkerAutoplayLoop,

    /// Used in FileSet groups, to indicate offset of this file from the start of the group.
    ///    
    /// It MUST be a RELATIVE type, and the File Set must include the tag
    /// <FILE_SET_START_TIME_HI> and <FILE_SET_START_TIME_LO> to anchor the entire
    /// group. If this tag is missing, a backup would be to iterate the file set and
    /// pick the earliest file, then take that file's timestamp, less that file's
    /// `group_offset` (or zero) as the start time for the Group. This information could
    /// also be implied by comparing the timestamps of individual group members.
    GroupOffset,

    /// Value outside of iXML spec.
    Custom(String),
}

impl Display for SyncPointFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let out = match self {
            SyncPointFunction::PreRecordSampleCount => "PRE_RECORD_SAMPLECOUNT",
            SyncPointFunction::SlateGeneric => "SLATE_GENERIC",
            SyncPointFunction::HeadSlate => "HEAD_SLATE",
            SyncPointFunction::MarkerGeneric => "MARKER_GENERIC",
            SyncPointFunction::MarkerAutoplay => "MARKER_AUTOPLAY",
            SyncPointFunction::MarkerAutoplayStop => "MARKER_AUTOPLAYSTOP",
            SyncPointFunction::MarkerAutoplayLoop => "MARKER_AUTOPLAYLOOP",
            SyncPointFunction::GroupOffset => "GROUP_OFFSET",
            SyncPointFunction::Custom(value) => value,
        };
        f.write_str(out)?;
        Ok(())
    }
}

impl From<String> for SyncPointFunction {
    fn from(value: String) -> Self {
        match value.as_str() {
            "PRE_RECORD_SAMPLECOUNT" => Self::PreRecordSampleCount,
            "SLATE_GENERIC" => Self::SlateGeneric,
            "HEAD_SLATE" => Self::HeadSlate,
            "MARKER_GENERIC" => Self::MarkerGeneric,
            "MARKER_AUTOPLAY" => Self::MarkerAutoplay,
            "MARKER_AUTOPLAYSTOP" => Self::MarkerAutoplayStop,
            "MARKER_AUTOPLAYLOOP" => Self::MarkerAutoplayLoop,
            "GROUP_OFFSET" => Self::GroupOffset,
            &_ => Self::Custom(value),
        }
    }
}

/// `SYNC_POINT_LIST` Sample based counts which represents sync points (markers and regions) for this recording.
/// [IXML2021](https://wavref.til.cafe/spec/ixml2021/)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SyncPoint {
    /// Determintes how the `SyncPoint` offset is calulated.
    ///
    /// Sample frames count from the start of the file (relative), or an absolute
    /// sample count since midnight.
    pub sync_point_type: Option<SyncPointType>,

    /// The purpose of the sync point or event.
    pub function: Option<SyncPointFunction>,

    /// Text comment about this `SyncPoint`.
    pub comment: Option<String>,

    /// Sample frames offset (low bytes).
    pub low: Option<String>,

    /// Sample frames offset (high bytes).
    pub high: Option<String>,

    /// If set non-zero, the sync point is a region with a defined start and
    /// stop point/duration. A value of 0 in `event_duration` implies a
    /// non-duration (marker) or unknown duration event.
    pub event_duration: Option<String>,

    /// Additional tags found in the XML document beyond those listed in
    /// the iXML spec.
    pub extra: BTreeMap<String, String>,
}

impl SyncPoint {
    /// User friendly name of the element's key
    pub fn name(&self) -> String {
        "SYNC_POINT".to_string()
    }

    /// Create a new Aswg struct.
    pub fn new() -> Self {
        SyncPoint {
            sync_point_type: None,
            function: None,
            comment: None,
            low: None,
            high: None,
            event_duration: None,
            extra: BTreeMap::new(),
        }
    }

    /// Helper to set value by path.
    pub fn set(&mut self, path: &[String], value: String) {
        if let Some(first) = path.first() {
            match first.as_str() {
                "SYNC_POINT_TYPE" => self.sync_point_type = Some(value.into()),
                "SYNC_POINT_FUNCTION" => self.function = Some(value.into()),
                "SYNC_POINT_COMMENT" => self.comment = Some(value),
                "SYNC_POINT_LOW" => self.low = Some(value),
                "SYNC_POINT_HIGH" => self.high = Some(value),
                "SYNC_POINT_EVENT_DURATION" => self.event_duration = Some(value),
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

        if let Some(sp_type) = &self.sync_point_type {
            push("SYNC_POINT_TYPE", &Some(sp_type.to_string()));
        }
        if let Some(sp_function) = &self.function {
            push("SYNC_POINT_FUNCTION", &Some(sp_function.to_string()));
        }
        push("SYNC_POINT_COMMENT", &self.comment);
        push("SYNC_POINT_LOW", &self.low);
        push("SYNC_POINT_HIGH", &self.high);
        push("SYNC_POINT_EVENT_DURATION", &self.event_duration);

        for (k, v) in &self.extra {
            items.push((format!("{} (extra)", k), v.clone()));
        }

        Box::new(items.into_iter())
    }
}

impl Default for SyncPoint {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {

    // use super::*;
}
