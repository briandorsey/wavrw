Read (and someday write) wave audio file chunks with a focus on metadata.

This is the API reference documentation, it is a bit dry.

Iterate over all dyn [`SizedChunk`] chunk objects from a file:

```
# use std::fs::File;
let file = File::open("../test_wavs/example_a.wav")?;
for result in wavrw::metadata_chunks(file)? {
    match result {
        Ok(chunk) => {
            println!(
                "{:12} {:10} {}",
                chunk.name(),
                chunk.size(),
                chunk.summary()
            );
        }
        Err(err) => {
            println!("ERROR: {err}");
        }
    }
}
# Ok::<(), std::io::Error>(())
```

Or parse a single chunk from a buffer:

```
# use binrw::BinRead;
# use wavrw::testing::hex_to_cursor;
# let mut buff = hex_to_cursor("666D7420 10000000 01000100 80BB0000 80320200 03001800");
use wavrw::{ChunkEnum, ChunkID, Summarizable};
use wavrw::FourCC;

let chunk = ChunkEnum::read(&mut buff).unwrap();

// Use methods from SizedChunk trait on any chunk
assert_eq!(chunk.id(), FourCC(*b"fmt "));
assert_eq!(chunk.summary(), "PCM (0x1), 1 chan, 24/48000".to_string());

// Or match on type and handle various chunks individually
match chunk {
    ChunkEnum::Fmt(fmt) => println!("sample rate: {}", fmt.data.samples_per_sec),
    _ => ()
}
```
