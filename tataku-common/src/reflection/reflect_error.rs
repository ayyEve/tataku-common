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
}