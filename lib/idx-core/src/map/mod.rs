use crate::span::IndexSpan;
use std::ops::Range;

#[cfg(feature = "serde")]
mod serde;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
pub struct IndexMap<Index, Item> {
    index_first: Index,
    items: Vec<Item>,
}

impl<Index, Item> Default for IndexMap<Index, Item>
where
    Index: Default,
{
    fn default() -> Self {
        Self {
            index_first: Index::default(),
            items: Vec::new(),
        }
    }
}

impl<Index, Item> IndexMap<Index, Item> {
    pub const fn len(&self) -> usize {
        self.items.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn index_next(&self) -> Option<Index>
    where
        Index: Copy + Into<u64> + From<u64>,
    {
        let first: u64 = self.index_first.into();

        first.checked_add(self.items.len() as u64).map(Index::from)
    }

    pub fn find(&self, index: Index) -> Option<&Item>
    where
        Index: Copy + Into<u64>,
    {
        self.items.get(self.position(index.into())?)
    }

    pub fn find_mut(&mut self, index: Index) -> Option<&mut Item>
    where
        Index: Copy + Into<u64>,
    {
        let position = self.position(index.into())?;

        self.items.get_mut(position)
    }

    pub fn find_mut_extending(&mut self, index: Index) -> Option<&mut Item>
    where
        Index: Copy + Into<u64>,
        Item: Default,
    {
        let first: u64 = self.index_first.into();
        let wanted: u64 = index.into();
        let len_reaching = usize::try_from(wanted.checked_sub(first)?.checked_add(1)?).ok()?;
        if len_reaching > self.items.len() {
            self.items
                .try_reserve(len_reaching - self.items.len())
                .ok()?;
            self.items.resize_with(len_reaching, Item::default);
        }

        self.find_mut(index)
    }

    pub fn iter(&self) -> impl Iterator<Item = (Index, &Item)>
    where
        Index: Copy + Into<u64> + From<u64>,
    {
        self.numbered(0..self.items.len())
    }

    pub fn contains_span(&self, span: IndexSpan<Index>) -> bool
    where
        Index: Copy + Into<u64>,
    {
        let first: u64 = self.index_first.into();
        let Some(offset) = span.first.into().checked_sub(first) else {
            return false;
        };

        offset
            .checked_add(span.len)
            .is_some_and(|offset_end| offset_end <= self.items.len() as u64)
    }

    pub fn items_in_span(&self, span: IndexSpan<Index>) -> impl Iterator<Item = (Index, &Item)>
    where
        Index: Copy + Into<u64> + From<u64>,
    {
        let first: u64 = self.index_first.into();
        let span_first: u64 = span.first.into();
        let skipped = first.saturating_sub(span_first);
        let offset_start = span_first.saturating_sub(first);
        let offset_end = offset_start.saturating_add(span.len.saturating_sub(skipped));

        self.numbered(self.position_clamped(offset_start)..self.position_clamped(offset_end))
    }

    pub fn push(&mut self, item: Item) -> Option<Index>
    where
        Index: Copy + Into<u64> + From<u64>,
    {
        let index = self.index_next()?;
        self.items.push(item);

        Some(index)
    }

    pub fn extend<Items>(&mut self, items: Items) -> Option<IndexSpan<Index>>
    where
        Index: Copy + Into<u64> + From<u64>,
        Items: IntoIterator<Item = Item>,
    {
        let first = self.index_next()?;
        let len_before = self.items.len();
        self.items.extend(items);

        if !indexes_reach(self.index_first.into(), self.items.len()) {
            self.items.truncate(len_before);
            return None;
        }

        Some(IndexSpan::new(
            first,
            (self.items.len() - len_before) as u64,
        ))
    }

    fn numbered(&self, positions: Range<usize>) -> impl Iterator<Item = (Index, &Item)>
    where
        Index: Copy + Into<u64> + From<u64>,
    {
        let first: u64 = self.index_first.into();
        let items = self.items.get(positions.clone()).unwrap_or_default();

        positions
            .zip(items)
            .map(move |(position, item)| (Index::from(first + position as u64), item))
    }

    fn position_clamped(&self, offset: u64) -> usize {
        usize::try_from(offset).map_or(self.items.len(), |position| position.min(self.items.len()))
    }

    fn position(&self, index: u64) -> Option<usize>
    where
        Index: Copy + Into<u64>,
    {
        let first: u64 = self.index_first.into();

        usize::try_from(index.checked_sub(first)?).ok()
    }
}

fn indexes_reach(index_first: u64, len: usize) -> bool {
    len.checked_sub(1).is_none_or(|offset_last| {
        u64::try_from(offset_last)
            .ok()
            .and_then(|offset_last| index_first.checked_add(offset_last))
            .is_some()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Index;

    type Shelf = IndexMap<Index, &'static str>;

    fn index(value: u64) -> Index {
        Index::new(value)
    }

    fn shelved(items: &[&'static str]) -> Shelf {
        let mut shelf = Shelf::default();
        shelf.extend(items.iter().copied());

        shelf
    }

    #[test]
    fn an_index_finds_the_item_it_was_given() {
        let shelf = shelved(&["a", "b", "c"]);

        assert_eq!(shelf.find(index(0)), Some(&"a"));
        assert_eq!(shelf.find(index(2)), Some(&"c"));
        assert_eq!(shelf.find(index(3)), None);
        assert_eq!(shelf.len(), 3);
    }

    #[test]
    fn an_extend_is_named_by_the_span_it_wrote() {
        let mut shelf = shelved(&["a"]);

        let span = shelf.extend(["b", "c"]).unwrap();

        assert_eq!(span, IndexSpan::new(index(1), 2));
        assert_eq!(
            shelf
                .items_in_span(span)
                .map(|(_, item)| *item)
                .collect::<Vec<_>>(),
            vec!["b", "c"],
            "one push is one span, and a span is contiguous"
        );
    }

    #[test]
    fn a_push_of_nothing_writes_nothing_and_spans_nothing() {
        let mut shelf = shelved(&["a"]);

        let span = shelf.extend(std::iter::empty()).unwrap();

        assert!(span.is_empty());
        assert_eq!(Some(span.first), shelf.index_next());
        assert_eq!(shelf.len(), 1);
    }

    #[test]
    fn pushes_number_from_where_the_map_starts() {
        let mut shelf = Shelf::default();

        assert_eq!(shelf.push("a"), Some(index(0)));
        assert_eq!(shelf.push("b"), Some(index(1)));
        assert_eq!(
            shelf
                .iter()
                .map(|(position, item)| (position.value(), *item))
                .collect::<Vec<_>>(),
            vec![(0, "a"), (1, "b")]
        );
    }

    #[test]
    fn a_span_is_contained_only_where_every_index_is_live() {
        let shelf = shelved(&["a", "b", "c"]);

        assert!(shelf.contains_span(IndexSpan::new(index(0), 3)));
        assert!(shelf.contains_span(IndexSpan::new(index(2), 1)));
        assert!(shelf.contains_span(IndexSpan::new(index(3), 0)));
        assert!(!shelf.contains_span(IndexSpan::new(index(2), 2)));
    }

    #[test]
    fn a_map_whose_indexes_run_out_refuses_items_rather_than_wrapping() {
        let mut shelf = Shelf {
            index_first: index(u64::MAX),
            items: Vec::new(),
        };

        assert_eq!(shelf.extend(["a", "b"]), None);
        assert_eq!(shelf.push("a"), Some(index(u64::MAX)));
        assert_eq!(shelf.push("b"), None);
        assert_eq!(shelf.len(), 1);
    }

    #[test]
    fn a_found_item_can_be_revised_in_place() {
        let mut shelf = shelved(&["a", "b"]);

        *shelf.find_mut(index(1)).unwrap() = "revised";

        assert_eq!(shelf.find(index(1)), Some(&"revised"));
        assert_eq!(shelf.find_mut(index(2)), None);
    }

    #[test]
    fn a_slot_past_the_end_is_reached_by_extending_with_defaults() {
        let mut shelf = shelved(&["a"]);

        *shelf.find_mut_extending(index(3)).unwrap() = "d";

        assert_eq!(
            shelf.iter().map(|(_, item)| *item).collect::<Vec<_>>(),
            vec!["a", "", "", "d"],
            "the slots between are filled, never skipped: no holes in an index map"
        );
        assert_eq!(shelf.index_next(), Some(index(4)));
    }

    #[test]
    fn a_slot_already_live_is_found_without_extending() {
        let mut shelf = shelved(&["a", "b"]);

        *shelf.find_mut_extending(index(0)).unwrap() = "revised";

        assert_eq!(shelf.find(index(0)), Some(&"revised"));
        assert_eq!(shelf.len(), 2);
    }
}
