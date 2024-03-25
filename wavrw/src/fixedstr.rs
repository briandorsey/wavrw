#![doc = include_str!("fixedstr.md")]

use alloc::string::FromUtf8Error;
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

    /// Embedded [`alloc::string::FromUtf8Error`];
    FromUtf8Error(FromUtf8Error),
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
            Self::FromUtf8Error(err) => write!(f, "{}", err),
        }
    }
}

impl From<FromUtf8Error> for FixedStrError {
    fn from(err: FromUtf8Error) -> Self {
        FixedStrError::FromUtf8Error(err)
    }
}

#[doc = include_str!("fixedstr.md")]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FixedStr<const N: usize>(String);
// This is only immutable because it would be a lot of work to correctly DeRef
// to the inner string while still enforcing the length constraint. Design
// quesion: is it worth the work? Maybe if it turns out to be annoying to work
// with them?

impl<const N: usize> Debug for FixedStr<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        f.debug_tuple(&format!("FixedStr::<{}>", N))
            .field(&self.to_string())
            .finish()
    }
}

impl<const N: usize> Display for FixedStr<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}", &self.0)
    }
}

impl<const N: usize> FixedStr<N> {
    /// Length of a [`FixedStr<N>`] is always N.
    pub fn len(&self) -> usize {
        N
    }

    /// A [`FixedStr`] is never empty or always empty, depending on N.
    pub const fn is_empty(&self) -> bool {
        !N == 0
    }

    /// Convert UTF-8 bytes into a `FixedStr`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use wavrw::fixedstr::FixedStr;
    ///
    /// let tea = vec![240, 159, 141, 181];
    /// let tea = FixedStr::<4>::from_utf8(tea)?;
    ///
    /// assert_eq!("🍵", tea.to_string());
    /// # Ok::<(), wavrw::fixedstr::FixedStrError>(())
    /// ```
    ///
    /// Invalid UTF-8:
    ///
    /// ```
    /// use wavrw::fixedstr::{FixedStr, FixedStrError};
    ///
    /// let data = vec![0, 159, 141, 181];
    ///
    /// let FixedStrError::FromUtf8Error(e) = FixedStr::<4>::from_utf8(data).unwrap_err()
    ///     else { unreachable!()};
    /// assert_eq!(e.utf8_error().valid_up_to(), 1);
    /// # Ok::<(), wavrw::fixedstr::FixedStrError>(())
    /// ```
    pub fn from_utf8(vec: Vec<u8>) -> Result<Self, FixedStrError> {
        if vec.len() > N {
            return Err(FixedStrError::Truncated {
                limit: N,
                len: vec.len(),
            });
        }
        let s = alloc::string::String::from_utf8(vec)?;
        let s = s.trim_end_matches('\0').to_string();
        Ok(Self(s))
    }

    /// Create a new [u8; N] from &self
    ///
    /// The array contains UTF-8 encoded bytes from the string followed by enough
    /// zero padding to fill the array.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use wavrw::fixedstr::FixedStr;
    /// use core::str::FromStr;
    ///
    /// let fs = FixedStr::<6>::from_str("abc").unwrap();
    /// let arr = fs.to_bytes();
    /// assert_eq!(arr.len(), 6);
    /// assert_eq!(arr, [97, 98, 99, 0, 0, 0]);
    /// ```
    pub fn to_bytes(&self) -> [u8; N] {
        let mut array_tmp = [0u8; N];
        let bytes = self.0.as_bytes();
        let l = min(bytes.len(), N);
        array_tmp[..l].copy_from_slice(&bytes[..l]);
        array_tmp
    }
}

impl<const N: usize> Default for FixedStr<N> {
    fn default() -> Self {
        FixedStr::<N>(String::new())
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
        Ok(FixedStr(s.to_string()))
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
                return match Self::from_utf8(values.to_vec()) {
                    Ok(fs) => Ok(fs),
                    Err(err) => Err(binrw::Error::Custom {
                        pos: index as u64,
                        err: Box::new(err),
                    }),
                };
            }
            let val = <u8>::read_options(reader, endian, ())?;
            if val == 0 {
                let offset = (N - index - 1).try_into();
                return match offset {
                    Ok(offset) => {
                        reader.seek(SeekFrom::Current(offset))?;
                        match Self::from_utf8(values.to_vec()) {
                            Ok(fs) => Ok(fs),
                            Err(err) => Err(binrw::Error::Custom {
                                pos: index as u64,
                                err: Box::new(err),
                            }),
                        }
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

impl<const N: usize> BinWrite for FixedStr<N> {
    type Args<'a> = ();

    fn write_options<W: std::io::prelude::Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        let bytes = self.0.as_bytes();
        let padding_size = N - bytes.len();
        bytes.write_options(writer, endian, args)?;
        for _ in 0..padding_size {
            b"\0".write_options(writer, endian, args)?;
        }
        Ok(())
    }
}

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {

    use super::*;
    use crate::testing::hex_to_cursor;
    use binrw::io::Cursor;

    #[test]
    fn fixed_string() {
        let fs = FixedStr::<6>("abc".to_string());
        assert_eq!(6, fs.len());
        let s = fs.to_string();
        assert_eq!("abc".to_string(), s);
        assert_eq!(3, s.len());
        let new_fs = FixedStr::<6>::from_str(&s).unwrap();
        assert_eq!(fs, new_fs);
        assert_eq!(6, fs.len());
        assert_eq!(
            new_fs.to_bytes()[3..6],
            b"\0\0\0"[..],
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
        // let long_str = "this is a longer str";
        // let fs = FixedStr::<6>::new(long_str);
        // assert_eq!(fs.to_string(), "this i");

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

    #[test]
    fn fixedstr_bytes_consistent() {
        let fs = FixedStr::<6>::from_str("abc").unwrap();
        let mut buff = Cursor::new(Vec::new());
        fs.write_le(&mut buff)
            .expect("failed to serialized with BinWrite");
        let v = buff.into_inner();
        assert_eq!(v.len(), 6);
        assert_eq!(v, vec![97, 98, 99, 0, 0, 0]);

        assert_eq!(
            fs.to_bytes(),
            *v,
            "to_bytes() and serialzation should be the same"
        );
    }
}
