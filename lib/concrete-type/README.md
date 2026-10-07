# concrete-type

[![Crates.io](https://img.shields.io/crates/v/concrete-type.svg)](https://crates.io/crates/concrete-type)
[![Documentation](https://docs.rs/concrete-type/badge.svg)](https://docs.rs/concrete-type)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/just-a-stream/open-core#licence)

Maps enum variants to concrete types, so a runtime value selects a type once and everything
after it is generic and statically dispatched.

```toml
[dependencies]
concrete-type = "0.3"
```

## `#[derive(Concrete)]`

Each variant names its concrete type with `#[concrete = "path::to::Type"]` and carries either
no data or exactly one unnamed field holding its configuration. The derive emits a matcher
macro named after the enum in snake case (`exchange!` for `Exchange`) with two forms:

- `exchange!(value; T => body)` evaluates the body with `T` aliased to the matched variant's
  concrete type;
- `exchange!(value; (T, config) => body)` also binds the variant's configuration, `()` for a
  unit variant.

The body is an ordinary expression in the caller's function, so it can capture locals and use
`?`, `return` and `.await`.

```rust
use concrete_type::Concrete;

mod exchanges {
    pub trait ExchangeApi {
        type Config;

        fn new(config: Self::Config) -> Self;
        fn name(&self) -> &'static str;
    }

    pub struct Binance;
    pub struct BinanceConfig {
        pub api_key: String,
    }

    impl ExchangeApi for Binance {
        type Config = BinanceConfig;

        fn new(_: Self::Config) -> Self {
            Self
        }

        fn name(&self) -> &'static str {
            "binance"
        }
    }

    pub struct Okx;

    impl ExchangeApi for Okx {
        type Config = ();

        fn new((): ()) -> Self {
            Self
        }

        fn name(&self) -> &'static str {
            "okx"
        }
    }
}

#[derive(Concrete)]
enum Exchange {
    #[concrete = "crate::exchanges::Binance"]
    Binance(exchanges::BinanceConfig),
    #[concrete = "crate::exchanges::Okx"]
    Okx,
}

fn main() {
    use exchanges::ExchangeApi;

    let config = Exchange::Binance(exchanges::BinanceConfig {
        api_key: "secret".to_string(),
    });

    let type_name = exchange!(&config; ExchangeImpl => std::any::type_name::<ExchangeImpl>());
    let name = exchange!(config; (ExchangeImpl, cfg) => ExchangeImpl::new(cfg).name());

    assert!(type_name.ends_with("Binance"));
    assert_eq!(name, "binance");
}
```

## `concrete_map!`

An enum defined in another crate cannot carry the derive. `concrete_map!` emits the same
matcher for it beside the invocation: `Data(_)` marks a variant carrying one field of
configuration, and a final `_ => Type` maps every variant not listed, which also covers a
`#[non_exhaustive]` enum. A leading `pub` exports the macro to other crates, and
`#[concrete(bound(..))]` works as on the derive:

```rust
mod widths {
    pub struct Narrow;
    pub struct Wide;
}

concrete_type::concrete_map! {
    core::cmp::Ordering => {
        Less => crate::widths::Narrow,
        _ => crate::widths::Wide,
    }
}

fn main() {
    let name = ordering!(core::cmp::Ordering::Equal; T => std::any::type_name::<T>());

    assert!(name.ends_with("Wide"));
}
```

## Bounds

`#[concrete(bound(..))]` on the enum takes a where-clause bound list that every concrete type
must satisfy. A type that misses one fails to compile with E0277 at its own variant, not inside
the first body that needs the trait:

```rust,compile_fail,E0277
use concrete_type::Concrete;

trait ExchangeApi {}

struct Binance;
struct Okx;

impl ExchangeApi for Binance {}

#[derive(Concrete)]
#[concrete(bound(ExchangeApi + Send + 'static))]
enum Exchange {
    #[concrete = "Binance"]
    Binance,
    #[concrete = "Okx"]
    Okx,
}

fn main() {}
```

## Path resolution

The matcher macro lives beside its enum: an enum at `crate::config::Exchange` gets
`crate::config::exchange!`, callable by path or imported with `use`, with no dependence on
textual order. A `pub` enum's macro is also exported, so another crate calls
`dependency::config::exchange!`. The macro names the enum unqualified, so the enum must be in
scope where the macro is called.

A `#[concrete]` path starting with `crate::` is rewritten to `$crate::`, so the generated
macro resolves it from the defining crate and from any other. Any other path resolves where
the macro is called: use it for types from external crates.

## Licence

Licensed under MIT OR Apache-2.0.
