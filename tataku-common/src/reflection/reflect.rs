use crate::prelude::*;
use std::any::type_name;

pub type ReflectResult<'a, T> = Result<T, ReflectError<'a>>;

/// value-able, as in able-to-valuez
/// not valuable, as in has-value
pub trait Reflect: downcast_rs::DowncastSync {
    fn type_name(&self) -> &'static str {
        type_name::<Self>()
    }

    fn as_dyn(&self) -> &dyn Reflect where Self: Sized { self }
    fn as_dyn_mut(&mut self) -> &mut dyn Reflect where Self: Sized { self }

    fn impl_get<'s, 'v>(&'s self, path: ReflectPath<'v>) -> ReflectResult<'v, &'s dyn Reflect>;
    fn impl_get_mut<'s, 'v>(&'s mut self, path: ReflectPath<'v>) -> ReflectResult<'v, &'s mut dyn Reflect>;

    fn impl_insert<'v>(&mut self, path: ReflectPath<'v>, value: Box<dyn Reflect>) -> ReflectResult<'v, ()>;

    fn impl_iter<'s, 'v>(&'s self, _path: ReflectPath<'v>) -> ReflectResult<'v, IterThing<'s>> {
        Ok(Default::default())
    }
    fn impl_iter_mut<'s, 'v>(&'s mut self, _path: ReflectPath<'v>) -> ReflectResult<'v, IterThingMut<'s>> {
        Ok(Default::default())
    }

    fn duplicate(&self) -> Option<Box<dyn Reflect>>;

    fn from_string(str: &str) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized;
}

impl dyn Reflect {
    pub fn reflect_get<'a, T: Reflect + 'static>(&self, path: impl Into<ReflectPath<'a>>) -> Result<&T, ReflectError<'a>> {
        self.impl_get(path.into())?
            .downcast_ref::<T>()
            .ok_or(ReflectError::wrong_type(self.type_name(), type_name::<T>()))
    }

    pub fn reflect_get_mut<'a, T: Reflect + 'static>(&mut self, path: impl Into<ReflectPath<'a>>) -> Result<&mut T, ReflectError<'a>> {
        let self_type_name = self.type_name();

        self.impl_get_mut(path.into())?
            .downcast_mut::<T>()
            .ok_or(ReflectError::wrong_type(self_type_name, type_name::<T>()))
    }

    pub fn reflect_insert<'a, T: Reflect + 'static>(&mut self, path: impl Into<ReflectPath<'a>>, value: T) -> Result<(), ReflectError<'a>> {
        self.impl_insert(path.into(), Box::new(value))
    }

    pub fn reflect_iter<'a>(&self, path: impl Into<ReflectPath<'a>>) -> Result<IterThing<'_>, ReflectError<'a>> {
        self.impl_iter(path.into())
    }

    pub fn reflect_iter_mut<'a>(&mut self, path: impl Into<ReflectPath<'a>>) -> Result<IterThingMut<'_>, ReflectError<'a>> {
        self.impl_iter_mut(path.into())
    }
}
downcast_rs::impl_downcast!(sync Reflect);


/// currently does not do lazy iteration
#[derive(Default)]
pub struct IterThing<'a> {
    items: std::vec::IntoIter<&'a dyn Reflect>
}

impl<'a> From<Vec<&'a dyn Reflect>> for IterThing<'a> {
    fn from(items: Vec<&'a dyn Reflect>) -> Self {
        Self {
            items: items.into_iter(),
        }
    }
}

impl<'a> Iterator for IterThing<'a> {
    type Item = &'a dyn Reflect;
    fn next(&mut self) -> Option<Self::Item> {
        self.items.next()
    }
}

/// currently does not do lazy iteration
#[derive(Default)]
pub struct IterThingMut<'a>  {
    items: std::vec::IntoIter<&'a mut dyn Reflect>
}

impl<'a> From<Vec<&'a mut dyn Reflect>> for IterThingMut<'a> {
    fn from(items: Vec<&'a mut dyn Reflect>) -> Self {
        Self {
            items: items.into_iter(),
        }
    }
}

impl<'a> Iterator for IterThingMut<'a> {
    type Item = &'a mut dyn Reflect;
    fn next(&mut self) -> Option<Self::Item> {
        self.items.next()
    }
}



// pub struct Value<'a>(&'a dyn std::any::Any);
// impl<'a> Valueable for Value<'a> {
//     fn get<'a, T:'static>(&self, path: impl Into<ValueIdent<'a>>) -> Result<&T, ValueError<'a>> {
//         self.0
//     }
//     fn get_mut<'a, T:'static>(&mut self, path: impl Into<ValueIdent<'a>>) -> Result<&mut T, ValueError<'a>>;

//     fn insert<'a, T:Clone+'static>(&mut self, path: impl Into<ValueIdent<'a>>, value: T) -> Result<(), ValueError<'a>>;

//     fn iter<'a>(&self, _path: impl Into<ValueIdent<'a>>, _f: impl Fn(&dyn Box<Valueable>)) {}
//     fn iter_mut<'a>(&mut self, _path: impl Into<ValueIdent<'a>>, _f: impl Fn(&mut Box<dyn Valueable>)) {}
// }

#[macro_export]
macro_rules! base_valueable_impl {
    (impl<$($g:ty),*> for $ty:ty where $($where:tt)*) => {
        impl<$($g),*> Reflect for $ty where $($where)* {
            fn impl_get<'a>(&self, mut path: ReflectPath<'a>) -> Result<&dyn Reflect, ReflectError<'a>> {
                if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }

                Ok(self as &dyn Reflect)
            }

            fn impl_get_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> Result<&mut dyn Reflect, ReflectError<'a>> {
                if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }

                Ok(self as &mut dyn Reflect)
            }

            fn impl_insert<'a>(&mut self, mut path: ReflectPath<'a>, value: Box<dyn Reflect>) -> Result<(), ReflectError<'a>> {
                if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }

                value
                    .downcast::<$ty>()
                    .map(|a| *self = *a)
                    .map_err(|v| ReflectError::wrong_type(type_name::<$ty>(), v.type_name()))
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

    ($($ty:ty),*) => { $( base_valueable_impl!(impl<> for $ty where ); )* };
}

base_valueable_impl!(
    u8, i8,
    u16, i16,
    u32, i32,
    u64, i64,
    u128, i128,
    usize, isize,
    f32, f64,
    bool,
    String
);
impl Reflect for &'static str {
    fn impl_get<'a>(&self, mut path: ReflectPath<'a>) -> Result<&dyn Reflect, ReflectError<'a>> {
        if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }

        Ok(self as &dyn Reflect)
    }

    fn impl_get_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> Result<&mut dyn Reflect, ReflectError<'a>> {
        if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }

        Ok(self as &mut dyn Reflect)
    }

    fn impl_insert<'a>(&mut self, mut path: ReflectPath<'a>, value: Box<dyn Reflect>) -> Result<(), ReflectError<'a>> {
        if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }

        value
            .downcast::<&'static str>()
            .map(|a| *self = *a)
            .map_err(|v| ReflectError::wrong_type(type_name::<str>(), v.type_name()))
    }

    fn duplicate(&self) -> Option<Box<dyn Reflect>> {
        Some(Box::new(self.to_owned()))
    }

    fn from_string(str: &str) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized {
        Ok(Box::new(str.to_owned()))
    }
}




impl<T:Reflect+Clone> Reflect for Option<T> {
    fn impl_get<'a>(&self, path: ReflectPath<'a>) -> Result<&dyn Reflect, ReflectError<'a>> {
        let next = path.clone().next();
        match (next, self) {
            (None, None) => Err(ReflectError::OptionIsNone),
            (None, Some(s)) => Ok(s as &dyn Reflect),
            (Some("is_some"), None) => Ok(&false as &dyn Reflect),
            (Some("is_some"), Some(_)) => Ok(&true as &dyn Reflect),
            (Some(_), Some(s)) => s.impl_get(path),
            (Some(_), None) => Err(ReflectError::OptionIsNone)
            // Some(next) => 
            // Err(ReflectError::entry_not_exist(next))
        }

        
        // if let Some(next) = path.next() { 
        //     return Err(ReflectError::entry_not_exist(next))
        // }

        // Ok(self as &dyn Reflect)
    }

    fn impl_get_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> Result<&mut dyn Reflect, ReflectError<'a>> {
        if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }

        Ok(self as &mut dyn Reflect)
    }

    fn impl_insert<'a>(&mut self, mut path: ReflectPath<'a>, value: Box<dyn Reflect>) -> Result<(), ReflectError<'a>> {
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



impl<K, V> Reflect for std::collections::HashMap<K, V>
    where
    K: core::str::FromStr + std::string::ToString + core::hash::Hash + core::cmp::Eq + Send + Sync + 'static,
    V: Reflect
{
    fn impl_get<'a>(&self, mut path: ReflectPath<'a>) -> Result<&dyn Reflect, ReflectError<'a>> {
        let Some(key) = path.next() else {
            return Ok(self as &dyn Reflect)
        };

        let key = key.parse::<K>().map_err(|_| ReflectError::InvalidHashmapKey)?;

        let val = self.get(&key)
            .ok_or(ReflectError::entry_not_exist(key.to_string()))?;

        Ok(val as &dyn Reflect)
    }

    fn impl_get_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> Result<&mut dyn Reflect, ReflectError<'a>> {
        let Some(key) = path.next() else {
            return Ok(self as &mut dyn Reflect)
        };
        let key = key.parse::<K>().map_err(|_| ReflectError::InvalidHashmapKey)?;

        let val = self.get_mut(&key)
            .ok_or(ReflectError::entry_not_exist(key.to_string()))?;

        Ok(val as &mut dyn Reflect)
    }

    fn impl_insert<'a>(&mut self, mut path: ReflectPath<'a>, value: Box<dyn Reflect>) -> Result<(), ReflectError<'a>> {
        let Some(key) = path.next() else {
            return value
                .downcast::<Self>()
                .map(|a| *self = *a)
                .map_err(|v| ReflectError::wrong_type(type_name::<Self>(), v.type_name()))
        };
        let key = key.parse::<K>().map_err(|_| ReflectError::InvalidHashmapKey)?;

        let value = value
            .downcast::<V>()
            .map_err(|v| ReflectError::wrong_type(type_name::<V>(), v.type_name()))?;

        self.insert(key, *value);
        Ok(())
    }

    fn impl_iter<'a>(&self, mut path: ReflectPath<'a>) -> Result<IterThing<'_>, ReflectError<'a>> {
        let Some(key) = path.next() else {
            return Ok(self.values()
                    .map(|i| i as &dyn Reflect)
                    .collect::<Vec<_>>()
                    .into()
            )
        };

        let key = key.parse::<K>().map_err(|_| ReflectError::InvalidHashmapKey)?;
        let val = self.get(&key)
            .ok_or(ReflectError::entry_not_exist(key.to_string()))?;

        val.impl_iter(path)
    }
    fn impl_iter_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> Result<IterThingMut<'_>, ReflectError<'a>> {
        let Some(key) = path.next() else {
            return Ok(self.values_mut()
                .map(|i| i as &mut dyn Reflect)
                .collect::<Vec<_>>()
                .into()
        )
        };

        let key = key.parse::<K>().map_err(|_| ReflectError::InvalidHashmapKey)?;
        let val = self.get_mut(&key)
            .ok_or(ReflectError::entry_not_exist(key.to_string()))?;

        val.impl_iter_mut(path)
    }


    fn from_string(_str: &str) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized {
        Err(ReflectError::NoFromString)
    }

    fn duplicate(&self) -> Option<Box<dyn Reflect>> {
        None
    }
}


impl<T> Reflect for std::collections::HashSet<T>
    where
    T: Reflect + core::str::FromStr + std::string::ToString + core::hash::Hash + core::cmp::Eq + 'static,
{
    fn impl_get<'a>(&self, mut path: ReflectPath<'a>) -> Result<&dyn Reflect, ReflectError<'a>> {
        let Some(key) = path.next() else {
            return Ok(self as &dyn Reflect)
        };

        let key = key.parse::<T>().map_err(|_| ReflectError::InvalidHashmapKey)?;

        let val = self.get(&key)
            .ok_or(ReflectError::entry_not_exist(key.to_string()))?;

        Ok(val as &dyn Reflect)
    }

    fn impl_get_mut<'a>(&mut self, path: ReflectPath<'a>) -> Result<&mut dyn Reflect, ReflectError<'a>> {
        if !path.has_next() {
            return Ok(self as &mut dyn Reflect)
        };

        Err(ReflectError::CantMutHashSetKey)
    }

    fn impl_insert<'a>(&mut self, path: ReflectPath<'a>, value: Box<dyn Reflect>) -> Result<(), ReflectError<'a>> {
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



    fn impl_iter<'a>(&self, mut path: ReflectPath<'a>) -> Result<IterThing<'_>, ReflectError<'a>> {
        let Some(key) = path.next() else {
            return Ok(self.iter()
                .map(|i| i as &dyn Reflect)
                .collect::<Vec<_>>()
                .into()
            )
        };

        let key = key.parse::<T>().map_err(|_| ReflectError::InvalidHashmapKey)?;
        let val = self.get(&key)
            .ok_or(ReflectError::entry_not_exist(key.to_string()))?;

        val.impl_iter(path)
    }
    fn impl_iter_mut<'a>(&mut self, _path: ReflectPath<'a>) -> Result<IterThingMut<'_>, ReflectError<'a>> {
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
    fn impl_get<'a>(&self, mut path: ReflectPath<'a>) -> Result<&dyn Reflect, ReflectError<'a>> {
        let Some(index) = path.next() else {
            return Ok(self as &dyn Reflect)
        };

        let index = index.parse::<usize>().map_err(|_| ReflectError::InvalidIndex)?;

        let val = self.get(index)
            .ok_or(ReflectError::entry_not_exist(index.to_string()))?;

        Ok(val as &dyn Reflect)
    }

    fn impl_get_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> Result<&mut dyn Reflect, ReflectError<'a>> {
        let Some(index) = path.next() else {
            return Ok(self as &mut dyn Reflect)
        };
        let index = index.parse::<usize>().map_err(|_| ReflectError::InvalidIndex)?;

        let val = self.get_mut(index)
            .ok_or(ReflectError::entry_not_exist(index.to_string()))?;

        Ok(val as &mut dyn Reflect)
    }

    fn impl_insert<'a>(&mut self, mut path: ReflectPath<'a>, value: Box<dyn Reflect>) -> Result<(), ReflectError<'a>> {
        let Some(index) = path.next() else {

            if value.is::<T>() {
                let _ = value
                    .downcast::<T>()
                    .map(|a| self.push(*a));
            } else {
                value
                    .downcast::<Self>()
                    .map(|a| *self = *a)
                    .map_err(|v| ReflectError::wrong_type(type_name::<Self>(), v.type_name()))?;
            }

            return Ok(())
        };
        let index = index.parse::<usize>().map_err(|_| ReflectError::InvalidIndex)?;

        let value = value
            .downcast::<T>()
            .map_err(|v| ReflectError::wrong_type(type_name::<T>(), v.type_name()))?;

        if index >= self.len() {
            self.push(*value)
        } else {
            self[index] = *value
        }

        Ok(())
    }



    fn impl_iter<'a>(&self, mut path: ReflectPath<'a>) -> Result<IterThing<'_>, ReflectError<'a>> {
        let Some(index) = path.next() else {
            return Ok(self
                .iter()
                .map(|i| i as &dyn Reflect)
                .collect::<Vec<_>>()
                .into()
            )
        };

        let index = index.parse::<usize>().map_err(|_| ReflectError::InvalidIndex)?;
        let val = self.get(index)
            .ok_or(ReflectError::entry_not_exist(index.to_string()))?;

        val.impl_iter(path)
    }
    fn impl_iter_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> Result<IterThingMut<'_>, ReflectError<'a>> {
        let Some(index) = path.next() else {
            return Ok(self
                .iter_mut()
                .map(|i| i as &mut dyn Reflect)
                .collect::<Vec<_>>()
                .into()
            )
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
    fn impl_get<'a>(&self, mut path: ReflectPath<'a>) -> Result<&dyn Reflect, ReflectError<'a>> {
        let Some(index) = path.next() else {
            return Ok(self as &dyn Reflect)
        };

        let index = index.parse::<usize>().map_err(|_| ReflectError::InvalidIndex)?;

        let val = self.get(index)
            .ok_or(ReflectError::entry_not_exist(index.to_string()))?;

        Ok(val as &dyn Reflect)
    }

    fn impl_get_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> Result<&mut dyn Reflect, ReflectError<'a>> {
        let Some(index) = path.next() else {
            return Ok(self as &mut dyn Reflect)
        };
        let index = index.parse::<usize>().map_err(|_| ReflectError::InvalidIndex)?;

        let val = self.get_mut(index)
            .ok_or(ReflectError::entry_not_exist(index.to_string()))?;

        Ok(val as &mut dyn Reflect)
    }

    fn impl_insert<'a>(&mut self, mut path: ReflectPath<'a>, value: Box<dyn Reflect>) -> Result<(), ReflectError<'a>> {
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



    fn impl_iter<'a>(&self, mut path: ReflectPath<'a>) -> Result<IterThing<'_>, ReflectError<'a>> {
        let Some(index) = path.next() else {
            return Ok(self
                .iter()
                .map(|i| i as &dyn Reflect)
                .collect::<Vec<_>>()
                .into()
            )
        };

        let index = index.parse::<usize>().map_err(|_| ReflectError::InvalidIndex)?;
        let val = self.get(index)
            .ok_or(ReflectError::entry_not_exist(index.to_string()))?;

        val.impl_iter(path)
    }
    fn impl_iter_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> Result<IterThingMut<'_>, ReflectError<'a>> {
        let Some(index) = path.next() else {
            return Ok(self
                .iter_mut()
                .map(|i| i as &mut dyn Reflect)
                .collect::<Vec<_>>()
                .into()
            )
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

macro_rules! impl_reflect_immutable_container {
    ($($ty:ty),*) => { $(
        impl<T:Reflect> Reflect for $ty where Self: std::ops::Deref<Target=T> {

            fn impl_get<'a>(&self, path: ReflectPath<'a>) -> Result<&dyn Reflect, ReflectError<'a>> {
                (&**self).impl_get(path)
            }

            fn impl_get_mut<'a>(&mut self, _path: ReflectPath<'a>) -> Result<&mut dyn Reflect, ReflectError<'a>> {
                Err(ReflectError::ImmutableContainer)
            }

            fn impl_insert<'a>(&mut self, path: ReflectPath<'a>, value: Box<dyn Reflect>) -> Result<(), ReflectError<'a>> {
                if let Some(value) = value.downcast::<Self>().ok().filter(|_| !path.has_next()) {
                    *self = *value;
                    return Ok(())
                }

                Err(ReflectError::ImmutableContainer)
            }

            fn impl_iter<'a>(&self, path: ReflectPath<'a>) -> Result<IterThing<'_>, ReflectError<'a>> {
                (&**self).impl_iter(path)
            }
            fn impl_iter_mut<'a>(&mut self, _path: ReflectPath<'a>) -> Result<IterThingMut<'_>, ReflectError<'a>> {
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


            fn impl_get<'a>(&self, path: ReflectPath<'a>) -> Result<&dyn Reflect, ReflectError<'a>> {
                (&**self).impl_get(path)
            }

            fn impl_get_mut<'a>(&mut self, path: ReflectPath<'a>) -> Result<&mut dyn Reflect, ReflectError<'a>> {
                (&mut **self).impl_get_mut(path)
            }

            fn impl_insert<'a>(&mut self, path: ReflectPath<'a>, value: Box<dyn Reflect>) -> Result<(), ReflectError<'a>> {
                if !path.has_next() && value.is::<Self>() {
                    *self = *value.downcast::<Self>().ok().unwrap();
                    return Ok(())
                }

                (&mut **self).impl_insert(path, value)
            }

            fn impl_iter<'a>(&self, path: ReflectPath<'a>) -> Result<IterThing<'_>, ReflectError<'a>> {
                (&**self).impl_iter(path)
            }
            fn impl_iter_mut<'a>(&mut self, path: ReflectPath<'a>) -> Result<IterThingMut<'_>, ReflectError<'a>> {
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
            fn impl_get<'a>(&self, mut path: ReflectPath<'a>) -> Result<&dyn Reflect, ReflectError<'a>> {
                match path.next() {
                    None => Ok(self.as_dyn()),
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

            fn impl_get_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> Result<&mut dyn Reflect, ReflectError<'a>> {
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

            fn impl_insert<'a>(&mut self, mut path: ReflectPath<'a>, value: Box<dyn Reflect>) -> Result<(), ReflectError<'a>> {
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


            fn impl_iter<'a>(&self, mut path: ReflectPath<'a>) -> Result<IterThing<'_>, ReflectError<'a>> {
                match path.next() {
                    None => Ok(vec![
                        $(
                            self.$i.as_dyn()
                        ),+
                    ].into()),
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
            fn impl_iter_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> Result<IterThingMut<'_>, ReflectError<'a>> {
                match path.next() {
                    None => Ok(vec![
                        $(
                            self.$i.as_dyn_mut()
                        ),+
                    ].into()),
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
            u128, i128,
            usize, isize,
            f32, f64,
            bool,
            String, &'static str
        );

        write!(f, "Other ({})", self.type_name())
    }
}



#[cfg(test)]
mod test {
    use super::*;


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

    #[derive(Clone, Debug, PartialEq)]
    #[derive(Reflect)]
    #[repr(i32)]
    #[allow(unused)]
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

        assert_eq!(a.as_dyn().reflect_get("hi"), Ok(&a.hi));
        assert_eq!(a.as_dyn().reflect_get("hello"), Ok(&a.hi));
        assert!(a.as_dyn().reflect_get::<f32>("b").is_err());
        assert_eq!(a.as_dyn().reflect_get("bb"), Ok(&a.b));
        assert_eq!(a.as_dyn().reflect_get("b2.q"), Ok(&a.b2.q));
        assert!(a.as_dyn().reflect_get::<bool>("_skip").is_err());

        assert_eq!(a.as_dyn_mut().reflect_insert("hi", "awawa".to_owned()), Ok(()));
        assert_eq!(a.as_dyn_mut().reflect_insert("bb", 5.9f32), Ok(()));
        assert_eq!(a.as_dyn_mut().reflect_insert("b2.q", 33u64), Ok(()));

        assert_eq!(a.as_dyn().reflect_get("hi"), Ok(&"awawa".to_owned()));
        assert_eq!(a.as_dyn().reflect_get("bb"), Ok(&5.9f32));
        assert_eq!(a.as_dyn().reflect_get("b2.q"), Ok(&33u64));

        let mut iter = a.as_dyn().reflect_iter("").unwrap();
        assert_eq!(iter.next().unwrap().reflect_get(""), Ok(&a.hi));
        assert_eq!(iter.next().unwrap().reflect_get(""), Ok(&a.b));
        assert_eq!(iter.next().unwrap().reflect_get(""), Ok(&a.b2));
        assert!(iter.next().is_none());

        let mut iter = a.as_dyn().reflect_iter("b2").unwrap();
        assert_eq!(iter.next().unwrap().reflect_get(""), Ok(&a.b2.q));
        assert!(iter.next().is_none());

        assert_eq!(skip_all.as_dyn().reflect_get(""), Ok(&skip_all));
        assert!(skip_all.as_dyn().reflect_get::<u32>("a").is_err());
        assert!(skip_all.as_dyn().reflect_get::<bool>("b").is_err());

        let mut iter = skip_all.as_dyn().reflect_iter("").unwrap();
        assert_eq!(iter.next().unwrap().reflect_get(""), Ok(&skip_all));
        assert!(iter.next().is_none());

        let e = TestEnum::Tuple("123".to_owned());
        assert_eq!(e.as_dyn().reflect_get("Tuple"), Ok(&e));
        assert_eq!(e.as_dyn().reflect_get("Tuple.0"), Ok(&"123".to_owned()));
        assert!(e.as_dyn().reflect_get::<TestEnum>("Unit").is_err());

        // todo: enum tests
    }
}



