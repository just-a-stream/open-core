# codec-core

Two small traits for turning values into bytes and back. An `Encoder` writes a
value into a buffer you pass in, or hands back owned `Bytes`; a `Decoder` reads
a value out of a byte slice, and may borrow from it.

```rust
use codec_core::Encoder;
use std::convert::Infallible;

struct Plain;

impl Encoder<str> for Plain {
    type Error = Infallible;

    fn encode_into(&self, input: &str, buffer: &mut Vec<u8>) -> Result<(), Infallible> {
        buffer.extend_from_slice(input.as_bytes());
        Ok(())
    }
}

assert_eq!(Plain.encode("hi").unwrap(), "hi");
```

For JSON, see [`codec-json`](https://docs.rs/codec-json).

Licensed under MIT OR Apache-2.0.
