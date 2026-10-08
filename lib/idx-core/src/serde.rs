pub use ::serde::{Deserialize, Deserializer, Serialize, Serializer};

#[doc(hidden)]
#[macro_export]
macro_rules! __define_index_serde {
    ($name:ident) => {
        impl $crate::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
            where
                S: $crate::serde::Serializer,
            {
                serializer.serialize_u64(self.0)
            }
        }

        impl<'de> $crate::serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
            where
                D: $crate::serde::Deserializer<'de>,
            {
                <u64 as $crate::serde::Deserialize>::deserialize(deserializer).map(Self)
            }
        }
    };
}
