# Changelog

Annotations: NEW, IMProved, FIX, DEPrecated, REMoved, SECurity

## [unreleased] - 

- [NEW] - RIFF1994 updates [RIFF1994](https://wavref.til.cafe/spec/riff1994/)
  - [IMP] - add new Languages and Dialects
  - [IMP] - add new [INFO](https://wavref.til.cafe/chunk/info/) subchunks from RIFF1994: ISMP, IDIT
  - [NEW] - smpl chunk support. Information needed for use as a sampling instrument. [smpl](https://wavref.til.cafe/chunk/smpl/)
  - [NEW] - inst chunk support. Pitch, volume, and velocity for playback by sampler. [RIFF1994](https://wavref.til.cafe/chunk/inst/)

## [0.1.0] - 2024/04/27

- [NEW] - Initial release. 
- [NEW] - Supports reading all [RIFF1991](https://wavref.til.cafe/spec/riff1991/) metadata chunks: RIFF, LIST-INFO, CSET, JUNK, fmt, data, LIST-adtl, info, cue, fact, plst, LIST-wavl, slnt. 
- [NEW] - bext chunk support. [Broadcast Wave Format](https://en.wikipedia.org/wiki/Broadcast_Wave_Format)
- [NEW] - MD5 of audio data chunk support. As specified by [BWFMetaEdit's Audio Data Checksums](https://mediaarea.net/BWFMetaEdit/md5)


