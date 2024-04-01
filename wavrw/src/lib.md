Read (and someday write) wave audio file chunks with a focus on metadata.

This is the API reference documentation, it is a bit dry.

Iterate over all chunk objects from a file, returns [`SizedChunkEnum`]s with
convenience methods exposed via the [`SizedChunk`] trait:

```
# use std::fs::File;
# use std::io::BufReader; 
use wavrw::{Summarizable, SizedChunk}; 

let file = File::open("../test_wavs/example_a.wav")?;
let file = BufReader::new(file);
let mut wave = wavrw::Wave::from_reader(file)?;

for result in wave.iter_chunks() {
    match result {
        Ok(chunk) => {
            println!(
                "{:12} {:10} {}",
                chunk.name(),
                chunk.size(),
                chunk.summary()
            )
        },
        Err(err) => {
            println!("{:12} {}", "ERROR".to_string(), err)
        }
    }
}
# Ok::<(), wavrw::WaveError>(())
```

Or parse a single chunk from a buffer:

```
# use binrw::BinRead;
# use wavrw::testing::hex_to_cursor;
# let mut buff = hex_to_cursor("666D7420 10000000 01000100 80BB0000 80320200 03001800");
use wavrw::{SizedChunkEnum, ChunkID, Summarizable, FourCC};

let chunk = SizedChunkEnum::read(&mut buff).unwrap();

// Use methods from SizedChunk trait on any chunk
assert_eq!(chunk.id(), FourCC(*b"fmt "));
assert_eq!(chunk.summary(), "PCM (0x0001), 1 chan, 24/48000".to_string());

// Or match on type and handle various chunks individually
match chunk {
    SizedChunkEnum::Fmt(fmt) => println!("sample rate: {}", fmt.data.samples_per_sec),
    _ => ()
}
```


NOTE: Many WAVE chunk specifications assume or specify ASCII strings. This
library parses ASCII strings as UTF8 encoded strings instead. All ASCII
characters are valid UTF8, and writing UTF8 strings appears to be common
practice in applications which write metadata.

WARNING: This library does not attempt to interpret strings according to code
page settings specified via CSET. Setting character set information in CSET
chunks appears to be very rare, however if a file *did* specify an extended
codepage, text would likely be misinterpreted when decoded as UTF8. If you
do run into this situation, please consider filing an issue and if possible,
sharing sample files to test against so I can improve codepage handling.

