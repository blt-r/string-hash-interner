use crate::{StringInterner, Symbol};
use alloc::boxed::Box;
use core::{default::Default, fmt, hash::BuildHasher, marker};
use serde::{
    de::{Deserialize, Deserializer, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};

impl<S: Symbol, H: BuildHasher> Serialize for StringInterner<S, H> {
    fn serialize<T>(&self, serializer: T) -> Result<T::Ok, T::Error>
    where
        T: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for (_symbol, string) in self {
            seq.serialize_element(string)?
        }
        seq.end()
    }
}

impl<'de, S: Symbol, H: BuildHasher + Default> Deserialize<'de> for StringInterner<S, H> {
    fn deserialize<D>(deserializer: D) -> Result<StringInterner<S, H>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(StringInternerVisitor::default())
    }
}

struct StringInternerVisitor<S: Symbol, H: BuildHasher> {
    mark: marker::PhantomData<(S, H)>,
}

impl<S: Symbol, H: BuildHasher> Default for StringInternerVisitor<S, H> {
    fn default() -> Self {
        StringInternerVisitor {
            mark: marker::PhantomData,
        }
    }
}

impl<'de, S: Symbol, H: BuildHasher + Default> Visitor<'de> for StringInternerVisitor<S, H> {
    type Value = StringInterner<S, H>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Expected a contiguous sequence of strings.")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut interner: StringInterner<S, H> =
            StringInterner::with_capacity_and_hasher(seq.size_hint().unwrap_or(0), H::default());
        while let Some(s) = seq.next_element::<Box<str>>()? {
            interner.intern_and_hash(s);
        }
        Ok(interner)
    }
}

macro_rules! impl_serde_for_symbol {
    ($name:ident, $ty:ty) => {
        impl ::serde::Serialize for $crate::symbol::$name {
            fn serialize<T: ::serde::Serializer>(
                &self,
                serializer: T,
            ) -> ::core::result::Result<T::Ok, T::Error> {
                self.value.serialize(serializer)
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $crate::symbol::$name {
            fn deserialize<D: ::serde::Deserializer<'de>>(
                deserializer: D,
            ) -> ::core::result::Result<Self, D::Error> {
                let index = <$ty as ::serde::Deserialize<'de>>::deserialize(deserializer)?;
                let ::core::option::Option::Some(symbol) = Self::new(index) else {
                    return ::core::result::Result::Err(<D::Error as ::serde::de::Error>::custom(
                        ::core::concat!(
                            "invalid index value for `",
                            ::core::stringify!($name),
                            "`"
                        ),
                    ));
                };
                ::core::result::Result::Ok(symbol)
            }
        }
    };
}
impl_serde_for_symbol!(SymbolU16, u16);
impl_serde_for_symbol!(SymbolU32, u32);
impl_serde_for_symbol!(SymbolUsize, usize);
