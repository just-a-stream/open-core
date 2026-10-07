#![allow(unused_crate_dependencies)]

use concrete_type::Concrete;
use config::ExchangeConfig;
use std::mem::size_of;

mod core {
    pub mod primitive {
        #[allow(non_camel_case_types)]
        pub struct u8;
    }
}

mod exchanges {
    pub struct Binance;
    pub struct Okx;

    pub struct BinanceConfig {
        pub api_key: &'static str,
    }
}

trait Build: Sized {
    type Config;

    fn build(config: Self::Config) -> Result<&'static str, &'static str>;
}

impl Build for exchanges::Binance {
    type Config = exchanges::BinanceConfig;

    fn build(config: Self::Config) -> Result<&'static str, &'static str> {
        if config.api_key.is_empty() {
            return Err("empty api key");
        }

        Ok("binance")
    }
}

impl Build for exchanges::Okx {
    type Config = ();

    fn build((): ()) -> Result<&'static str, &'static str> {
        Ok("okx")
    }
}

#[derive(Clone, Copy, Concrete)]
enum Width {
    #[concrete(crate::exchanges::Binance)]
    Unit,
    #[concrete(::core::primitive::u8)]
    Byte,
    #[concrete(core::primitive::u8)]
    Shadowed,
}

mod config {
    use crate::exchanges;
    use concrete_type::Concrete;

    #[derive(Concrete)]
    #[concrete(bound(crate::Build + Send + 'static))]
    pub enum ExchangeConfig {
        #[concrete(crate::exchanges::Binance)]
        Binance(exchanges::BinanceConfig),
        #[concrete(crate::exchanges::Okx)]
        Okx,
    }
}

fn build(config: ExchangeConfig) -> Result<&'static str, &'static str> {
    let name = crate::config::exchange_config!(config; (Exchange, cfg) => Exchange::build(cfg)?);

    Ok(name)
}

#[test]
fn concrete_maps_each_variant_to_its_type_path() {
    let widths = [Width::Unit, Width::Byte, Width::Shadowed];

    let sizes = widths.map(|width| width!(width; T => { size_of::<T>() }));

    assert_eq!(sizes, [0, 1, 0]);
}

#[test]
fn concrete_binds_each_variant_to_its_type_and_config_through_the_enum_module_path() {
    let binance = ExchangeConfig::Binance(exchanges::BinanceConfig { api_key: "key" });
    let unkeyed = ExchangeConfig::Binance(exchanges::BinanceConfig { api_key: "" });

    let names = [binance, unkeyed, ExchangeConfig::Okx].map(build);

    assert_eq!(names, [Ok("binance"), Err("empty api key"), Ok("okx")]);
}
