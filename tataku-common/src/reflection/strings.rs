use crate::prelude::*;
use std::any::type_name;


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

            // fn from_string(str: &str) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized {
            //     Ok(Box::new(str.to_owned()))
            // }
        }
    )*}
}
immutable_str!(&'static str);


macro_rules! str_container {
    ($($ty: ty),*$(,)?) => { $(
        impl Reflect for $ty {
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
                *self = ReflectMultiparse::<$ty>::parse(value, |v| v
                        .try_downcast_deref::<Arc<str>>()?
                        .try_downcast_deref::<Box<str>>()?
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

            // fn from_string(str: &str) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized {
            //     Ok(Box::new(str.to_owned()))
            // }
        }
    )*}
}

str_container!(
    std::sync::Arc<str>,
    Box<str>,
);
