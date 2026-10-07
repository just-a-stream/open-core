#![allow(unused_crate_dependencies)]

use concrete_type::{Concrete, concrete};
use std::marker::PhantomData;

mod exchanges {
    pub struct Binance;
    pub struct Okx;
}

mod strategies {
    pub struct Momentum;
    pub struct MeanReversion;
}

#[derive(Clone, Copy, Concrete)]
enum Exchange {
    #[concrete(crate::exchanges::Binance)]
    Binance,
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

struct TradingSystem<Exchange, Strategy> {
    capital: u32,
    phantom: PhantomData<(Exchange, Strategy)>,
}

impl<Exchange, Strategy> TradingSystem<Exchange, Strategy> {
    const fn new(capital: u32) -> Self {
        Self {
            capital,
            phantom: PhantomData,
        }
    }

    fn run(&self) -> String {
        format!(
            "trading {} with {} on {}",
            std::any::type_name::<Strategy>(),
            self.capital,
            std::any::type_name::<Exchange>()
        )
    }
}

fn main() {
    for exchange in [Exchange::Binance, Exchange::Okx] {
        for strategy in [Strategy::Momentum, Strategy::MeanReversion] {
            let report = concrete!(match (exchange, strategy) {
                (E: Exchange, S: Strategy) => TradingSystem::<E, S>::new(1_000).run(),
            });

            println!("{report}");
        }
    }
}
