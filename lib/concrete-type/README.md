# concrete-type

Turn a config enum into a concrete type once, then run generic, statically dispatched code.

```rust
use concrete_type::Concrete;

struct Binance;
struct Okx;

#[derive(Concrete)]
enum Exchange { #[concrete(Binance)] Binance, #[concrete(Okx)] Okx(&'static str) }

fn main() {
    let name = exchange!(Exchange::Binance; E => std::any::type_name::<E>());
    let key = exchange!(Exchange::Okx("key"); (E, key) => format!("{key:?}"));
    assert!(name.ends_with("Binance") && key.contains("key"));
}
```

The body is ordinary code (`?`, `return` and `.await` work); `(E, key)` binds a variant's field,
`()` for a unit variant. `concrete!` matches several enums at once, and `concrete_map!` maps an
enum from another crate, `_` taking the variants it leaves out:

```rust
use concrete_type::{Concrete, concrete, concrete_map};

mod instrument { #[derive(Clone, Copy)] pub enum ExchangeId { BinanceSpot, Kraken, Other } }
use instrument::ExchangeId;

trait Name { const NAME: &'static str; }
struct BinanceSpot; struct Kraken; struct NoConnector; struct Trades; struct Book;
impl Name for BinanceSpot { const NAME: &'static str = "binance"; }
impl Name for Kraken { const NAME: &'static str = "kraken"; }
impl Name for NoConnector { const NAME: &'static str = "none"; }
impl Name for Trades { const NAME: &'static str = "trades"; }
impl Name for Book { const NAME: &'static str = "book"; }

concrete_map! { instrument::ExchangeId => { BinanceSpot => BinanceSpot, Kraken => Kraken, _ => NoConnector } }

#[derive(Clone, Copy, Concrete)]
enum SubKind { #[concrete(Trades)] Trades, #[concrete(Book)] Book }

fn stream(exchange: ExchangeId, kind: SubKind) -> String {
    concrete!(match (exchange, kind) {
        (E: ExchangeId, SubKind) => format!("{} {}", E::NAME, SubKind::NAME),
    })
}

fn main() {
    assert_eq!(stream(ExchangeId::Kraken, SubKind::Book), "kraken book");
    assert_eq!(stream(ExchangeId::Other, SubKind::Trades), "none trades");
}
```

`E(cfg): Enum` also binds a field; a bare `SubKind` binds the type under the enum's own name.
`#[concrete(bound(Venue + Send))]` checks every variant's type where it is declared.
A matcher runs where it is called, so import the enum there; if that is outside the enum's
module, name this crate's types from `crate::`, as in `#[concrete(crate::venues::Binance)]`.

Licensed under MIT OR Apache-2.0.
