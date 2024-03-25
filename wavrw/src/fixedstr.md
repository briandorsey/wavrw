Null terminated fixed length strings.

The Broadcast Wave Format's Broadcast Audio Extension Chunk specifies a string
type that is fixed length and zero byte (`0x0`) padded. [`FixedStr`] implements this type, 
including handling the corner cases where the string fully uses the space, thus having
no padding.


```
use wavrw::fixedstr::FixedStr; 
use std::io::Cursor; 
use binrw::BinWrite;
use core::str::FromStr; 

let fs = FixedStr::<6>::from_str("abc").unwrap();
let s = fs.to_string();
assert_eq!(s, "abc".to_string());
assert_eq!(s.len(), 3);

// or create via parse() or from_str() (using FromStr trait)
let new_fs = "abc".parse::<FixedStr<6>>(); 
assert_eq!(new_fs, Ok(FixedStr::<6>::from_str("abc").unwrap()));

// normally, `wavrw` handles serialization, but to prove 
// that we're storing 0x0 byte padding, write with BinWrite
let mut buff = Cursor::new(Vec::new());
fs.write_le(&mut buff); 
let v = buff.into_inner();
assert_eq!(v.len(), 6); 
assert_eq!(v, vec![97, 98, 99, 0, 0, 0]); 
```


Converting from strings longer than the fixed size (N) will return a 
[`FixedStrError::Truncated`] error:
```
use wavrw::fixedstr::{FixedStr, FixedStrError}; 
use core::str::FromStr; 

// use FromStr trait if you want to catch this as an error
let long_str = "abcdefghijklmnopqrstuvwxyz";
let err = FixedStr::<6>::from_str(long_str);
assert_eq!(err, Err(FixedStrError::Truncated { limit: 6, len: 26 }));

// or create via parse() 
let err = "abcdefghijklmnopqrstuvwxyz".parse::<FixedStr<6>>();
assert_eq!(err, Err(FixedStrError::Truncated { limit: 6, len: 26 }));

```

See [`FixedStr::from_utf8()`] to convert from bytes. 


