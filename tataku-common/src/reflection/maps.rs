use crate::prelude::*;
use std::any::type_name;

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

        val.impl_get(path)
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

        val.impl_get_mut(path)
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
}
