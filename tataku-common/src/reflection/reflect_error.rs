use std::borrow::Cow;

pub type Result<'a, T> = std::result::Result<T, ReflectError<'a>>;

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
        expected: Cow<'a, str>,
        provided: Cow<'a, str>,
    },

    InvalidHashmapKey,
    InvalidIndex,
    NoHashmapKey {
        key: Cow<'a, str>,
    },
    HashmapKeyNotProvided,

    WrongVariant {
        expected: Cow<'a, str>,
        provided: Cow<'a, str>,
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
        expected: impl Into<Cow<'a, str>>,
        provided: impl Into<Cow<'a, str>>, 
    ) -> Self {
        Self::ValueWrongType { expected: expected.into(), provided: provided.into() }
    }
    pub fn wrong_variant(
        expected: impl Into<Cow<'a, str>>,
        provided: impl Into<Cow<'a, str>>, 
    ) -> Self {
        Self::WrongVariant { expected: expected.into(), provided: provided.into() }
    }


    pub fn to_owned(self) -> ReflectError<'static> {
        fn own(cow: Cow<'_, str>) -> Cow<'static, str> {
            Cow::<'static, str>::Owned(cow.into_owned())
        }
        match self {
            Self::EntryNotExist { entry } => ReflectError::EntryNotExist { entry: own(entry) },
            Self::ValueWrongType { expected, provided } => ReflectError::ValueWrongType { expected: own(expected), provided: own(provided) },
            Self::InvalidHashmapKey => ReflectError::InvalidHashmapKey,
            Self::InvalidIndex => ReflectError::InvalidIndex,
            Self::NoHashmapKey { key } => ReflectError::NoHashmapKey { key: own(key) },
            Self::HashmapKeyNotProvided => ReflectError::HashmapKeyNotProvided,
            Self::WrongVariant { expected, provided } => ReflectError::WrongVariant { expected: own(expected), provided: own(provided) },
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
