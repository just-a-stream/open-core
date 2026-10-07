#![allow(unused_crate_dependencies)]

use concrete_type::{Concrete, concrete};
use exchanges::Okx;

mod exchanges {
    pub struct Binance;
    pub struct Okx;
}

mod strategies {
    pub struct Momentum;
    pub struct MeanReversion;
}

mod servers {
    pub struct Http;
    pub struct Http2;
}

#[derive(Clone, Copy, Concrete)]
enum Exchange {
    #[concrete(crate::exchanges::Binance)]
    Binance,
    #[concrete(crate::exchanges::Okx)]
    Okx,
}

#[derive(Clone, Copy, Concrete)]
enum Venue {
    #[concrete(Okx)]
    Okx,
}

#[derive(Concrete)]
enum ExchangeConfig {
    #[concrete(crate::exchanges::Binance)]
    Binance(u8),
    #[concrete(crate::exchanges::Okx)]
    Okx,
}

#[derive(Clone, Copy, Concrete)]
enum Strategy {
    #[concrete(crate::strategies::Momentum)]
    Momentum,
    #[concrete(crate::strategies::MeanReversion)]
    MeanReversion,
}

#[derive(Clone, Copy, Concrete)]
#[allow(clippy::upper_case_acronyms)]
enum HTTPServer {
    #[concrete(crate::servers::Http)]
    Http,
    #[concrete(crate::servers::Http2)]
    Http2,
}

trait Name {
    const NAME: &'static str;
}

impl Name for exchanges::Binance {
    const NAME: &'static str = "binance";
}

impl Name for exchanges::Okx {
    const NAME: &'static str = "okx";
}

impl Name for strategies::Momentum {
    const NAME: &'static str = "momentum";
}

impl Name for strategies::MeanReversion {
    const NAME: &'static str = "mean_reversion";
}

impl Name for servers::Http {
    const NAME: &'static str = "http";
}

impl Name for servers::Http2 {
    const NAME: &'static str = "http2";
}

#[test]
fn concrete_binds_every_combination_of_several_enums() {
    let exchanges = [Exchange::Binance, Exchange::Okx];
    let strategies = [Strategy::Momentum, Strategy::MeanReversion];
    let servers = [HTTPServer::Http, HTTPServer::Http2];
    let configs = [ExchangeConfig::Binance(7), ExchangeConfig::Okx];

    let triples = exchanges.map(|exchange| {
        strategies.map(|strategy| {
            servers.map(|server| {
                concrete!(match (exchange, strategy, server) {
                    (E: Exchange, S: Strategy, H: HTTPServer) => [E::NAME, S::NAME, H::NAME],
                })
            })
        })
    });
    let named_after_enums = concrete!(match (Exchange::Okx, Strategy::Momentum) {
        (Exchange, Strategy) => (Exchange::NAME, Strategy::NAME),
    });
    let with_config = configs.map(|config| {
        concrete!(match (config, Strategy::MeanReversion) {
            (E(config): ExchangeConfig, S: Strategy) => format!("{} {} {config:?}", E::NAME, S::NAME),
        })
    });
    let unshadowed = concrete!(match (Strategy::Momentum, Venue::Okx) {
        (Okx: Strategy, V: Venue) => (Okx::NAME, V::NAME),
    });

    assert_eq!(
        triples[0][0],
        [
            ["binance", "momentum", "http"],
            ["binance", "momentum", "http2"]
        ]
    );
    assert_eq!(triples[1][1][1], ["okx", "mean_reversion", "http2"]);
    assert_eq!(named_after_enums, ("okx", "momentum"));
    assert_eq!(
        with_config,
        ["binance mean_reversion 7", "okx mean_reversion ()"]
    );
    assert_eq!(unshadowed, ("momentum", "okx"));
}
