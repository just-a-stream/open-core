# codec-json

`Json` encodes any `Serialize` value to JSON bytes and decodes any
`Deserialize` value back, through the `codec-core` traits. A decoded value can
borrow from the bytes, such as a `&str`.

```rust
use codec_core::{Decoder, Encoder};
use codec_json::Json;

let bytes = Json.encode(&vec![1, 2, 3]).unwrap();
let numbers: Vec<u64> = Json.decode(&bytes).unwrap();

assert_eq!(bytes, "[1,2,3]");
assert_eq!(numbers, [1, 2, 3]);
```

`encode_into` writes into a buffer you already have, so a buffer with enough
room needs no allocation for the output; serialising the value may still
allocate.

Licensed under MIT OR Apache-2.0.
