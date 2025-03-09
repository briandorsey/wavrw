use core::fmt;
use core::fmt::{Debug, Display, Formatter};

/// Represents iXML `TAKE_TYPE` dictionary
///
/// the `TAKE_TYPE` tag contains a comma delimited list of values from the
/// dictionary below, following recommendations by AFSI. This tag supercedes
/// the older explicit tags for `WILD_TRACK`, `NO_GOOD`, `FALSE_START` and
/// replaces them with a comma delimited list from the dictionary below
/// which can be expanded in the future. If this tag is absent or empty
/// or contains just the word `DEFAULT`, the take should be considered to be
/// standard `TAKE`.
///
/// Enum variants as defined by
/// [the iXML spec](https://www.gallery.co.uk/ixml/taketype_dictionary.html)
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
    /// Create a `Vec<TakeType>` from a comma delimited String.
    ///
    /// Defers to [`TakeType::from<String>`] after splitting the string.
    pub fn from_multiple(value: String) -> Vec<Self> {
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
#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {

    use super::*;
    use crate::chunk::ixml::Ixml;

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
}
