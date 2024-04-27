//! RIFF WAVE chunk parsers and structs.
//!
//! At a high level, each chunk module contains at least two structs, an inner
//! data struct and a wrapper created with [`KnownChunk<T>`][crate::KnownChunk]
//! type aliases. Ex: [`Fmt`] and [`FmtChunk`]. These type aliases are the primary
//! interface to the chunks when reading from a file.
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
pub mod wavl;

pub use bext::{Bext, BextChunk};
pub use cset::{Cset, CsetChunk, RiffCountryCode};
pub use cue::{Cue, CueChunk, CuePoint};
pub use data::{Data, DataChunk};
pub use fact::{Fact, FactChunk};
pub use fmt::{Fmt, FmtChunk, FormatTag};
pub use junk::{Fllr, FllrChunk, Junk, JunkChunk, Pad, PadChunk};
pub use md5::{Md5, Md5Chunk};
pub use plst::{Plst, PlstChunk, PlstSegment};
pub use riff::RiffChunk;
