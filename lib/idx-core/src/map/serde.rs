use super::{IndexMap, indexes_reach};
use serde::{Deserialize, Deserializer, de::Error};

impl<'de, Index, Item> Deserialize<'de> for IndexMap<Index, Item>
where
    Index: Deserialize<'de> + Copy + Into<u64>,
    Item: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename = "IndexMap")]
        struct Unchecked<Index, Item> {
            index_first: Index,
            items: Vec<Item>,
        }

        let Unchecked { index_first, items } = Unchecked::<Index, Item>::deserialize(deserializer)?;
        if !indexes_reach(index_first.into(), items.len()) {
            return Err(D::Error::custom(
                "an index map's last item would need an index past u64::MAX",
            ));
        }

        Ok(Self { index_first, items })
    }
}
