use alloc::collections::BTreeMap;
use core::fmt::Debug;

/// `LOUDNESS` equivalent to the loudness fields from [`Bext`](crate::chunk::bext::Bext).
///
/// [IXML2021](https://wavref.til.cafe/spec/ixml2021/)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Loudness {
    /// Duplicated from bext chunk. iXML body equivalent to the `loudness_value` field from
    /// [`Bext`](crate::chunk::bext::Bext).
    pub loudness_value: Option<String>,

    /// Duplicated from bext chunk. iXML body equivalent to the `loudness_range` field from
    /// [`Bext`](crate::chunk::bext::Bext).
    pub loudness_range: Option<String>,

    /// Duplicated from bext chunk. iXML body equivalent to the `max_true_peak_level`
    /// field from [`Bext`](crate::chunk::bext::Bext).
    pub max_true_peak_level: Option<String>,

    /// Duplicated from bext chunk. iXML body equivalent to the `max_momentary_loudness`
    /// field from [`Bext`](crate::chunk::bext::Bext).
    pub max_momentary_loudness: Option<String>,

    /// Duplicated from bext chunk. iXML body equivalent to the `max_short_term_loudness`
    /// field from [`Bext`](crate::chunk::bext::Bext).
    pub max_short_term_loudness: Option<String>,

    /// Additional tags found in the XML document beyond those listed in
    /// the ASWG spec.
    pub extra: BTreeMap<String, String>,
}

impl Loudness {
    /// User friendly name of the element's key
    pub fn name(&self) -> String {
        "LOUDNESS".to_string()
    }

    /// Create a new Aswg struct.
    pub fn new() -> Self {
        Loudness {
            loudness_value: None,
            loudness_range: None,
            max_true_peak_level: None,
            max_momentary_loudness: None,
            max_short_term_loudness: None,
            extra: BTreeMap::new(),
        }
    }

    /// Helper to set value by path.
    pub fn set(&mut self, path: &[String], value: String) {
        if let Some(first) = path.first() {
            match first.as_str() {
                "LOUDNESS_VALUE" => self.loudness_value = Some(value),
                "LOUDNESS_RANGE" => self.loudness_range = Some(value),
                "MAX_TRUE_PEAK_LEVEL" => self.max_true_peak_level = Some(value),
                "MAX_MOMENTARY_LOUDNESS" => self.max_momentary_loudness = Some(value),
                "MAX_SHORT_TERM_LOUDNESS" => self.max_short_term_loudness = Some(value),
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
        push("LOUDNESS_VALUE", &self.loudness_value);
        push("LOUDNESS_RANGE", &self.loudness_range);
        push("MAX_TRUE_PEAK_LEVEL", &self.max_true_peak_level);
        push("MAX_MOMENTARY_LOUDNESS", &self.max_momentary_loudness);
        push("MAX_SHORT_TERM_LOUDNESS", &self.max_short_term_loudness);

        for (k, v) in &self.extra {
            items.push((format!("{} (extra)", k), v.clone()));
        }

        Box::new(items.into_iter())
    }
}

impl Default for Loudness {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {

    // use super::*;
}
