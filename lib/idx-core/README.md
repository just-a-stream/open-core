# idx-core

Number things in order. `Index` hands out 0, 1, 2 and so on, `Indexed` pairs a
value with its index, and `IndexSpan` names a run of indexes. `define_index!`
makes your own index types that work the same way.

```rust
use idx_core::{Index, Indexed};

let mut next = Index::ZERO;
let first = next.advance().map(|index| Indexed::new(index, "created"));

assert_eq!(first, Some(Indexed::new(Index::ZERO, "created")));
assert_eq!("1".parse::<Index>(), Ok(next));
assert_eq!(Index::new(u64::MAX).next(), None);
```

An index prints and parses as a plain number; `01` or `+1` does not parse.
Once the numbers run out, `next` and `advance` return `None`.

The `map` feature adds `map::IndexMap`, a list you look items up in by index.
The `serde` feature serialises every index, your own included, as a plain
number, without your crate depending on serde.

Licensed under MIT OR Apache-2.0.
