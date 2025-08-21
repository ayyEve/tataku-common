use crate::prelude::*;
use std::any::type_name;


/// helper to try to downcast multiple types to a single type
pub struct ReflectMultiparse<Out>(Box<dyn Reflect>, std::marker::PhantomData<Out>);
impl<Out> ReflectMultiparse<Out> {
    pub fn try_downcast_deref<T>(self) -> Result<Self, Out> 
    where 
        T: Reflect + std::ops::Deref,
        for<'a> &'a <T as std::ops::Deref>::Target: Into<Out>
    {
        match self.0.downcast::<T>() {
            Ok(t) => Err((&**t).into()),
            Err(e) => Ok(Self(e, self.1))
        }
    }
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


/// helper to try to downcast multiple types to a single type when the input is not owned
pub struct ReflectMultiparseRef<'r, Out>(&'r dyn Reflect, std::marker::PhantomData<Out>);
impl<'r, Out> ReflectMultiparseRef<'r, Out> {
    pub fn try_downcast_deref<T>(self) -> Result<Self, Out> 
    where 
        T: Reflect + std::ops::Deref,
        &'r <T as std::ops::Deref>::Target: Into<Out>
    {
        match self.0.downcast_ref::<T>() {
            Some(t) => Err((&**t).into()),
            None => Ok(self)
        }
    }
    
    pub fn try_downcast<T>(self) -> Result<Self, Out> 
        where T: Reflect + Into<Out>,
        &'r T: Into<Out>
    {
        match self.0.downcast_ref::<T>() {
            Some(t) => Err(t.into()),
            None => Ok(self)
        }
    }

    pub fn parse(
        value: &'r dyn Reflect, 
        checker: fn(Self) -> Result<Self, Out>
    ) -> Result<Out, &'r dyn Reflect> {
        let v = Self(value, std::marker::PhantomData);
        match checker(v) {
            Ok(failed) => Err(failed.0),
            Err(success) => Ok(success)
        }
    }

    pub fn parse_reflect_err(
        value: &'r dyn Reflect, 
        checker: fn(Self) -> Result<Self, Out>
    ) -> ReflectResult<'static, Out> {
        Self::parse(value, checker).map_err(|v| 
            ReflectError::wrong_type(
                type_name::<Out>(), 
                v.type_name()
            )
        )
    }
}

#[test]
fn test() {
    let test = String::from("hi mom");

    let a = ReflectMultiparseRef::<String>::parse(&test, |t| t
            .try_downcast::<String>()?
            .try_downcast_deref::<&str>()?
            .try_downcast_deref::<std::sync::Arc<str>>()?
            .try_downcast_deref::<Box<str>>()
    ).unwrap();

    assert_eq!(a, test)
}
