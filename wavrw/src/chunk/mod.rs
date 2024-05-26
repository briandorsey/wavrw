//! RIFF WAVE chunk parsers and structs.
//!
//! At a high level, each chunk module contains at least two structs, an inner
//! data struct and a wrapper created with [`KnownChunk<T>`][crate::KnownChunk]
//! type aliases. Ex: [`fact::Fact`] and [`fact::FactChunk`]. These type aliases are the primary
//! interface to the chunks when reading from a file.
//!
//! For specifications and reference materials related to WAVE files, see the
//! sibling project: [Wav Reference book](https://wavref.til.cafe/)
//!
//! TODO: write about architecture

pub mod adtl;
pub mod bext;
pub mod cset;
pub mod cue;
pub mod data;
pub mod fact;
pub mod fmt;
pub mod info;
pub mod inst;
pub mod junk;
pub mod md5;
pub mod plst;
pub mod riff;
pub mod smpl;
pub mod wavl;
