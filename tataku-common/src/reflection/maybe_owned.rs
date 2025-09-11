use crate::reflection::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MaybeOwned<'a, T> {
    Borrowed(&'a T),
    Owned(T),
}
impl<T:Clone> MaybeOwned<'_, T> {
    pub fn cloned(&self) -> T {
        match self {
            Self::Borrowed(t) => (*t).clone(),
            Self::Owned(t) => t.clone(),
        }
    }
}
impl<T:Copy> MaybeOwned<'_, T> {
    pub fn copied(&self) -> T {
        match self {
            Self::Borrowed(t) => **t,
            Self::Owned(t) => *t,
        }
    }
}
impl<T> std::ops::Deref for MaybeOwned<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(b) => b,
            Self::Owned(b) => b,
        }
    }
}



pub enum MaybeOwnedReflect<'a> {
    Borrowed(&'a dyn Reflect),
    Owned(Box<dyn Reflect>),
}
impl<R:Reflect> From<R> for MaybeOwnedReflect<'_> {
    fn from(value: R) -> Self {
        Self::Owned(Box::new(value))
    }
}
impl<'a> From<&'a dyn Reflect> for MaybeOwnedReflect<'a> {
    fn from(value: &'a dyn Reflect) -> Self {
        Self::Borrowed(value)
    }
}
impl AsRef<dyn Reflect> for MaybeOwnedReflect<'_> {
    fn as_ref(&self) -> &dyn Reflect {
        match self {
            Self::Borrowed(r) => *r,
            Self::Owned(b) => &**b
        }
    }
}
