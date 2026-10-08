#[macro_export]
macro_rules! define_index {
    ($($(#[$meta:meta])* $name:ident),+ $(,)?) => {$(
        $(#[$meta])*
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
        pub struct $name(u64);

        impl $name {
            pub const ZERO: Self = Self(0);

            pub const fn new(value: u64) -> Self {
                Self(value)
            }

            pub const fn value(&self) -> u64 {
                self.0
            }

            pub const fn next(self) -> Option<Self> {
                match self.0.checked_add(1) {
                    Some(value) => Some(Self(value)),
                    None => None,
                }
            }

            pub const fn advance(&mut self) -> Option<Self> {
                let current = *self;
                let Some(next) = current.next() else {
                    return None;
                };
                *self = next;

                Some(current)
            }
        }

        impl From<u64> for $name {
            fn from(value: u64) -> Self {
                Self(value)
            }
        }

        impl From<$name> for u64 {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl ::core::fmt::Display for $name {
            fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Display::fmt(&self.0, formatter)
            }
        }

        impl ::core::str::FromStr for $name {
            type Err = $crate::IndexUnparseable;

            fn from_str(text: &str) -> ::core::result::Result<Self, Self::Err> {
                $crate::index_value_parsed(text).map(Self)
            }
        }

        $crate::__define_index_serde!($name);
    )+};
}

#[cfg(not(feature = "serde"))]
#[doc(hidden)]
#[macro_export]
macro_rules! __define_index_serde {
    ($name:ident) => {};
}
