use std::borrow::Cow;

#[derive(Debug, Eq, PartialEq)]
pub enum ReflectError<'a> {
    EntryNotExist {
        entry: Cow<'a, str>
    },
    OptionIsNone,
    NotANumber,
    NoDisplay,
    NoIter,

    ValueWrongType {
        expected: &'static str,
        actual: &'static str,
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
    pub fn wrong_type(
        actual: &'static str,
        incoming: &'static str, 
    ) -> Self {
        Self::ValueWrongType { expected: incoming, actual }
    }
    pub fn wrong_variant(actual: &'static str, provided: &'static str) -> Self {
        Self::WrongVariant { actual, provided }
    }


    pub fn to_owned(self) -> ReflectError<'static> {
        fn own(cow: Cow<'_, str>) -> Cow<'static, str> {
            Cow::<'static, str>::Owned(cow.into_owned())
        }
        match self {
            Self::EntryNotExist { entry } => ReflectError::EntryNotExist { entry: own(entry) },
            Self::ValueWrongType { expected: actual, actual: provided } => ReflectError::ValueWrongType { expected: actual, actual: provided },
            Self::InvalidHashmapKey => ReflectError::InvalidHashmapKey,
            Self::InvalidIndex => ReflectError::InvalidIndex,
            Self::NoHashmapKey { key } => ReflectError::NoHashmapKey { key: own(key) },
            Self::HashmapKeyNotProvided => ReflectError::HashmapKeyNotProvided,
            Self::WrongVariant { actual, provided } => ReflectError::WrongVariant { actual, provided },
            Self::OutOfBounds { length, index } => ReflectError::OutOfBounds { length, index },
            Self::CantMutHashSetKey => ReflectError::CantMutHashSetKey,
            Self::ImmutableContainer => ReflectError::ImmutableContainer,
            Self::OptionIsNone => ReflectError::OptionIsNone,
            Self::NoFromString => ReflectError::NoFromString,

            Self::NotANumber => ReflectError::NotANumber,
            Self::NoDisplay => ReflectError::NoDisplay,
            Self::NoIter => ReflectError::NoIter,
        }
    }
}
