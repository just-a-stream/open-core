#![doc = include_str!("../README.md")]

use bytes::Bytes;
use codec_core::{Decoder, Encoder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Json;

impl<Input> Encoder<Input> for Json
where
    Input: Serialize + ?Sized,
{
    type Error = serde_json::Error;

    fn encode_into(&self, input: &Input, buffer: &mut Vec<u8>) -> Result<(), serde_json::Error> {
        serde_json::to_writer(buffer, input)
    }

    fn encode(&self, input: &Input) -> Result<Bytes, serde_json::Error> {
        serde_json::to_vec(input).map(Bytes::from)
    }
}

impl<'de, Output> Decoder<'de, Output> for Json
where
    Output: Deserialize<'de>,
{
    type Error = serde_json::Error;

    fn decode(&self, input: &'de [u8]) -> Result<Output, serde_json::Error> {
        serde_json::from_slice(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trips_a_value_through_bytes() {
        let encoded = Json.encode(&vec![1u64, 2, 3]).unwrap();

        assert_eq!(encoded.as_ref(), b"[1,2,3]");
        assert_eq!(
            Decoder::<Vec<u64>>::decode(&Json, &encoded).unwrap(),
            vec![1, 2, 3]
        );
    }

    #[test]
    fn json_writes_into_a_buffer_that_has_room_without_allocating() {
        let mut buffer = Vec::with_capacity(64);
        buffer.extend_from_slice(b"data: ");

        let allocations = allocation_counter::measure(|| {
            Json.encode_into(&[1u64, 2, 3], &mut buffer).unwrap();
        });

        assert_eq!(allocations.count_total, 0);
        assert_eq!(buffer, b"data: [1,2,3]");
    }

    #[test]
    fn json_refuses_bytes_that_do_not_hold_the_output() {
        assert!(Decoder::<Vec<u64>>::decode(&Json, b"not json").is_err());
    }
}
