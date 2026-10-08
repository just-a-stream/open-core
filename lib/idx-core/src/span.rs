#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct IndexSpan<Index> {
    pub first: Index,
    pub len: u64,
}

impl<Index> IndexSpan<Index> {
    pub const fn new(first: Index, len: u64) -> Self {
        Self { first, len }
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn indexes(self) -> impl Iterator<Item = Index>
    where
        Index: Copy + Into<u64> + From<u64>,
    {
        let first: u64 = self.first.into();

        (0..self.len)
            .map_while(move |at| first.checked_add(at))
            .map(Index::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Index;

    fn span(first: u64, len: u64) -> IndexSpan<Index> {
        IndexSpan::new(Index::new(first), len)
    }

    fn indexes(span: IndexSpan<Index>) -> Vec<u64> {
        span.indexes().map(|index| index.value()).collect()
    }

    #[test]
    fn a_span_names_every_index_it_covers() {
        assert_eq!(indexes(span(4, 3)), vec![4, 5, 6]);
    }

    #[test]
    fn a_span_of_one_names_only_its_first() {
        assert_eq!(indexes(span(9, 1)), vec![9]);
        assert!(!span(9, 1).is_empty());
    }

    #[test]
    fn an_empty_span_names_nothing() {
        assert!(span(9, 0).is_empty());
        assert!(indexes(span(9, 0)).is_empty());
    }

    #[test]
    fn a_span_reaching_past_the_last_index_names_only_indexes_that_exist() {
        assert_eq!(indexes(span(u64::MAX - 1, 4)), vec![u64::MAX - 1, u64::MAX]);
    }
}
