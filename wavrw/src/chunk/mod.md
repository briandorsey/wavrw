RIFF WAVE chunk parsers and structs.

At a high level, each chunk module contains at least two structs, an inner
data struct and a wrapper created with [`KnownChunk<T>`][crate::KnownChunk]
type aliases. Ex: [`Fmt`] and [`FmtChunk`]. These type aliases are the primary
interface to the chunks when reading from a file.

For specifications and reference materials related to WAVE files, see the
sibling project: [Wav Reference book](https://wavref.til.cafe/)

TODO: write about architecture
