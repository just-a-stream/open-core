#![allow(unused_crate_dependencies)]

use crate::{
    exchanges::{Binance, Okx},
    strategies::{StrategyA, StrategyB},
};
use concrete_type::Concrete;
use std::marker::PhantomData;

#[derive(Concrete, Clone, Copy)]
enum Exchange {
    #[concrete = "crate::exchanges::Binance"]
    Binance,
    #[concrete = "crate::exchanges::Okx"]
    Okx,
    #[concrete = "crate::exchanges::Kraken<crate::exchanges::KrakenSpotServer>"]
    Kraken,
}

mod exchanges {
    #[derive(Debug)]
    pub struct Binance;

    #[derive(Debug)]
    pub struct Okx;

    #[derive(Debug)]
    pub struct KrakenSpotServer;
    #[derive(Debug)]
    pub struct Kraken<Server> {
        pub _phantom: std::marker::PhantomData<Server>,
    }
}

#[derive(Concrete)]
enum Strategy {
    #[concrete = "crate::strategies::StrategyA"]
    StrategyA,

    #[concrete = "crate::strategies::StrategyB"]
    StrategyB,
}

pub mod strategies {
    #[derive(Debug)]
    pub struct StrategyA;

    #[derive(Debug)]
    pub struct StrategyB;
}

#[derive(Debug)]
pub struct TradingSystem<Exchange, Strategy> {
    phantom: PhantomData<(Exchange, Strategy)>,
}

impl<Exchange, Strategy> Default for TradingSystem<Exchange, Strategy> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Exchange, Strategy> TradingSystem<Exchange, Strategy> {
    pub const fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

fn main() {
    let exchange = Exchange::Binance;
    let strategy = Strategy::StrategyA;

    let name = exchange!(exchange; Exchange => {
        strategy!(strategy; Strategy => {
            TradingSystem::<Exchange, Strategy>::new().name()
        })
    });
    assert_eq!(name, "binance_strategy_a");

    let exchange = Exchange::Okx;
    let strategy = Strategy::StrategyB;

    let name = exchange!(exchange; Exchange => {
        strategy!(strategy; Strategy => {
            TradingSystem::<Exchange, Strategy>::new().name()
        })
    });
    assert_eq!(name, "okx_strategy_b");

    let exchange = Exchange::Kraken;
    let strategy = Strategy::StrategyA;

    let name = exchange!(exchange; Exchange => {
        strategy!(strategy; Strategy => {
            TradingSystem::<Exchange, Strategy>::new().name()
        })
    });
    assert_eq!(name, "kraken_strategy_a");
}

impl TradingSystem<Binance, StrategyA> {
    pub const fn name(&self) -> &'static str {
        "binance_strategy_a"
    }
}

impl TradingSystem<Binance, StrategyB> {
    pub const fn name(&self) -> &'static str {
        "binance_strategy_b"
    }
}

impl TradingSystem<Okx, StrategyA> {
    pub const fn name(&self) -> &'static str {
        "okx_strategy_a"
    }
}

impl TradingSystem<Okx, StrategyB> {
    pub const fn name(&self) -> &'static str {
        "okx_strategy_b"
    }
}

use crate::exchanges::{Kraken, KrakenSpotServer};

impl TradingSystem<Kraken<KrakenSpotServer>, StrategyA> {
    pub const fn name(&self) -> &'static str {
        "kraken_strategy_a"
    }
}

impl TradingSystem<Kraken<KrakenSpotServer>, StrategyB> {
    pub const fn name(&self) -> &'static str {
        "kraken_strategy_b"
    }
}
