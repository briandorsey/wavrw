Null terminated fixed length strings.

The Broadcast Wave Format's Broadcast Audio Extension Chunk specifies a string
type that is fixed length and zero byte (`0x0`) padded. [`FixedStr`] implments this type, 
including handling the corner cases where the string fully uses the space, thus having
no padding.


```
use wavrw::fixedstr::FixedStr; 
use std::io::Cursor; 
use binrw::BinWrite;

// create via ::new() constructor
let fs = FixedStr::<6>::new("abc");
let s = fs.to_string();
assert_eq!("abc".to_string(), s);
assert_eq!(3, s.len());

// or create via parse() (using FromStr trait)
if let Ok(new_fs) = "abc".parse::<FixedStr<6>>(){
    assert_eq!(fs, new_fs);
}

// normally, `wavrw` handles serialization, but to prove 
// that we're storing 0x0 byte padding, write with BinWrite
let mut buff = Cursor::new(Vec::new());
fs.write_le(&mut buff); 
let v = buff.into_inner();
assert_eq!(v.len(), 6); 
assert_eq!(v, vec![97, 98, 99, 0, 0, 0]); 
```


