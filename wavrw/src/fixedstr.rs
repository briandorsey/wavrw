#![doc = include_str!("fixedstr.md")]

use core::cmp::min;
use core::fmt::{Debug, Display, Formatter};
use core::str::FromStr;
use std::error::Error;

use binrw::io::{Read, Seek, SeekFrom};
use binrw::{BinRead, BinResult, BinWrite, Endian};

#[derive(Debug, Clone, PartialEq)]
/// Error when converting from a string would truncate
pub enum FixedStrError {
    /// Error when converting from a string would truncate
    Truncated {
        /// The fixed length (N) of [`FixedStr<N>`]
        limit: usize,
        /// The length of the string that would have been truncated
        len: usize,
    },
}

impl Error for FixedStrError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl Display for FixedStrError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Truncated { limit, len } => {
                write!(f, "truncated string of length {} at {}", len, limit)
            }
        }
    }
}

/// Null terminated fixed length strings (from BEXT for example).
///
/// `FixedStr` is mostly used via binrw's [`BinRead`] trait.
#[derive(Clone, PartialEq, Eq, Hash, BinWrite)]
pub struct FixedStr<const N: usize>([u8; N]);

// FixedStr design question: Should this really be FixedString instead of str?
// And perhaps more fully implement traits, similar to heapless::String
// (https://docs.rs/heapless/latest/heapless/struct.String.html)?

// FixedStr display design question: RIFF spec uses ""Z notation for fixed strings. Should we do the same?

impl<const N: usize> Debug for FixedStr<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        f.debug_tuple(&format!("FixedStr::<{}>", N))
            .field(&self.to_string())
            .finish()
    }
}

impl<const N: usize> Display for FixedStr<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "{}",
            String::from_utf8_lossy(&self.0).trim_end_matches('\0')
        )
    }
}

impl<const N: usize> FixedStr<N> {
    /// Constructor. WARNING: Truncates Strings to N!
    pub fn new(s: &str) -> FixedStr<N> {
        let mut array_tmp = [0u8; N];
        let l = min(s.len(), N);
        array_tmp[..l].copy_from_slice(&s.as_bytes()[..l]);
        FixedStr::<N>(array_tmp)
    }
}

impl<const N: usize> Default for FixedStr<N> {
    fn default() -> Self {
        FixedStr::<N>::new("")
    }
}

impl<const N: usize> FromStr for FixedStr<N> {
    type Err = FixedStrError;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        if s.len() > N {
            return Err(FixedStrError::Truncated {
                limit: N,
                len: s.len(),
            });
        }
        Ok(FixedStr::new(s))
    }
}

impl<const N: usize> BinRead for FixedStr<N> {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut values: [u8; N] = [0; N];
        let mut index = 0;

        loop {
            if index >= N {
                return Ok(Self(values));
            }
            let val = <u8>::read_options(reader, endian, ())?;
            if val == 0 {
                let offset = (N - index - 1).try_into();
                return match offset {
                    Ok(offset) => {
                        reader.seek(SeekFrom::Current(offset))?;
                        Ok(Self(values))
                    }
                    Err(err) => Err(binrw::Error::Custom {
                        pos: index as u64,
                        err: Box::new(err),
                    }),
                };
            }
            values[index] = val;
            index += 1;
        }
    }
}

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {

    use super::*;
    use crate::testing::hex_to_cursor;

    #[test]
    fn fixed_string() {
        let fs = FixedStr::<6>::new("abc");
        assert_eq!(6, fs.0.len());
        let s = fs.to_string();
        assert_eq!("abc".to_string(), s);
        assert_eq!(3, s.len());
        let new_fs = FixedStr::<6>::from_str(&s).unwrap();
        assert_eq!(fs, new_fs);
        assert_eq!(6, fs.0.len());
        assert_eq!(
            b"\0\0\0"[..],
            new_fs.0[3..6],
            "extra space after the string should be null bytes"
        );

        // Debug string
        println!("{:?}", fs);
        assert_eq!(format!("{:?}", fs), "FixedStr::<6>(\"abc\")");
    }

    #[test]
    fn fixed_string_long() {
        // strings longer than fixed size should get truncated

        // initializing with ::new() truncates without error
        let long_str = "this is a longer str";
        let fs = FixedStr::<6>::new(long_str);
        assert_eq!(fs.to_string(), "this i");

        // via FromStr returns an error
        let long_str = "this is a longer str";
        let err = FixedStr::<6>::from_str(long_str);
        assert_eq!(err, Err(FixedStrError::Truncated { limit: 6, len: 20 }));

        // via FromStr returns an error
        let err = "this is a longer str".parse::<FixedStr<6>>();
        assert_eq!(err, Err(FixedStrError::Truncated { limit: 6, len: 20 }));
    }

    #[test]
    fn parse_fixedstr_data_after_zero() {
        // REAPER had (still has?) a bug where data from other BEXT fields
        // can be left in a fixed lenth string field after the terminating
        // zero byte. This input is an example that starts with "REAPER"
        // but has part of a path string after the terminating zero.
        let mut buff = hex_to_cursor(
            "52454150 45520065 72732F62 7269616E 2F70726F 6A656374 732F7761 7672772F",
        );
        let fs = FixedStr::<32>::read_options(&mut buff, binrw::Endian::Big, ())
            .expect("error parsing FixedStr");
        assert_eq!(fs, FixedStr::<32>::from_str("REAPER").unwrap());
    }
}
