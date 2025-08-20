use crate::prelude::*;
use std::any::type_name;

pub type ReflectResult<'a, T> = Result<T, ReflectError<'a>>;

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


pub trait Reflect: downcast_rs::DowncastSync {
    fn type_name(&self) -> &'static str {
        type_name::<Self>()
    }

    fn as_dyn(&self) -> &dyn Reflect where Self: Sized { self }
    fn as_dyn_mut(&mut self) -> &mut dyn Reflect where Self: Sized { self }

    fn impl_get<'s, 'v>(&'s self, path: ReflectPath<'v>) -> ReflectResult<'v, MaybeOwnedReflect<'s>>;
    fn impl_get_mut<'s, 'v>(&'s mut self, path: ReflectPath<'v>) -> ReflectResult<'v, &'s mut dyn Reflect>;

    fn impl_insert<'v>(&mut self, path: ReflectPath<'v>, value: Box<dyn Reflect>) -> ReflectResult<'v, ()>;

    fn impl_iter<'s, 'v>(&'s self, _path: ReflectPath<'v>) -> ReflectResult<'v, ReflectIter<'s>> {
        // Ok(Default::default())
        Err(ReflectError::NoIter)
    }
    fn impl_iter_mut<'s, 'v>(&'s mut self, _path: ReflectPath<'v>) -> ReflectResult<'v, ReflectIterMut<'s>> {
        // Ok(Default::default())
        Err(ReflectError::NoIter)
    }

    fn impl_display<'v>(&self, path: ReflectPath<'v>, precision: Option<usize>) -> ReflectResult<'v, String> {
        if !path.has_next() {
            return Ok("No Reflect Display".to_string());
        }
        match self.impl_get(path)? {
            MaybeOwnedReflect::Borrowed(reflect) => reflect.reflect_display(ReflectPath::new(""), precision),
            MaybeOwnedReflect::Owned(reflect) => reflect.reflect_display(ReflectPath::new(""), precision),
        }
    }
    
    fn impl_as_number<'v>(&self, path: ReflectPath<'v>) -> ReflectResult<'v, ReflectNumber> {
        if !path.has_next() {
            return Err(ReflectError::NotANumber);
        }
        match self.impl_get(path)? {
            MaybeOwnedReflect::Borrowed(reflect) => reflect.reflect_as_number(ReflectPath::new("")),
            MaybeOwnedReflect::Owned(reflect) => reflect.reflect_as_number(ReflectPath::new("")),
        }
    }
    

    fn duplicate(&self) -> Option<Box<dyn Reflect>>;

    fn from_string(str: &str) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized;
}

impl dyn Reflect {
    pub fn reflect_get<'a, T: Reflect + 'static>(&self, path: impl Into<ReflectPath<'a>>) -> ReflectResult<'a, MaybeOwned<T>> {
        let a = self.impl_get(path.into())?;
        let wrong_type = ReflectError::wrong_type(Reflect::type_name(a.as_ref()), type_name::<T>());
        match a {
            MaybeOwnedReflect::Borrowed(b) => b.downcast_ref::<T>().ok_or(wrong_type).map(MaybeOwned::Borrowed),
            MaybeOwnedReflect::Owned(b) => b.downcast().map_err(|_| wrong_type).map(|i| MaybeOwned::Owned(*i)),
        }
    }

    pub fn reflect_get_mut<'a, T: Reflect + 'static>(&mut self, path: impl Into<ReflectPath<'a>>) -> ReflectResult<'a, &mut T> {
        let a = self.impl_get_mut(path.into())?;
        let name = a.type_name();
        a.downcast_mut::<T>()
            .ok_or(ReflectError::wrong_type(name, type_name::<T>()))
    }

    pub fn reflect_insert<'a, T: Reflect + 'static>(&mut self, path: impl Into<ReflectPath<'a>>, value: T) -> ReflectResult<'a, ()> {
        self.impl_insert(path.into(), Box::new(value))
    }

    pub fn reflect_iter<'a>(&self, path: impl Into<ReflectPath<'a>>) -> ReflectResult<'a, ReflectIter<'_>> {
        self.impl_iter(path.into())
    }

    pub fn reflect_iter_mut<'a>(&mut self, path: impl Into<ReflectPath<'a>>) -> ReflectResult<'a, ReflectIterMut<'_>> {
        self.impl_iter_mut(path.into())
    }

    pub fn reflect_as_number<'a>(&self, path: impl Into<ReflectPath<'a>>) -> ReflectResult<'a, ReflectNumber> {
        self.impl_as_number(path.into())
    }

    pub fn reflect_display<'a>(&self, path: impl Into<ReflectPath<'a>>, precision: Option<usize>) -> ReflectResult<'a, String> {
        self.impl_display(path.into(), precision)
    }  
}
downcast_rs::impl_downcast!(sync Reflect);



#[macro_export]
macro_rules! base_reflect_impl {
    (impl<$($g:ty),*> for $ty:ty where $($where:tt)*) => {
        impl<$($g),*> Reflect for $ty where $($where)* {
            fn impl_get<'a, 's>(&'s self, mut path: ReflectPath<'a>) -> ReflectResult<'a, MaybeOwnedReflect<'s>> {
                if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }

                Ok((self as &dyn Reflect).into())
            }

            fn impl_get_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> ReflectResult<'a, &mut dyn Reflect> {
                if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }

                Ok(self as &mut dyn Reflect)
            }

            fn impl_insert<'a>(&mut self, mut path: ReflectPath<'a>, value: Box<dyn Reflect>) -> ReflectResult<'a, ()> {
                if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }

                value
                    .downcast::<$ty>()
                    .map(|a| *self = *a)
                    .map_err(|v| ReflectError::wrong_type(type_name::<$ty>(), v.type_name()))
            }


            fn impl_display<'a>(&self, mut path: ReflectPath<'a>, _precision: Option<usize>) -> ReflectResult<'a, String> {
                if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }
                Ok(format!("{self}"))
            }

            fn duplicate(&self) -> Option<Box<dyn Reflect>> {
                Some(Box::new(self.clone()))
            }

            fn from_string(str: &str) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized {
                Ok(Box::new(
                    str.parse::<$ty>().map_err(|_| ReflectError::wrong_type(stringify!($ty), "String"))?
                ))
            }
        }
    };

    ($($ty:ty),*) => { $( base_reflect_impl!(impl<> for $ty where ); )* };
}
base_reflect_impl!(
    bool,
    String
);

macro_rules! immutable_str {
    ($($t: ty),*$(,)?) => {$(
        impl Reflect for $t {
            fn impl_get<'a, 's>(&'s self, mut path: ReflectPath<'a>) -> ReflectResult<'a, MaybeOwnedReflect<'s>> {
                if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }

                Ok((self as &dyn Reflect).into())
            }

            fn impl_get_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> ReflectResult<'a, &mut dyn Reflect> {
                if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }

                Ok(self as &mut dyn Reflect)
            }

            fn impl_insert<'a>(&mut self, mut path: ReflectPath<'a>, value: Box<dyn Reflect>) -> ReflectResult<'a, ()> {
                if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }

                value
                    .downcast::<$t>()
                    .map(|a| *self = *a)
                    .map_err(|v| ReflectError::wrong_type(type_name::<str>(), v.type_name()))
            }

            fn impl_display<'v>(&self, mut path: ReflectPath<'v>, _precision: Option<usize>) -> ReflectResult<'v, String> {
                if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }
                Ok((*self).to_string())
            }

            fn duplicate(&self) -> Option<Box<dyn Reflect>> {
                Some(Box::new(self.to_owned()))
            }

            fn from_string(str: &str) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized {
                Ok(Box::new(str.to_owned()))
            }
        }
    )*}
}
immutable_str!(&'static str);

impl Reflect for std::sync::Arc<str> {
    fn impl_get<'a, 's>(&'s self, mut path: ReflectPath<'a>) -> ReflectResult<'a, MaybeOwnedReflect<'s>> {
        if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }

        Ok((self as &dyn Reflect).into())
    }

    fn impl_get_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> ReflectResult<'a, &mut dyn Reflect> {
        if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }

        Ok(self as &mut dyn Reflect)
    }

    fn impl_insert<'a>(&mut self, mut path: ReflectPath<'a>, value: Box<dyn Reflect>) -> ReflectResult<'a, ()> {
        if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }

        use std::sync::Arc;
        *self = ReflectMultiparse::<Arc<str>>::parse(value, |v| 
            v
                .try_downcast::<Arc<str>>()?
                .try_downcast::<String>()?
                .try_downcast::<&'static str>()
        ).map_err(|v| ReflectError::wrong_type(
            type_name::<str>(), 
            v.type_name()
        ))?;

        Ok(())
    }

    fn impl_display<'v>(&self, mut path: ReflectPath<'v>, _precision: Option<usize>) -> ReflectResult<'v, String> {
        if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }
        Ok((*self).to_string())
    }

    fn duplicate(&self) -> Option<Box<dyn Reflect>> {
        Some(Box::new(self.to_owned()))
    }

    fn from_string(str: &str) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized {
        Ok(Box::new(str.to_owned()))
    }
}


impl<T:Reflect+Clone> Reflect for Option<T> {
    fn impl_get<'a, 's>(&'s self, path: ReflectPath<'a>) -> ReflectResult<'a, MaybeOwnedReflect<'s>> {
        let next = path.clone().next();
        match (next, self) {
            (None, None) => Err(ReflectError::OptionIsNone),
            (None, Some(s)) => Ok((s as &dyn Reflect).into()),
            (Some("is_some"), None) => Ok(MaybeOwnedReflect::Owned(Box::new(false))),
            (Some("is_some"), Some(_)) => Ok(MaybeOwnedReflect::Owned(Box::new(true))),
            (Some(_), Some(s)) => s.impl_get(path),
            (Some(_), None) => Err(ReflectError::OptionIsNone)
        }
    }

    fn impl_get_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> ReflectResult<'a, &mut dyn Reflect> {
        if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }

        Ok(self as &mut dyn Reflect)
    }

    fn impl_insert<'a>(&mut self, mut path: ReflectPath<'a>, value: Box<dyn Reflect>) -> ReflectResult<'a, ()> {
        if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }

        if value.is::<T>() {
            let _ = value
                .downcast::<T>()
                .map(|a| *self = Some(*a));

            return Ok(())
        }

        value
            .downcast::<Option<T>>()
            .map(|a| *self = *a)
            .map_err(|v| ReflectError::wrong_type(type_name::<Self>(), v.type_name()))?;
        Ok(())
    }

    fn duplicate(&self) -> Option<Box<dyn Reflect>> {
        Some(Box::new(self.clone()))
    }

    fn from_string(str: &str) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized {
        T::from_string(str)
    }
}



pub trait Stringable: Sized {
    type Err;
    fn parse_str(s: &str) -> Result<Self, Self::Err>;
}
macro_rules! ugh {
    ($($ty: ty),*$(,)?) => {$(
        impl Stringable for $ty {
            type Err = <$ty as std::str::FromStr>::Err;
            fn parse_str(s: &str) -> Result<Self, Self::Err> {
                s.parse()
            }
        }

    )*}
}
ugh!(
    u8, u16, u32, u64, u128, usize,
    i8, i16, i32, i64, i128, isize,
    String,
);

impl Stringable for std::sync::Arc<str> {
    type Err = ();
    fn parse_str(s: &str) -> Result<Self, Self::Err> {
        Ok(<std::sync::Arc<str> as From<&str>>::from(s))
    }
}

impl<K, V> Reflect for std::collections::HashMap<K, V>
    where
    K: Stringable
        + std::string::ToString 
        + core::hash::Hash 
        + core::cmp::Eq 
        + Send + Sync 
        + Reflect
        + 'static, 
    V: Reflect
{
    fn impl_get<'a, 's>(
        &'s self, 
        mut path: ReflectPath<'a>
    ) -> ReflectResult<'a, MaybeOwnedReflect<'s>> {
        let Some(key) = path.next() else {
            return Ok((self as &dyn Reflect).into())
        };

        let key = K::parse_str(key).map_err(|_| ReflectError::InvalidHashmapKey)?;

        let val = self.get(&key)
            .ok_or(ReflectError::entry_not_exist(key.to_string()))?;

        Ok((val as &dyn Reflect).into())
    }

    fn impl_get_mut<'a>(
        &mut self, 
        mut path: ReflectPath<'a>
    ) -> ReflectResult<'a, &mut dyn Reflect> {
        let Some(key) = path.next() else {
            return Ok(self as &mut dyn Reflect)
        };
        let key = K::parse_str(key).map_err(|_| ReflectError::InvalidHashmapKey)?;

        let val = self.get_mut(&key)
            .ok_or(ReflectError::entry_not_exist(key.to_string()))?;

        Ok(val as &mut dyn Reflect)
    }

    fn impl_insert<'a>(
        &mut self, 
        mut path: ReflectPath<'a>, 
        value: Box<dyn Reflect>
    ) -> ReflectResult<'a, ()> {
        let Some(key) = path.next() else {
            return value
                .downcast::<Self>()
                .map(|a| *self = *a)
                .map_err(|v| ReflectError::wrong_type(type_name::<Self>(), v.type_name()))
        };
        let key = K::parse_str(key).map_err(|_| ReflectError::InvalidHashmapKey)?;

        let value = value
            .downcast::<V>()
            .map_err(|v| ReflectError::wrong_type(type_name::<V>(), v.type_name()))?;

        self.insert(key, *value);
        Ok(())
    }

    fn impl_iter<'s, 'a>(
        &'s self, 
        mut path: ReflectPath<'a>
    ) -> ReflectResult<'a, ReflectIter<'s>> {
        let Some(key) = path.next() else {
            return Ok(ReflectIter { 
                iter: self.iter().map(|(k, v)| {
                    ReflectIterEntry {
                        item: v as &dyn Reflect,
                        index: Some(ReflectItemIndex::Value(k as &dyn Reflect))
                    }
                })
                .collect()
            })
        };

        
        let key = K::parse_str(key).map_err(|_| ReflectError::InvalidHashmapKey)?;
        let val = self.get(&key)
            .ok_or(ReflectError::entry_not_exist(key.to_string()))?;

        val.impl_iter(path)
    }
    fn impl_iter_mut<'a>(
        &mut self,
        mut path: ReflectPath<'a>
    ) -> ReflectResult<'a, ReflectIterMut<'_>> {
        let Some(key) = path.next() else {
            return Ok(ReflectIterMut { 
                iter: self.iter_mut().map(|(k, v)| {
                    ReflectIterMutEntry {
                        item: v as &mut dyn Reflect,
                        index: Some(ReflectItemIndex::Value(k as &dyn Reflect))
                    }
                })
                .collect()
            });
        };

        let key = K::parse_str(key).map_err(|_| ReflectError::InvalidHashmapKey)?;
        let val = self.get_mut(&key)
            .ok_or(ReflectError::entry_not_exist(key.to_string()))?;

        val.impl_iter_mut(path)
    }


    fn impl_as_number<'v>(
        &self, 
        mut path: ReflectPath<'v>
    ) -> ReflectResult<'v, ReflectNumber> {
        let Some(key) = path.next() else {
            return Err(ReflectError::NotANumber);
        };

        let key = K::parse_str(key).map_err(|_| ReflectError::InvalidHashmapKey)?;

        let val = self.get(&key)
            .ok_or(ReflectError::entry_not_exist(key.to_string()))?;

        val.impl_as_number(path)
    }
    fn impl_display<'v>(
        &self, 
        mut path: ReflectPath<'v>, 
        precision: Option<usize>
    ) -> ReflectResult<'v, String> {
        let Some(key) = path.next() else {
            return Err(ReflectError::NoDisplay);
        };

        let key = K::parse_str(key).map_err(|_| ReflectError::InvalidHashmapKey)?;
        let val = self.get(&key)
            .ok_or(ReflectError::entry_not_exist(key.to_string()))?;

        val.impl_display(path, precision)
    }

    fn from_string(
        _str: &str
    ) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized {
        Err(ReflectError::NoFromString)
    }

    fn duplicate(&self) -> Option<Box<dyn Reflect>> {
        None
    }
}



impl<T> Reflect for std::collections::HashSet<T>
    where T: Reflect 
    + Stringable
    + std::string::ToString 
    + core::hash::Hash 
    + core::cmp::Eq 
    + 'static,
{
    fn impl_get<'a, 's>(&'s self, mut path: ReflectPath<'a>) -> ReflectResult<'a, MaybeOwnedReflect<'s>> {
        let Some(key) = path.next() else {
            return Ok((self as &dyn Reflect).into())
        };

        let key = T::parse_str(key).map_err(|_| ReflectError::InvalidHashmapKey)?;

        let val = self.get(&key)
            .ok_or(ReflectError::entry_not_exist(key.to_string()))?;

        Ok((val as &dyn Reflect).into())
    }

    fn impl_get_mut<'a>(&mut self, path: ReflectPath<'a>) -> ReflectResult<'a, &mut dyn Reflect> {
        if !path.has_next() {
            return Ok(self as &mut dyn Reflect)
        };

        Err(ReflectError::CantMutHashSetKey)
    }

    fn impl_insert<'a>(&mut self, path: ReflectPath<'a>, value: Box<dyn Reflect>) -> ReflectResult<'a, ()> {
        if !path.has_next() {
            return value
                .downcast::<Self>()
                .map(|a| *self = *a)
                .map_err(|v| ReflectError::wrong_type(type_name::<Self>(), v.type_name()))
        };
        // let key = key.parse::<T>().map_err(|_| ReflectError::InvalidHashmapKey)?;

        let value = value
            .downcast::<T>()
            .map_err(|v| ReflectError::wrong_type(type_name::<T>(), v.type_name()))?;

        self.insert(*value);
        Ok(())
    }

    fn impl_iter<'a>(&self, mut path: ReflectPath<'a>) -> ReflectResult<'a, ReflectIter<'_>> {
        let Some(key) = path.next() else {
            return Ok(ReflectIter { 
                iter: self.iter().enumerate().map(|(k, v)| {
                    ReflectIterEntry {
                        item: v as &dyn Reflect,
                        index: Some(ReflectItemIndex::Number(k))
                    }
                })
                .collect()
            })
        };

        let key = T::parse_str(key).map_err(|_| ReflectError::InvalidHashmapKey)?;
        let val = self.get(&key)
            .ok_or(ReflectError::entry_not_exist(key.to_string()))?;

        val.impl_iter(path)
    }
    fn impl_iter_mut<'a>(&mut self, _path: ReflectPath<'a>) -> ReflectResult<'a, ReflectIterMut<'_>> {
        Err(ReflectError::CantMutHashSetKey)
    }

    fn duplicate(&self) -> Option<Box<dyn Reflect>> {
        None
    }

    fn from_string(_: &str) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized {
        Err(ReflectError::NoFromString)
    }
}

impl<T: Reflect + Clone> Reflect for Vec<T> {
    fn impl_get<'a, 's>(&'s self, mut path: ReflectPath<'a>) -> ReflectResult<'a, MaybeOwnedReflect<'s>> {
        let Some(key) = path.next() else {
            return Ok((self as &dyn Reflect).into())
        };

        match key {
            "len" | "length" | "count" => Ok(MaybeOwnedReflect::Owned(Box::new(self.len()))),
            "empty" | "is_empty" => Ok(MaybeOwnedReflect::Owned(Box::new(self.is_empty()))),

            index => {
                let index = index.parse::<usize>()
                    .map_err(|_| ReflectError::InvalidIndex)?;

                let val = self
                    .get(index)
                    .ok_or(ReflectError::entry_not_exist(index.to_string()))?;

                Ok((val as &dyn Reflect).into())
            }
        }

    }

    fn impl_get_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> ReflectResult<'a, &mut dyn Reflect> {
        let Some(index) = path.next() else {
            return Ok(self as &mut dyn Reflect)
        };
        let index = index.parse::<usize>().map_err(|_| ReflectError::InvalidIndex)?;

        let val = self.get_mut(index)
            .ok_or(ReflectError::entry_not_exist(index.to_string()))?;

        Ok(val as &mut dyn Reflect)
    }

    fn impl_insert<'a>(
        &mut self, 
        mut path: ReflectPath<'a>, 
        value: Box<dyn Reflect>,
    ) -> ReflectResult<'a, ()> {
        let Some(index) = path.next() 
        else {
            if value.is::<T>() {
                let _ = value
                    .downcast::<T>()
                    .map(|a| self.push(*a));
            } else {
                value
                    .downcast::<Self>()
                    .map(|a| *self = *a)
                    .map_err(|v| ReflectError::wrong_type(
                        type_name::<Self>(), 
                        v.type_name()
                    ))?;
            }

            return Ok(())
        };


        let index = index
            .parse::<usize>()
            .map_err(|_| ReflectError::InvalidIndex)?;

        let value = value
            .downcast::<T>()
            .map_err(|v| ReflectError::wrong_type(
                type_name::<T>(), 
                v.type_name()
            ))?;

        if index >= self.len() {
            self.push(*value);
        } else {
            self[index] = *value;
        }

        Ok(())
    }



    fn impl_iter<'a>(&self, mut path: ReflectPath<'a>) -> ReflectResult<'a, ReflectIter<'_>> {
        let Some(index) = path.next() else {
            return Ok(ReflectIter { 
                iter: self.iter().enumerate().map(|(k, v)| {
                    ReflectIterEntry {
                        item: v as &dyn Reflect,
                        index: Some(ReflectItemIndex::Number(k))
                    }
                })
                .collect()
            })
        };

        let index = index.parse::<usize>().map_err(|_| ReflectError::InvalidIndex)?;
        let val = self.get(index)
            .ok_or(ReflectError::entry_not_exist(index.to_string()))?;

        val.impl_iter(path)
    }
    fn impl_iter_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> ReflectResult<'a, ReflectIterMut<'_>> {
        let Some(index) = path.next() else {
            return Ok(ReflectIterMut { 
                iter: self.iter_mut().enumerate().map(|(k, v)| {
                    ReflectIterMutEntry {
                        item: v as &mut dyn Reflect,
                        index: Some(ReflectItemIndex::Number(k))
                    }
                })
                .collect()
            })
        };

        let index = index.parse::<usize>().map_err(|_| ReflectError::InvalidHashmapKey)?;
        let val = self.get_mut(index)
            .ok_or(ReflectError::entry_not_exist(index.to_string()))?;

        val.impl_iter_mut(path)
    }

    fn duplicate(&self) -> Option<Box<dyn Reflect>> {
        Some(Box::new(self.clone()))
    }

    fn from_string(_: &str) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized {
        Err(ReflectError::NoFromString)
    }
}

impl<T: Reflect, const SIZE:usize> Reflect for [T; SIZE] where Self:Sized {
    fn impl_get<'a, 's>(&'s self, mut path: ReflectPath<'a>) -> ReflectResult<'a, MaybeOwnedReflect<'s>> {
        let Some(index) = path.next() else {
            return Ok((self as &dyn Reflect).into())
        };

        let index = index.parse::<usize>().map_err(|_| ReflectError::InvalidIndex)?;

        let val = self.get(index)
            .ok_or(ReflectError::entry_not_exist(index.to_string()))?;

        Ok((val as &dyn Reflect).into())
    }

    fn impl_get_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> ReflectResult<'a, &mut dyn Reflect> {
        let Some(index) = path.next() else {
            return Ok(self as &mut dyn Reflect)
        };
        let index = index.parse::<usize>().map_err(|_| ReflectError::InvalidIndex)?;

        let val = self.get_mut(index)
            .ok_or(ReflectError::entry_not_exist(index.to_string()))?;

        Ok(val as &mut dyn Reflect)
    }

    fn impl_insert<'a>(&mut self, mut path: ReflectPath<'a>, value: Box<dyn Reflect>) -> ReflectResult<'a, ()> {
        let Some(index) = path.next() else {

            value
                .downcast::<Self>()
                .map(|a| *self = *a)
                .map_err(|v| ReflectError::wrong_type(type_name::<Self>(), v.type_name()))?;

            return Ok(())
        };
        let index = index.parse::<usize>().map_err(|_| ReflectError::InvalidIndex)?;

        let value = value
            .downcast::<T>()
            .map_err(|v| ReflectError::wrong_type(type_name::<T>(), v.type_name()))?;

        if index >= self.len() {
            return Err(ReflectError::OutOfBounds {
                length: self.len(),
                index
            })
        } else {
            self[index] = *value
        }

        Ok(())
    }


    fn impl_iter<'a>(&self, mut path: ReflectPath<'a>) -> ReflectResult<'a, ReflectIter<'_>> {
        let Some(index) = path.next() else {
            return Ok(ReflectIter { 
                iter: self.iter().enumerate().map(|(k, v)| {
                    ReflectIterEntry {
                        item: v as &dyn Reflect,
                        index: Some(ReflectItemIndex::Number(k))
                    }
                })
                .collect()
            })
        };

        let index = index.parse::<usize>().map_err(|_| ReflectError::InvalidIndex)?;
        let val = self.get(index)
            .ok_or(ReflectError::entry_not_exist(index.to_string()))?;

        val.impl_iter(path)
    }
    fn impl_iter_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> ReflectResult<'a, ReflectIterMut<'_>> {
        let Some(index) = path.next() else {
            return Ok(ReflectIterMut { 
                iter: self.iter_mut().enumerate().map(|(k, v)| {
                    ReflectIterMutEntry {
                        item: v as &mut dyn Reflect,
                        index: Some(ReflectItemIndex::Number(k))
                    }
                })
                .collect()
            })
        };

        let index = index.parse::<usize>().map_err(|_| ReflectError::InvalidHashmapKey)?;
        let val = self.get_mut(index)
            .ok_or(ReflectError::entry_not_exist(index.to_string()))?;

        val.impl_iter_mut(path)
    }

    fn duplicate(&self) -> Option<Box<dyn Reflect>> {
        None
    }

    fn from_string(_: &str) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized {
        Err(ReflectError::NoFromString)
    }
}

impl<T: Reflect, const SIZE:usize> Reflect for &'static [T; SIZE] where Self:Sized {
    fn impl_get<'a, 's>(&'s self, mut path: ReflectPath<'a>) -> ReflectResult<'a, MaybeOwnedReflect<'s>> {
        let Some(index) = path.next() else {
            return Ok((self as &dyn Reflect).into())
        };

        let index = index.parse::<usize>().map_err(|_| ReflectError::InvalidIndex)?;

        let val = self.get(index)
            .ok_or(ReflectError::entry_not_exist(index.to_string()))?;

        Ok((val as &dyn Reflect).into())
    }

    fn impl_get_mut<'a>(&mut self, _path: ReflectPath<'a>) -> ReflectResult<'a, &mut dyn Reflect> {
        Err(ReflectError::ImmutableContainer)
    }

    fn impl_insert<'a>(&mut self, _path: ReflectPath<'a>, _value: Box<dyn Reflect>) -> ReflectResult<'a, ()> {
        Err(ReflectError::ImmutableContainer)
    }
    
    fn impl_iter<'a>(&self, mut path: ReflectPath<'a>) -> ReflectResult<'a, ReflectIter<'_>> {
        let Some(index) = path.next() else {
            return Ok(ReflectIter { 
                iter: self.iter().enumerate().map(|(k, v)| {
                    ReflectIterEntry {
                        item: v as &dyn Reflect,
                        index: Some(ReflectItemIndex::Number(k))
                    }
                })
                .collect()
            })
        };

        let index = index.parse::<usize>().map_err(|_| ReflectError::InvalidIndex)?;
        let val = self.get(index)
            .ok_or(ReflectError::entry_not_exist(index.to_string()))?;

        val.impl_iter(path)
    }
    fn impl_iter_mut<'a>(&mut self, _path: ReflectPath<'a>) -> ReflectResult<'a, ReflectIterMut<'_>> {
        Err(ReflectError::ImmutableContainer)
    }

    fn duplicate(&self) -> Option<Box<dyn Reflect>> {
        None
    }

    fn from_string(_: &str) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized {
        Err(ReflectError::NoFromString)
    }
}

impl<T: Reflect> Reflect for &'static [T] where Self:Sized {
    fn impl_get<'a, 's>(&'s self, mut path: ReflectPath<'a>) -> ReflectResult<'a, MaybeOwnedReflect<'s>> {
        let Some(index) = path.next() else {
            return Ok((self as &dyn Reflect).into())
        };

        let index = index.parse::<usize>().map_err(|_| ReflectError::InvalidIndex)?;

        let val = self.get(index)
            .ok_or(ReflectError::entry_not_exist(index.to_string()))?;

        Ok((val as &dyn Reflect).into())
    }

    fn impl_get_mut<'a>(&mut self, _path: ReflectPath<'a>) -> ReflectResult<'a, &mut dyn Reflect> {
        Err(ReflectError::ImmutableContainer)
    }

    fn impl_insert<'a>(&mut self, _path: ReflectPath<'a>, _value: Box<dyn Reflect>) -> ReflectResult<'a, ()> {
        Err(ReflectError::ImmutableContainer)
    }
    
    fn impl_iter<'a>(&self, mut path: ReflectPath<'a>) -> ReflectResult<'a, ReflectIter<'_>> {
        let Some(index) = path.next() else {
            return Ok(ReflectIter { 
                iter: self.iter().enumerate().map(|(k, v)| {
                    ReflectIterEntry {
                        item: v as &dyn Reflect,
                        index: Some(ReflectItemIndex::Number(k))
                    }
                })
                .collect()
            })
        };

        let index = index.parse::<usize>().map_err(|_| ReflectError::InvalidIndex)?;
        let val = self.get(index)
            .ok_or(ReflectError::entry_not_exist(index.to_string()))?;

        val.impl_iter(path)
    }
    fn impl_iter_mut<'a>(&mut self, _path: ReflectPath<'a>) -> ReflectResult<'a, ReflectIterMut<'_>> {
        Err(ReflectError::ImmutableContainer)
    }

    fn duplicate(&self) -> Option<Box<dyn Reflect>> {
        None
    }

    fn from_string(_: &str) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized {
        Err(ReflectError::NoFromString)
    }
}

macro_rules! impl_reflect_immutable_container {
    ($($ty:ty),*) => { $(
        impl<T:Reflect> Reflect for $ty where Self: std::ops::Deref<Target=T> {

            fn impl_get<'a,'s>(&'s self, path: ReflectPath<'a>) -> ReflectResult<'a, MaybeOwnedReflect<'s>> {
                (&**self).impl_get(path)
            }

            fn impl_get_mut<'a>(&mut self, _path: ReflectPath<'a>) -> ReflectResult<'a, &mut dyn Reflect> {
                Err(ReflectError::ImmutableContainer)
            }

            fn impl_insert<'a>(&mut self, path: ReflectPath<'a>, value: Box<dyn Reflect>) -> ReflectResult<'a, ()> {
                if let Some(value) = value.downcast::<Self>().ok().filter(|_| !path.has_next()) {
                    *self = *value;
                    return Ok(())
                }

                Err(ReflectError::ImmutableContainer)
            }

            fn impl_iter<'a>(&self, path: ReflectPath<'a>) -> ReflectResult<'a, ReflectIter<'_>> {
                (&**self).impl_iter(path)
            }
            fn impl_iter_mut<'a>(&mut self, _path: ReflectPath<'a>) -> ReflectResult<'a, ReflectIterMut<'_>> {
                Err(ReflectError::ImmutableContainer)
            }

            fn duplicate(&self) -> Option<Box<dyn Reflect>> {
                Some(Box::new(self.clone()))
            }

            fn from_string(str: &str) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized {
                Ok(Box::new(T::from_string(str)?))
            }
        }
    )* };
}
impl_reflect_immutable_container!(std::sync::Arc<T>);


macro_rules! impl_reflect_mutable_container {
    ($($ty:ty),*) => { $(
        impl<T:Reflect + ?Sized> Reflect for $ty where Self: std::ops::Deref<Target=T> + std::ops::DerefMut {
            fn type_name(&self) -> &'static str {
                T::type_name(&**self)
            }
            
            fn impl_get<'a, 's>(&'s self, path: ReflectPath<'a>) -> ReflectResult<'a, MaybeOwnedReflect<'s>> {
                (&**self).impl_get(path)
            }

            fn impl_get_mut<'a>(&mut self, path: ReflectPath<'a>) -> ReflectResult<'a, &mut dyn Reflect> {
                (&mut **self).impl_get_mut(path)
            }

            fn impl_insert<'a>(&mut self, path: ReflectPath<'a>, value: Box<dyn Reflect>) -> ReflectResult<'a, ()> {
                if !path.has_next() && value.is::<Self>() {
                    *self = *value.downcast::<Self>().ok().unwrap();
                    return Ok(())
                }

                (&mut **self).impl_insert(path, value)
            }

            fn impl_iter<'a>(&self, path: ReflectPath<'a>) -> ReflectResult<'a, ReflectIter<'_>> {
                (&**self).impl_iter(path)
            }
            fn impl_iter_mut<'a>(&mut self, path: ReflectPath<'a>) -> ReflectResult<'a, ReflectIterMut<'_>> {
                (&mut **self).impl_iter_mut(path)
            }


            fn duplicate(&self) -> Option<Box<dyn Reflect>> {
                T::duplicate(&**self)
            }

            fn from_string(_str: &str) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized {
                Err(ReflectError::NoFromString)
                //Ok(Box::new(T::from_string(str)?))
            }
        }

    )* };
}
impl_reflect_mutable_container!(Box<T>);


macro_rules! impl_reflect_tuple {
    ($($g:ident $ty:tt => $v:literal => $i:tt),+) => {
        impl<$($g: Reflect),+> Reflect for ($($ty),+ ,) {
            fn impl_get<'a, 's>(&'s self, mut path: ReflectPath<'a>) -> ReflectResult<'a, MaybeOwnedReflect<'s>> {
                match path.next() {
                    None => Ok(self.as_dyn().into()),
                    Some(index) => {
                        let index: usize = index.parse()
                            .map_err(|_| ReflectError::InvalidIndex)?;

                        $(
                            if index == $v {
                                return self.$i.as_dyn().impl_get(path);
                            }
                        )+

                        Err(ReflectError::InvalidIndex)
                    }
                }
            }

            fn impl_get_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> ReflectResult<'a, &mut dyn Reflect> {
                match path.next() {
                    None => Ok(self.as_dyn_mut()),
                    Some(index) => {
                        let index: usize = index.parse()
                            .map_err(|_| ReflectError::InvalidIndex)?;

                        $(
                            if index == $v {
                                return self.$i.as_dyn_mut().impl_get_mut(path);
                            }
                        )+

                        Err(ReflectError::InvalidIndex)
                    }
                }
            }

            fn impl_insert<'a>(&mut self, mut path: ReflectPath<'a>, value: Box<dyn Reflect>) -> ReflectResult<'a, ()> {
                match path.next() {
                    None => value.downcast::<Self>()
                        .map(|v| *self = *v)
                        .map_err(|v| ReflectError::wrong_type(type_name::<Self>(), v.type_name())),
                    Some(index) => {
                        let index: usize = index.parse()
                            .map_err(|_| ReflectError::InvalidIndex)?;

                        $(
                            if index == $v {
                                return self.$i.as_dyn_mut().impl_insert(path, value);
                            }
                        )+

                        Err(ReflectError::InvalidIndex)
                    }
                }
            }


            fn impl_iter<'a>(&self, mut path: ReflectPath<'a>) -> ReflectResult<'a, ReflectIter<'_>> {
                match path.next() {
                    None => Ok(ReflectIter { 
                        iter: vec![
                            $( self.$i.as_dyn() ),+
                        ].into_iter().enumerate().map(|(k, v)| {
                            ReflectIterEntry {
                                item: v as &dyn Reflect,
                                index: Some(ReflectItemIndex::Number(k))
                            }
                        })
                        .collect()
                    }),
                    Some(index) => {
                        let index: usize = index.parse()
                            .map_err(|_| ReflectError::InvalidIndex)?;

                        $(
                            if index == $v {
                                return self.$i.as_dyn().impl_iter(path);
                            }
                        )+

                        Err(ReflectError::InvalidIndex)
                    }
                }
            }
            fn impl_iter_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> ReflectResult<'a, ReflectIterMut<'_>> {
                match path.next() {
                    None => Ok(ReflectIterMut { 
                        iter: vec![
                            $( self.$i.as_dyn_mut() ),+
                        ].into_iter().enumerate().map(|(k, v)| {
                            ReflectIterMutEntry {
                                item: v as &mut dyn Reflect,
                                index: Some(ReflectItemIndex::Number(k))
                            }
                        })
                        .collect()
                    }),
                    Some(index) => {
                        let index: usize = index.parse()
                            .map_err(|_| ReflectError::InvalidIndex)?;

                        $(
                            if index == $v {
                                return self.$i.as_dyn_mut().impl_iter_mut(path);
                            }
                        )+

                        Err(ReflectError::InvalidIndex)
                    }
                }
            }


            fn duplicate(&self) -> Option<Box<dyn Reflect>> {
                None
            }

            fn from_string(_: &str) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized {
                Err(ReflectError::NoFromString)
            }
        }
    };

    ($($ty:tt => $e:tt),+) => {
        impl_reflect_tuple!($($ty $ty => $e => $e),+);
    }
}

mod tuple_impl {
    use super::*;

    impl_reflect_tuple!(T1 => 0);
    impl_reflect_tuple!(T1 => 0, T2 => 1);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5, T7 => 6);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5, T7 => 6, T8 => 7);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5, T7 => 6, T8 => 7, T9 => 8);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5, T7 => 6, T8 => 7, T9 => 8, T10 => 9);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5, T7 => 6, T8 => 7, T9 => 8, T10 => 9, T11 => 10);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5, T7 => 6, T8 => 7, T9 => 8, T10 => 9, T11 => 10, T12 => 11);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5, T7 => 6, T8 => 7, T9 => 8, T10 => 9, T11 => 10, T12 => 11, T13 => 12);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5, T7 => 6, T8 => 7, T9 => 8, T10 => 9, T11 => 10, T12 => 11, T13 => 12, T14 => 13);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5, T7 => 6, T8 => 7, T9 => 8, T10 => 9, T11 => 10, T12 => 11, T13 => 12, T14 => 13, T15 => 14);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5, T7 => 6, T8 => 7, T9 => 8, T10 => 9, T11 => 10, T12 => 11, T13 => 12, T14 => 13, T15 => 14, T16 => 15);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5, T7 => 6, T8 => 7, T9 => 8, T10 => 9, T11 => 10, T12 => 11, T13 => 12, T14 => 13, T15 => 14, T16 => 15, T17 => 16);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5, T7 => 6, T8 => 7, T9 => 8, T10 => 9, T11 => 10, T12 => 11, T13 => 12, T14 => 13, T15 => 14, T16 => 15, T17 => 16, T18 => 17);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5, T7 => 6, T8 => 7, T9 => 8, T10 => 9, T11 => 10, T12 => 11, T13 => 12, T14 => 13, T15 => 14, T16 => 15, T17 => 16, T18 => 17, T19 => 18);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5, T7 => 6, T8 => 7, T9 => 8, T10 => 9, T11 => 10, T12 => 11, T13 => 12, T14 => 13, T15 => 14, T16 => 15, T17 => 16, T18 => 17, T19 => 18, T20 => 19);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5, T7 => 6, T8 => 7, T9 => 8, T10 => 9, T11 => 10, T12 => 11, T13 => 12, T14 => 13, T15 => 14, T16 => 15, T17 => 16, T18 => 17, T19 => 18, T20 => 19, T21 => 20);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5, T7 => 6, T8 => 7, T9 => 8, T10 => 9, T11 => 10, T12 => 11, T13 => 12, T14 => 13, T15 => 14, T16 => 15, T17 => 16, T18 => 17, T19 => 18, T20 => 19, T21 => 20, T22 => 21);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5, T7 => 6, T8 => 7, T9 => 8, T10 => 9, T11 => 10, T12 => 11, T13 => 12, T14 => 13, T15 => 14, T16 => 15, T17 => 16, T18 => 17, T19 => 18, T20 => 19, T21 => 20, T22 => 21, T23 => 22);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5, T7 => 6, T8 => 7, T9 => 8, T10 => 9, T11 => 10, T12 => 11, T13 => 12, T14 => 13, T15 => 14, T16 => 15, T17 => 16, T18 => 17, T19 => 18, T20 => 19, T21 => 20, T22 => 21, T23 => 22, T24 => 23);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5, T7 => 6, T8 => 7, T9 => 8, T10 => 9, T11 => 10, T12 => 11, T13 => 12, T14 => 13, T15 => 14, T16 => 15, T17 => 16, T18 => 17, T19 => 18, T20 => 19, T21 => 20, T22 => 21, T23 => 22, T24 => 23, T25 => 24);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5, T7 => 6, T8 => 7, T9 => 8, T10 => 9, T11 => 10, T12 => 11, T13 => 12, T14 => 13, T15 => 14, T16 => 15, T17 => 16, T18 => 17, T19 => 18, T20 => 19, T21 => 20, T22 => 21, T23 => 22, T24 => 23, T25 => 24, T26 => 25);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5, T7 => 6, T8 => 7, T9 => 8, T10 => 9, T11 => 10, T12 => 11, T13 => 12, T14 => 13, T15 => 14, T16 => 15, T17 => 16, T18 => 17, T19 => 18, T20 => 19, T21 => 20, T22 => 21, T23 => 22, T24 => 23, T25 => 24, T26 => 25, T27 => 26);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5, T7 => 6, T8 => 7, T9 => 8, T10 => 9, T11 => 10, T12 => 11, T13 => 12, T14 => 13, T15 => 14, T16 => 15, T17 => 16, T18 => 17, T19 => 18, T20 => 19, T21 => 20, T22 => 21, T23 => 22, T24 => 23, T25 => 24, T26 => 25, T27 => 26, T28 => 27);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5, T7 => 6, T8 => 7, T9 => 8, T10 => 9, T11 => 10, T12 => 11, T13 => 12, T14 => 13, T15 => 14, T16 => 15, T17 => 16, T18 => 17, T19 => 18, T20 => 19, T21 => 20, T22 => 21, T23 => 22, T24 => 23, T25 => 24, T26 => 25, T27 => 26, T28 => 27, T29 => 28);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5, T7 => 6, T8 => 7, T9 => 8, T10 => 9, T11 => 10, T12 => 11, T13 => 12, T14 => 13, T15 => 14, T16 => 15, T17 => 16, T18 => 17, T19 => 18, T20 => 19, T21 => 20, T22 => 21, T23 => 22, T24 => 23, T25 => 24, T26 => 25, T27 => 26, T28 => 27, T29 => 28, T30 => 29);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5, T7 => 6, T8 => 7, T9 => 8, T10 => 9, T11 => 10, T12 => 11, T13 => 12, T14 => 13, T15 => 14, T16 => 15, T17 => 16, T18 => 17, T19 => 18, T20 => 19, T21 => 20, T22 => 21, T23 => 22, T24 => 23, T25 => 24, T26 => 25, T27 => 26, T28 => 27, T29 => 28, T30 => 29, T31 => 30);
    impl_reflect_tuple!(T1 => 0, T2 => 1, T3 => 2, T4 => 3, T5 => 4, T6 => 5, T7 => 6, T8 => 7, T9 => 8, T10 => 9, T11 => 10, T12 => 11, T13 => 12, T14 => 13, T15 => 14, T16 => 15, T17 => 16, T18 => 17, T19 => 18, T20 => 19, T21 => 20, T22 => 21, T23 => 22, T24 => 23, T25 => 24, T26 => 25, T27 => 26, T28 => 27, T29 => 28, T30 => 29, T31 => 30, T32 => 31);
}


impl std::fmt::Debug for dyn Reflect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        macro_rules! pain {
            ($($ty:ty),*) => { $(
                if let Some(a) = self.downcast_ref::<$ty>() {
                    return write!(f, "{a:?}");
                }
            )*}
        }

        pain!(
            u8, i8,
            u16, i16,
            u32, i32,
            u64, i64,
            // u128, i128,
            usize, isize,
            f32, f64,
            bool,
            String, &'static str
        );

        write!(f, "Other ({})", self.type_name())
    }
}




pub struct ReflectMultiparse<Out>(Box<dyn Reflect>, std::marker::PhantomData<Out>);
impl<Out> ReflectMultiparse<Out> {
    pub fn try_downcast<T: Reflect + Into<Out>>(self) -> Result<Self, Out> {
        match self.0.downcast::<T>() {
            Ok(t) => Err((*t).into()),
            Err(e) => Ok(Self(e, self.1))
        }
    }

    pub fn parse(
        value: Box<dyn Reflect>, 
        checker: fn(Self) -> Result<Self, Out>
    ) -> Result<Out, Box<dyn Reflect>> {
        let v = Self(value, std::marker::PhantomData);
        match checker(v) {
            Ok(failed) => Err(failed.0),
            Err(success) => Ok(success)
        }
    }

    pub fn parse_reflect_err(
        value: Box<dyn Reflect>, 
        checker: fn(Self) -> Result<Self, Out>
    ) -> ReflectResult<'static, Out> {
        Self::parse(value, checker)
            .map_err(|v| ReflectError::wrong_type(
            type_name::<Out>(), 
            v.type_name()
        ))
    }
}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_str() {
        #[derive(Reflect)]
        #[derive(Clone)]
        struct A {
            a: &'static str,
        }

        let a = A {
            a: "hi mom",
        };

        let s = a
            .as_dyn()
            .reflect_display("a", None)
            .unwrap();

        assert_eq!(&s, a.a);
    }


    #[derive(Debug, Reflect, Clone)]
    struct A {
        #[reflect(alias("hello"))]
        hi: String,
        #[reflect(rename = "bb")]
        b: f32,
        b2: B,
        #[reflect(skip)]
        _skip: bool,
    }

    #[derive(Clone, Debug, PartialEq)]
    #[derive(Reflect)]
    struct B {
        q: u64
    }

    #[derive(Reflect, Debug, PartialEq, Clone)]
    #[reflect(skip)]
    struct SkipAll {
        a: u32,
        b: bool,
    }

    #[allow(unused)]
    #[derive(Clone, Debug, PartialEq)]
    #[derive(Reflect)]
    #[repr(i32)]
    enum TestEnum {
        Unit,
        #[reflect(rename = "value")]
        Value = 10,
        Tuple(String),
        #[reflect(alias("hello"))]
        Struct {
            hi: String
        },
        #[reflect(skip)]
        Skip,
    }



    // todo: skip all enum

    #[test]
    fn test() {
        let mut a = A {
            hi: "hi mom".to_owned(),
            b: 4.5,
            b2: B {
                q: 77
            },
            _skip: true,
        };

        let skip_all = SkipAll {
            a: 4,
            b: false,
        };

        assert_eq!(a.as_dyn().reflect_get("hi"), Ok(MaybeOwned::Borrowed(&a.hi)));
        assert_eq!(a.as_dyn().reflect_get("hello"), Ok(MaybeOwned::Borrowed(&a.hi)));
        assert!(a.as_dyn().reflect_get::<f32>("b").is_err());
        assert_eq!(a.as_dyn().reflect_get("bb"), Ok(MaybeOwned::Borrowed(&a.b)));
        assert_eq!(a.as_dyn().reflect_get("b2.q"), Ok(MaybeOwned::Borrowed(&a.b2.q)));
        assert!(a.as_dyn().reflect_get::<bool>("_skip").is_err());

        assert_eq!(a.as_dyn_mut().reflect_insert("hi", "awawa".to_owned()), Ok(()));
        assert_eq!(a.as_dyn_mut().reflect_insert("bb", 5.9f32), Ok(()));
        assert_eq!(a.as_dyn_mut().reflect_insert("b2.q", 33u64), Ok(()));

        assert_eq!(a.as_dyn().reflect_get("hi"), Ok(MaybeOwned::Borrowed(&"awawa".to_owned())));
        assert_eq!(a.as_dyn().reflect_get("bb"), Ok(MaybeOwned::Borrowed(&5.9f32)));
        assert_eq!(a.as_dyn().reflect_get("b2.q"), Ok(MaybeOwned::Borrowed(&33u64)));

        let mut iter = a.as_dyn().reflect_iter("").unwrap();
        assert_eq!(iter.next().unwrap().reflect_get(""), Ok(MaybeOwned::Borrowed(&a.hi)));
        assert_eq!(iter.next().unwrap().reflect_get(""), Ok(MaybeOwned::Borrowed(&a.b)));
        assert_eq!(iter.next().unwrap().reflect_get(""), Ok(MaybeOwned::Borrowed(&a.b2)));
        assert!(iter.next().is_none());

        let mut iter = a.as_dyn().reflect_iter("b2").unwrap();
        assert_eq!(iter.next().unwrap().reflect_get(""), Ok(MaybeOwned::Borrowed(&a.b2.q)));
        assert!(iter.next().is_none());

        assert_eq!(skip_all.as_dyn().reflect_get(""), Ok(MaybeOwned::Borrowed(&skip_all)));
        assert!(skip_all.as_dyn().reflect_get::<u32>("a").is_err());
        assert!(skip_all.as_dyn().reflect_get::<bool>("b").is_err());

        let mut iter = skip_all.as_dyn().reflect_iter("").unwrap();
        assert_eq!(iter.next().unwrap().reflect_get(""), Ok(MaybeOwned::Borrowed(&skip_all)));
        assert!(iter.next().is_none());

        let e = TestEnum::Tuple("123".to_owned());
        assert_eq!(e.as_dyn().reflect_get("Tuple"), Ok(MaybeOwned::Borrowed(&e)));
        assert_eq!(e.as_dyn().reflect_get("Tuple.0"), Ok(MaybeOwned::Borrowed(&"123".to_owned())));
        assert!(e.as_dyn().reflect_get::<TestEnum>("Unit").is_err());

        // todo: enum tests
    }




}

