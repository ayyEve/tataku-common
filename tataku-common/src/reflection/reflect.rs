use crate::prelude::*;
use std::any::type_name;

// pub trait Any: std::any::Any {
//     fn type_name(&self) -> &'static str { type_name::<Self>() }
//     fn impl_as_any(&self) -> &dyn std::any::Any where Self:Sized { self }
// }
// impl dyn Any {
//     fn as_any(&self) -> &dyn std::any::Any {
//         self.impl_as_any()
//     }
// }
#[allow(unused)]
macro_rules! reflect_impl {
    (Sized) => { reflect_impl!(@impl where Self: Sized); };
    () => { reflect_impl!(@impl); };
    (@impl $($where:tt)*) => {
        #[allow(unused)]
        fn reflect_get<'a, T: Reflect + 'static>(&self, path: impl Into<ReflectPath<'a>>) -> Result<&T, ReflectError<'a>> $($where)* {
            self.impl_get(path.into())?
                .downcast_ref::<T>()
                .ok_or(ReflectError::wrong_type("TODO: cry", type_name::<T>()))
        }
        #[allow(unused)]
        fn reflect_get_mut<'a, T: Reflect + 'static>(&mut self, path: impl Into<ReflectPath<'a>>) -> Result<&mut T, ReflectError<'a>> $($where)* {
            self.impl_get_mut(path.into())?
                .downcast_mut::<T>()
                .ok_or(ReflectError::wrong_type("TODO: cry", type_name::<T>()))
        }

        #[allow(unused)]
        fn reflect_insert<'a, T: Reflect + 'static>(&mut self, path: impl Into<ReflectPath<'a>>, value: T) -> Result<(), ReflectError<'a>> $($where)* {
            self.impl_insert(path.into(), Box::new(value))
        }

        #[allow(unused)]
        fn reflect_iter<'a>(&self, path: impl Into<ReflectPath<'a>>) -> Result<IterThing<'_>, ReflectError<'a>> $($where)* {
            self.impl_iter(path.into())
        }
        #[allow(unused)]
        fn reflect_iter_mut<'a>(&mut self, path: impl Into<ReflectPath<'a>>) -> Result<IterThingMut<'_>, ReflectError<'a>> $($where)* {
            self.impl_iter_mut(path.into())
        }
    };
}

/// value-able, as in able-to-valuez
/// not valuable, as in has-value
pub trait Reflect: downcast_rs::DowncastSync + std::fmt::Debug {
    reflect_impl!(Sized);

    fn impl_get<'s, 'v>(&'s self, path: ReflectPath<'v>) -> Result<&'s dyn Reflect, ReflectError<'v>>;
    fn impl_get_mut<'s, 'v>(&'s mut self, path: ReflectPath<'v>) -> Result<&'s mut dyn Reflect, ReflectError<'v>>;

    fn impl_insert<'s, 'v>(&'s mut self, path: ReflectPath<'v>, value: Box<dyn Reflect>) -> Result<(), ReflectError<'v>>;

    fn impl_iter<'s, 'v>(&'s self, _path: ReflectPath<'v>) -> Result<IterThing<'s>, ReflectError<'v>> {
        Ok(Default::default())
    }
    fn impl_iter_mut<'s, 'v>(&'s mut self, _path: ReflectPath<'v>) -> Result<IterThingMut<'s>, ReflectError<'v>> {
        Ok(Default::default())
    }
}

impl dyn Reflect { reflect_impl!(); }
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
                    .map_err(|_e| ReflectError::wrong_type(type_name::<$ty>(), "TODO: cry"))
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
    f32, f64,
    bool,
    String, &'static str
);

// base_valueable_impl!(impl<T> for <Option<T>> where T: Reflect);


impl<T:Reflect> Reflect for Option<T> {
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

        if value.is::<T>() {
            value
                .downcast::<T>()
                .map(|a| *self = Some(*a))
                .unwrap();

            return Ok(())
        }

        value
            .downcast::<Option<T>>()
            .map(|a| *self = *a)
            .map_err(|_e| ReflectError::wrong_type(type_name::<Self>(), "TODO: cry"))?;
        Ok(())
    }
}


impl<K, V> Reflect for std::collections::HashMap<K, V>
    where
    K: core::str::FromStr + std::string::ToString + core::hash::Hash + core::cmp::Eq + Send + Sync + std::fmt::Debug + 'static,
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
                .map_err(|_e| ReflectError::wrong_type(type_name::<Self>(), "TODO: cry"))
        };
        let key = key.parse::<K>().map_err(|_| ReflectError::InvalidHashmapKey)?;

        let value = value
            .downcast::<V>()
            .map_err(|_e| ReflectError::wrong_type(type_name::<V>(), "TODO: cry"))?;

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

}

impl<T: Reflect> Reflect for Vec<T> {
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
                value
                    .downcast::<T>()
                    .map(|a| self.push(*a))
                    .unwrap()
            } else {
                value
                .downcast::<Self>()
                .map(|a| *self = *a)
                .map_err(|_e| ReflectError::wrong_type(type_name::<Self>(), "TODO: cry"))?;
            }

            return Ok(())
        };
        let index = index.parse::<usize>().map_err(|_| ReflectError::InvalidIndex)?;

        let value = value
            .downcast::<T>()
            .map_err(|_e| ReflectError::wrong_type(type_name::<T>(), "TODO: cry"))?;

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

}




#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, Reflect)]
    struct A {
        hi: String,
        b: f32,
        b2: B
    }
    #[derive(Clone, Debug)]
    #[derive(Reflect)]
    struct B {
        q: u64
    }

    #[derive(Clone, Debug, PartialEq)]
    #[derive(Reflect)]
    #[repr(i32)]
    #[allow(unused)]
    enum _TestEnum {
        Unit,
        Value = 10,
        Tuple(String),
        Struct {
            hi: String
        }
    }

    #[test]
    fn test() {
        let mut a = A {
            hi: "hi mom".to_owned(),
            b: 4.5,
            b2: B {
                q: 77
            }
        };

        assert_eq!(a.reflect_get("hi"), Ok(&a.hi));
        assert_eq!(a.reflect_get("b"), Ok(&a.b));
        assert_eq!(a.reflect_get("b2.q"), Ok(&a.b2.q));

        assert_eq!(a.reflect_insert("hi", format!("awawa")), Ok(()));
        assert_eq!(a.reflect_insert("b", 5.9f32), Ok(()));
        assert_eq!(a.reflect_insert("b2.q", 33u64), Ok(()));

        assert_eq!(a.reflect_get("hi"), Ok(&"awawa".to_owned()));
        assert_eq!(a.reflect_get("b"), Ok(&5.9f32));
        assert_eq!(a.reflect_get("b2.q"), Ok(&33u64));

        for i in a.reflect_iter("").unwrap() {
            println!("{i:?}")
        }


        let fuck = _TestEnum::Tuple(format!("gfnkjd;"));
        assert_eq!(fuck.reflect_get("Tuple"), Ok(&fuck));
        assert_eq!(fuck.reflect_get("Tuple.0"), Ok(&format!("gfnkjd;")));
    }
}
