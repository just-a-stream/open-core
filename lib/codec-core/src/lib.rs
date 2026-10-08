#![doc = include_str!("../README.md")]

use bytes::Bytes;

pub trait Encoder<Input>
where
    Input: ?Sized,
{
    type Error;

    fn encode_into(&self, input: &Input, buffer: &mut Vec<u8>) -> Result<(), Self::Error>;

    fn encode(&self, input: &Input) -> Result<Bytes, Self::Error> {
        let mut encoded = Vec::new();
        self.encode_into(input, &mut encoded)?;

        Ok(Bytes::from(encoded))
    }
}

pub trait Decoder<'de, Output> {
    type Error;

    fn decode(&self, input: &'de [u8]) -> Result<Output, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::Infallible;

    struct Verbatim;

    impl Encoder<str> for Verbatim {
        type Error = Infallible;

        fn encode_into(&self, input: &str, buffer: &mut Vec<u8>) -> Result<(), Infallible> {
            buffer.extend_from_slice(input.as_bytes());

            Ok(())
        }
    }

    #[test]
    fn an_encoder_answers_owned_bytes_by_writing_into_a_buffer_of_its_own() {
        let encoded = Verbatim.encode("[1]").unwrap();

        assert_eq!(encoded, Bytes::from_static(b"[1]"));
    }
}
