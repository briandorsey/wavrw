use alloc::collections::BTreeMap;
use core::fmt::Debug;

/// `HISTORY` tracking of a file's origins.
///
/// [IXML2021](https://wavref.til.cafe/spec/ixml2021/)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct History {
    /// Name given to this file when it was created. Using the `original_filename`
    /// metadata allows systems to track back to the original name if it changes.
    pub original_filename: Option<String>,

    /// Identification of the source of a derived file.
    pub parent_filename: Option<String>,

    /// Identification of the source of a derived file. Likely contains values
    /// from [`Ixml.file_uid`](crate::chunk::ixml::Ixml#structfield.file_uid) in another file.
    pub parent_uid: Option<String>,

    /// Additional tags found in the XML document beyond those listed in
    /// the ASWG spec.
    pub extra: BTreeMap<String, String>,
}

impl History {
    /// User friendly name of the element's key
    pub fn name(&self) -> String {
        "HISTORY".to_string()
    }

    /// Create a new Aswg struct.
    pub fn new() -> Self {
        History {
            original_filename: None,
            extra: BTreeMap::new(),
            parent_filename: None,
            parent_uid: None,
        }
    }

    /// Helper to set value by path.
    pub fn set(&mut self, path: &[String], value: String) {
        if let Some(first) = path.first() {
            match first.as_str() {
                "ORIGINAL_FILENAME" => self.original_filename = Some(value),
                "PARENT_FILENAME" => self.parent_filename = Some(value),
                "PARENT_UID" => self.parent_uid = Some(value),
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
        push("ORIGINAL_FILENAME", &self.original_filename);
        push("PARENT_FILENAME", &self.parent_filename);
        push("PARENT_UID", &self.parent_uid);

        for (k, v) in &self.extra {
            items.push((format!("{} (extra)", k), v.clone()));
        }

        Box::new(items.into_iter())
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {

    // use super::*;
}
