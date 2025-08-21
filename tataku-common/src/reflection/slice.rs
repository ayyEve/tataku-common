use crate::prelude::*;
use std::any::type_name;

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

}

