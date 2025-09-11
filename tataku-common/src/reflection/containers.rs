use crate::reflection::*;

macro_rules! impl_reflect_immutable_container {
    ($($ty:ty),*) => { $(
        impl<T:Reflect> Reflect for $ty where Self: std::ops::Deref<Target=T> {
            fn impl_get<'a,'s>(&'s self, path: ReflectPath<'a>) -> reflect::Result<'a, MaybeOwnedReflect<'s>> {
                (&**self).impl_get(path)
            }

            fn impl_get_mut<'a>(&mut self, _path: ReflectPath<'a>) -> reflect::Result<'a, &mut dyn Reflect> {
                Err(ReflectError::ImmutableContainer)
            }

            fn impl_insert<'a>(&mut self, path: ReflectPath<'a>, value: Box<dyn Reflect>) -> reflect::Result<'a, ()> {
                if let Some(value) = value.downcast::<Self>().ok().filter(|_| !path.has_next()) {
                    *self = *value;
                    return Ok(())
                }

                Err(ReflectError::ImmutableContainer)
            }

            fn impl_iter<'a>(&self, path: ReflectPath<'a>) -> reflect::Result<'a, ReflectIter<'_>> {
                (&**self).impl_iter(path)
            }
            fn impl_iter_mut<'a>(&mut self, _path: ReflectPath<'a>) -> reflect::Result<'a, ReflectIterMut<'_>> {
                Err(ReflectError::ImmutableContainer)
            }

            fn duplicate(&self) -> Option<Box<dyn Reflect>> {
                Some(Box::new(self.clone()))
            }

            fn impl_display<'a>(&self, path: ReflectPath<'a>, precision: Option<usize>) -> reflect::Result<'a, String> {
                (&**self).impl_display(path, precision)
            }
            fn impl_as_number<'a>(&self, path: ReflectPath<'a>) -> reflect::Result<'a, ReflectNumber> {
                (&**self).impl_as_number(path)
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
            
            fn impl_get<'a, 's>(&'s self, path: ReflectPath<'a>) -> reflect::Result<'a, MaybeOwnedReflect<'s>> {
                (&**self).impl_get(path)
            }

            fn impl_get_mut<'a>(&mut self, path: ReflectPath<'a>) -> reflect::Result<'a, &mut dyn Reflect> {
                (&mut **self).impl_get_mut(path)
            }

            fn impl_insert<'a>(&mut self, path: ReflectPath<'a>, value: Box<dyn Reflect>) -> reflect::Result<'a, ()> {
                if !path.has_next() && value.is::<Self>() {
                    *self = *value.downcast::<Self>().ok().unwrap();
                    return Ok(())
                }

                (&mut **self).impl_insert(path, value)
            }

            fn impl_iter<'a>(&self, path: ReflectPath<'a>) -> reflect::Result<'a, ReflectIter<'_>> {
                (&**self).impl_iter(path)
            }
            fn impl_iter_mut<'a>(&mut self, path: ReflectPath<'a>) -> reflect::Result<'a, ReflectIterMut<'_>> {
                (&mut **self).impl_iter_mut(path)
            }

            fn impl_display<'a>(&self, path: ReflectPath<'a>, precision: Option<usize>) -> reflect::Result<'a, String> {
                (&**self).impl_display(path, precision)
            }
            fn impl_as_number<'a>(&self, path: ReflectPath<'a>) -> reflect::Result<'a, ReflectNumber> {
                (&**self).impl_as_number(path)
            }


            fn duplicate(&self) -> Option<Box<dyn Reflect>> {
                T::duplicate(&**self)
            }
        }

    )* };
}
impl_reflect_mutable_container!(Box<T>);
