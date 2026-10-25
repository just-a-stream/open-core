use derive_more::{Constructor, Display, Error};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Constructor, Display)]
pub struct Sequence(u64);

impl Sequence {
    pub const fn value(self) -> u64 {
        self.0
    }

    const fn next(self) -> Option<Self> {
        match self.0.checked_add(1) {
            Some(next) => Some(Self(next)),
            None => None,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Sequencer {
    sequence_next: Sequence,
}

impl Sequencer {
    pub const fn new(sequence_next: Sequence) -> Self {
        Self { sequence_next }
    }

    pub const fn sequence_next(&self) -> Sequence {
        self.sequence_next
    }

    pub fn sequence<Item>(&mut self, item: Item) -> Result<Sequenced<Item>, SequenceError> {
        let sequence = self.sequence_next;

        let Some(next) = sequence.next() else {
            return Err(SequenceError::Exhausted);
        };
        self.sequence_next = next;

        Ok(Sequenced::new(sequence, item))
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SequenceValidator {
    sequence_expected: Sequence,
}

impl SequenceValidator {
    pub const fn new(sequence_expected: Sequence) -> Self {
        Self { sequence_expected }
    }

    pub const fn sequence_expected(&self) -> Sequence {
        self.sequence_expected
    }

    pub fn validate_and_increment(
        &mut self,
        received: Sequence,
    ) -> Result<Sequence, SequenceError> {
        if received != self.sequence_expected {
            return Err(SequenceError::Unexpected(SequenceUnexpected {
                expected: self.sequence_expected,
                received,
            }));
        }

        let Some(next) = received.next() else {
            return Err(SequenceError::Exhausted);
        };
        self.sequence_expected = next;

        Ok(received)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Display, Error)]
pub enum SequenceError {
    #[display("sequence unexpected")]
    Unexpected(#[error(source)] SequenceUnexpected),
    #[display("sequence exhausted")]
    Exhausted,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Display, Error)]
#[display("expected sequence {expected} but received {received}")]
pub struct SequenceUnexpected {
    pub expected: Sequence,
    pub received: Sequence,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Sequenced<Item> {
    sequence: Sequence,
    item: Item,
}

impl<Item> Sequenced<Item> {
    pub(crate) const fn new(sequence: Sequence, item: Item) -> Self {
        Self { sequence, item }
    }

    pub const fn sequence(&self) -> Sequence {
        self.sequence
    }

    pub const fn item(&self) -> &Item {
        &self.item
    }

    pub fn into_item(self) -> Item {
        self.item
    }

    pub const fn as_ref(&self) -> Sequenced<&Item> {
        Sequenced {
            sequence: self.sequence,
            item: &self.item,
        }
    }

    pub fn map<Mapped, FnMap>(self, map: FnMap) -> Sequenced<Mapped>
    where
        FnMap: FnOnce(Item) -> Mapped,
    {
        Sequenced {
            sequence: self.sequence,
            item: map(self.item),
        }
    }
}
