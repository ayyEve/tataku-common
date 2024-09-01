use std::borrow::Cow;

#[derive(Debug, Eq, PartialEq)]
pub enum ReflectError<'a> {
    EntryDoesntExist {
        entry: Cow<'a, str>
    },

    ValueWrongType {
        actual: Cow<'a, str>,
        provided: Cow<'a, str>
    },

    InvalidHashmapKey,
    InvalidIndex,
    NoHashmapKey {
        key: Cow<'a, str>,
    },
    HashmapKeyNotProvided,

    WrongVariant {
        actual: Cow<'a, str>,
        provided: Cow<'a, str>,
    },
    
    OutOfBounds {
        length: usize,
        index: usize
    },

    CantMutHashSetKey,
    ImmutibleContainer,
}
impl<'a> ReflectError<'a> {
    pub fn entry_not_exist(entry: impl Into<Cow<'a, str>>) -> Self {
        Self::EntryDoesntExist { entry: entry.into() }
    }
    pub fn wrong_type(actual: impl Into<Cow<'a, str>>, provided: impl Into<Cow<'a, str>>) -> Self {
        Self::ValueWrongType { actual: actual.into(), provided: provided.into() }
    }
    pub fn wrong_variant(actual: impl Into<Cow<'a, str>>, provided: impl Into<Cow<'a, str>>) -> Self {
        Self::WrongVariant { actual: actual.into(), provided: provided.into() }
    }


    pub fn to_owned(self) -> ReflectError<'static> {
        fn own<'a>(cow: Cow<'a, str>) -> Cow<'static, str> {
            Cow::<'static, str>::Owned(cow.into_owned())
        }
        match self {
            Self::EntryDoesntExist { entry } => ReflectError::EntryDoesntExist { entry: own(entry) },
            Self::ValueWrongType { actual, provided } => ReflectError::ValueWrongType { 
                actual: own(actual), 
                provided: own(provided)
            },
            Self::InvalidHashmapKey => ReflectError::InvalidHashmapKey,
            Self::InvalidIndex => ReflectError::InvalidIndex,
            Self::NoHashmapKey { key } => ReflectError::NoHashmapKey { key: own(key) },
            Self::HashmapKeyNotProvided => ReflectError::HashmapKeyNotProvided,
            Self::WrongVariant { actual, provided } => ReflectError::WrongVariant { actual: own(actual), provided: own(provided) },
            Self::OutOfBounds { length, index } => ReflectError::OutOfBounds { length, index },
            Self::CantMutHashSetKey => ReflectError::CantMutHashSetKey,
            Self::ImmutibleContainer => ReflectError::ImmutibleContainer,
        }
    }
}