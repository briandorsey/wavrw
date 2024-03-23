//! RIFF WAVE chunk parsers and structs.
//!
//! At a high level, each chunk module contains at least two structs, an inner data struct and a wrapper created with type aliases around [`KnownChunk<T>`][crate::KnownChunk]. Ex: [`FmtData`] and [`Fmt`]. These type aliases are the primary interface to the chunks when reading from a file.
//!
//! For specifications and reference materials related to WAVE files, see the
//! sibling project: [Wav Reference book](https://wavref.til.cafe/)
//!
//! TODO: write about architecture

pub mod adtl;
mod bext;
mod cset;
mod cue;
mod data;
mod fact;
mod fmt;
pub mod info;
mod junk;
mod md5;
mod plst;
mod riff;

pub use bext::{Bext, BextData};
pub use cset::{Cset, CsetCountryCode, CsetData};
pub use cue::{Cue, CueData, CuePoint};
pub use data::{Data, DataData};
pub use fact::{Fact, FactData};
pub use fmt::{Fmt, FmtData, FormatTag};
pub use junk::{Fllr, FllrData, Junk, JunkData, Pad, PadData};
pub use md5::{Md5, Md5Data};
pub use plst::{Plst, PlstData, PlstSegment};
pub use riff::Riff;
