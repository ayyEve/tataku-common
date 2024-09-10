use std::borrow::Cow;

#[derive(Debug, Eq, PartialEq)]
pub enum ReflectError<'a> {
    EntryNotExist {
        entry: Cow<'a, str>
    },
    OptionIsNone,

    ValueWrongType {
        actual: &'static str,
        provided: &'static str,
    },

    InvalidHashmapKey,
    InvalidIndex,
    NoHashmapKey {
        key: Cow<'a, str>,
    },
    HashmapKeyNotProvided,

    WrongVariant {
        actual: &'static str,
        provided: &'static str,
    },

    OutOfBounds {
        length: usize,
        index: usize
    },

    CantMutHashSetKey,
    ImmutableContainer,

    NoFromString,
}
impl<'a> ReflectError<'a> {
    pub fn entry_not_exist(entry: impl Into<Cow<'a, str>>) -> Self {
        Self::EntryNotExist { entry: entry.into() }
    }
    pub fn wrong_type(actual: &'static str, provided: &'static str) -> Self {
        Self::ValueWrongType { actual, provided }
    }
    pub fn wrong_variant(actual: &'static str, provided: &'static str) -> Self {
        Self::WrongVariant { actual, provided }
    }


    pub fn to_owned(self) -> ReflectError<'static> {
        fn own<'a>(cow: Cow<'a, str>) -> Cow<'static, str> {
            Cow::<'static, str>::Owned(cow.into_owned())
        }
        match self {
            Self::EntryNotExist { entry } => ReflectError::EntryNotExist { entry: own(entry) },
            Self::ValueWrongType { actual, provided } => ReflectError::ValueWrongType { actual, provided },
            Self::InvalidHashmapKey => ReflectError::InvalidHashmapKey,
            Self::InvalidIndex => ReflectError::InvalidIndex,
            Self::NoHashmapKey { key } => ReflectError::NoHashmapKey { key: own(key) },
            Self::HashmapKeyNotProvided => ReflectError::HashmapKeyNotProvided,
            Self::WrongVariant { actual, provided } => ReflectError::WrongVariant { actual, provided },
            Self::OutOfBounds { length, index } => ReflectError::OutOfBounds { length, index },
            Self::CantMutHashSetKey => ReflectError::CantMutHashSetKey,
            Self::ImmutableContainer => ReflectError::ImmutableContainer,
            Self::OptionIsNone => ReflectError::OptionIsNone,
            Self::NoFromString => ReflectError::NoFromString
        }
    }
}