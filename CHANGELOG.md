# Changelog

Annotations: NEW, IMProved, FIX, DEPrecated, REMoved, SECurity

## [unreleased] - 

## [0.2.0] - 2024/06/09 

RIFF1994 specification supported: `smpl`, `inst`, new `INFO` subchunks, WAVEFORMATEX. 

- [NEW] - RIFF1994 updates [RIFF1994](https://wavref.til.cafe/spec/riff1994/)
  - [IMP] - Add new Languages and Dialects
  - [IMP] - Add new [INFO](https://wavref.til.cafe/chunk/info/) subchunks from RIFF1994: ISMP, IDIT
  - [NEW] - `smpl` chunk support. Information needed for use as a sampling instrument. [smpl](https://wavref.til.cafe/chunk/smpl/)
  - [NEW] - `inst` chunk support. Pitch, volume, and velocity for playback by sampler. [RIFF1994](https://wavref.til.cafe/chunk/inst/)
  - [IMP] - Support WAVEFORMATEX `fmt ` chunks
    - `fmt ` parsing now supports these formats: WAVE_FORMAT_PCM, WAVE_FORMAT_ADPCM, WAVE_FORMAT_DVI_ADPCM ( which also covers WAVE_FORMAT_IMA_PCM), and WAVE_FORMAT_UNKNOWN. Everything else with a valid Extended format should also parse correctly by falling back to a general Extended parser (extra fields returned as raw bytes in this case).
- Command Line Interface
  - [IMP] - view command: add -d alias for --format=detailed
- library internals
  - Move all chunks into submodules for consistency. 
  - [FIX] - FormatTag's TryFrom no longer relies on a binrw error (uses an error from std instead). 

## [0.1.0] - 2024/04/27

- [NEW] - Initial release. 
- [NEW] - Supports reading all [RIFF1991](https://wavref.til.cafe/spec/riff1991/) metadata chunks: RIFF, LIST-INFO, CSET, JUNK, fmt, data, LIST-adtl, info, cue, fact, plst, LIST-wavl, slnt. 
- [NEW] - `bext` chunk support. [Broadcast Wave Format](https://en.wikipedia.org/wiki/Broadcast_Wave_Format)
- [NEW] - MD5 of audio data chunk support. As specified by [BWFMetaEdit's Audio Data Checksums](https://mediaarea.net/BWFMetaEdit/md5)


