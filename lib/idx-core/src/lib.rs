#![doc = include_str!("../README.md")]

use thiserror::Error;

mod define_index;
#[cfg(feature = "map")]
pub mod map;
#[cfg(feature = "serde")]
#[doc(hidden)]
pub mod serde;
pub mod span;

define_index!(Index);

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Error)]
#[error("an index is the decimal it displays as, and {0:?} is not one")]
pub struct IndexUnparseable(String);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
pub struct Indexed<T> {
    pub index: Index,
    pub data: T,
}

impl<T> Indexed<T> {
    pub const fn new(index: Index, data: T) -> Self {
        Self { index, data }
    }
}

pub fn index_value_parsed(text: &str) -> Result<u64, IndexUnparseable> {
    match text.as_bytes() {
        [b'0'] | [b'1'..=b'9', ..] => text.parse().map_err(|_| IndexUnparseable(text.to_owned())),
        _ => Err(IndexUnparseable(text.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advancing_returns_the_index_consumed_and_moves_on() {
        let mut index = Index::ZERO;

        assert_eq!(index.advance(), Some(Index::ZERO));
        assert_eq!(index.advance(), Some(Index::new(1)));
        assert_eq!(index, Index::new(2));
    }

    #[test]
    fn an_exhausted_index_hands_out_nothing_and_stays_put() {
        let mut index = Index::new(u64::MAX);

        assert_eq!(index.advance(), None);
        assert_eq!(index, Index::new(u64::MAX));
    }

    #[test]
    fn an_index_parses_exactly_the_decimal_it_displays_and_nothing_else() {
        for value in [0, 1, 10, u64::MAX] {
            let index = Index::new(value);

            assert_eq!(index.to_string().parse::<Index>(), Ok(index));
        }
        for text in [
            "",
            " 1",
            "+1",
            "-0",
            "01",
            "1_0",
            "0x1",
            "18446744073709551616",
        ] {
            assert!(text.parse::<Index>().is_err(), "{text:?} parsed");
        }
    }
}
