#![allow(unused_crate_dependencies)]

use concrete_type::Concrete;
use concrete_type_rules::gen_match_concretes_macro;

// Define our enums with Concrete derive for testing
#[derive(Concrete, Clone, Copy)]
enum Exchange {
    #[concrete = "test_types::Binance"]
    Binance,
    #[concrete = "test_types::Okx"]
    Okx,
}

#[derive(Concrete, Clone, Copy)]
enum Strategy {
    #[concrete = "test_types::StrategyA"]
    StrategyA,
    #[concrete = "test_types::StrategyB"]
    StrategyB,
}

#[derive(Concrete, Clone, Copy)]
enum TimeFrame {
    #[concrete = "test_types::Minute"]
    Minute,
    #[concrete = "test_types::Hour"]
    Hour,
}

// All our concrete types in a test-specific module
mod test_types {
    pub struct Binance;
    pub struct Okx;
    pub struct StrategyA;
    pub struct StrategyB;
    pub struct Minute;
    pub struct Hour;
}

trait System {
    const NAME: &'static str;
}

impl System for (test_types::Binance, test_types::StrategyA) {
    const NAME: &'static str = "binance_strategy_a";
}

impl System for (test_types::Binance, test_types::StrategyB) {
    const NAME: &'static str = "binance_strategy_b";
}

impl System for (test_types::Okx, test_types::StrategyA) {
    const NAME: &'static str = "okx_strategy_a";
}

impl System for (test_types::Okx, test_types::StrategyB) {
    const NAME: &'static str = "okx_strategy_b";
}

impl System
    for (
        test_types::Binance,
        test_types::StrategyA,
        test_types::Minute,
    )
{
    const NAME: &'static str = "binance_strategy_a_minute";
}

impl System
    for (
        test_types::Binance,
        test_types::StrategyB,
        test_types::Minute,
    )
{
    const NAME: &'static str = "binance_strategy_b_minute";
}

impl System for (test_types::Binance, test_types::StrategyA, test_types::Hour) {
    const NAME: &'static str = "binance_strategy_a_hour";
}

impl System for (test_types::Binance, test_types::StrategyB, test_types::Hour) {
    const NAME: &'static str = "binance_strategy_b_hour";
}

impl System for (test_types::Okx, test_types::StrategyA, test_types::Minute) {
    const NAME: &'static str = "okx_strategy_a_minute";
}

impl System for (test_types::Okx, test_types::StrategyB, test_types::Minute) {
    const NAME: &'static str = "okx_strategy_b_minute";
}

impl System for (test_types::Okx, test_types::StrategyA, test_types::Hour) {
    const NAME: &'static str = "okx_strategy_a_hour";
}

impl System for (test_types::Okx, test_types::StrategyB, test_types::Hour) {
    const NAME: &'static str = "okx_strategy_b_hour";
}

// Generate the macro combinations for testing
gen_match_concretes_macro!(Exchange, Strategy);
gen_match_concretes_macro!(Exchange, Strategy, TimeFrame);

#[test]
fn test_two_enum_match() {
    let exchange = Exchange::Binance;
    let strategy = Strategy::StrategyA;

    let result = match_exchange_strategy!(
        exchange, strategy; E, S => { <(E, S) as System>::NAME }
    );

    assert_eq!(result, "binance_strategy_a");

    let exchange = Exchange::Okx;
    let strategy = Strategy::StrategyB;

    let result = match_exchange_strategy!(
        exchange, strategy; E, S => { <(E, S) as System>::NAME }
    );

    assert_eq!(result, "okx_strategy_b");
}

#[test]
fn test_three_enum_match() {
    let exchange = Exchange::Binance;
    let strategy = Strategy::StrategyA;
    let timeframe = TimeFrame::Minute;

    let result = match_exchange_strategy_time_frame!(
        exchange, strategy, timeframe; E, S, T => { <(E, S, T) as System>::NAME }
    );

    assert_eq!(result, "binance_strategy_a_minute");

    let exchange = Exchange::Okx;
    let strategy = Strategy::StrategyB;
    let timeframe = TimeFrame::Hour;

    let result = match_exchange_strategy_time_frame!(
        exchange, strategy, timeframe; E, S, T => { <(E, S, T) as System>::NAME }
    );

    assert_eq!(result, "okx_strategy_b_hour");
}
