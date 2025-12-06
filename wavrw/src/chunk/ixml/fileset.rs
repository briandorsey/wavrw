use alloc::collections::BTreeMap;
use core::fmt::Debug;

/// `FILE_SET` helps identify other members, when multiple files should be
/// treated as a group.
///
/// [IXML2021](https://wavref.til.cafe/spec/ixml2021/)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileSet {
    /// Total number of companion files in the same set of files.
    pub total_files: Option<String>,

    /// Multiple files which represent a single recording should have the same value
    /// in `family_uid`.
    pub family_uid: Option<String>,

    /// Non-unique text name for the file set.
    pub family_name: Option<String>,

    /// Origination index in a group of files.
    ///
    /// For mono files, use indexes 1 to n. for multi-poly files, use letters A, B etc.
    /// It is strongly recommended that dual poly recordings should use this tag.
    pub file_set_index: Option<String>,

    /// Additional tags found in the XML document beyond those listed in
    /// the iXML spec.
    pub extra: BTreeMap<String, String>,
}

impl FileSet {
    /// User friendly name of the element's key
    pub fn name(&self) -> String {
        "FILE_SET".to_string()
    }

    /// Create a new Aswg struct.
    pub fn new() -> Self {
        FileSet {
            total_files: None,
            family_uid: None,
            family_name: None,
            file_set_index: None,
            extra: BTreeMap::new(),
        }
    }

    /// Helper to set value by path.
    pub fn set(&mut self, path: &[String], value: String) {
        if let Some(first) = path.first() {
            match first.as_str() {
                "TOTAL_FILES" => self.total_files = Some(value),
                "FAMILY_UID" => self.family_uid = Some(value),
                "FAMILY_NAME" => self.family_name = Some(value),
                "FILE_SET_INDEX" => self.file_set_index = Some(value),
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
        push("TOTAL_FILES", &self.total_files);
        push("FAMILY_UID", &self.family_uid);
        push("FAMILY_NAME", &self.family_name);
        push("FILE_SET_INDEX", &self.file_set_index);

        for (k, v) in &self.extra {
            items.push((format!("{} (extra)", k), v.clone()));
        }

        Box::new(items.into_iter())
    }
}

impl Default for FileSet {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {

    // use super::*;
}
