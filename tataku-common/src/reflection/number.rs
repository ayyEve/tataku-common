use crate::prelude::*;
use std::any::type_name;
use half::{ f16, bf16 };

use num_traits::AsPrimitive;
// use num_traits::ToPrimitive;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ReflectNumber {
    F16(f16), BF16(bf16),
    U8(u8), I8(i8),
    U16(u16), I16(i16),
    U32(u32), I32(i32),
    U64(u64), I64(i64),
    U128(u128), I128(i128),
    Usize(usize), Isize(isize),
    F32(f32), F64(f64),
}
macro_rules! number_from {
    ($($t:ty, $a: ident),*$(,)?) => {
        $(
        impl From<$t> for ReflectNumber {
            fn from(n: $t) -> Self {
                Self::$a(n)
            }
        }
        impl From<ReflectNumber> for $t {
            fn from(n: ReflectNumber) -> Self {
                match n {
                    ReflectNumber::U8(n) => n.as_(), 
                    ReflectNumber::I8(n) => n.as_(),
                    ReflectNumber::U16(n) => n.as_(), 
                    ReflectNumber::I16(n) => n.as_(),
                    ReflectNumber::U32(n) => n.as_(), 
                    ReflectNumber::I32(n) => n.as_(),
                    ReflectNumber::U64(n) => n.as_(), 
                    ReflectNumber::I64(n) => n.as_(),
                    ReflectNumber::U128(n) => (n as f64).as_(), 
                    ReflectNumber::I128(n) => (n as f64).as_(),
                    ReflectNumber::Usize(n) => n.as_(), 
                    ReflectNumber::Isize(n) => n.as_(),
                    ReflectNumber::F16(n)  => n.to_f32().as_(), 
                    ReflectNumber::BF16(n) => n.to_f32().as_(), 
                    ReflectNumber::F32(n) => n.as_(), 
                    ReflectNumber::F64(n) => n.as_(),
                }
            }
        }
        )*

        impl std::str::FromStr for ReflectNumber {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                $(
                    if let Ok(num) = s.parse::<$t>() {
                        return Ok(num.into());
                    }
                )*
                
                Err(format!("failed to parse '{s}' as a number"))
            }
        }

        impl std::fmt::Display for ReflectNumber {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $( Self::$a(n) => n.fmt(f), )*
                }
            }
        }

    }
}
number_from!(
    u8, U8,
    i8, I8,
    u16, U16,
    i16, I16,
    u32, U32,
    i32, I32,
    u64, U64,
    i64, I64,
    u128, U128,
    i128, I128,
    usize, Usize,
    isize, Isize,
    f32, F32,
    f64, F64,
    f16, F16,
    bf16, BF16,
);

impl Reflect for ReflectNumber {
    fn impl_get<'s, 'v>(&'s self, mut path: ReflectPath<'v>) -> ReflectResult<'v, MaybeOwnedReflect<'s>> {
        if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }
        Ok(MaybeOwnedReflect::Owned(Box::new(*self)))
    }

    fn impl_get_mut<'s, 'v>(&'s mut self, mut path: ReflectPath<'v>) -> ReflectResult<'v, &'s mut dyn Reflect> {
        if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }
        Ok(self)
    }

    fn impl_insert<'v>(&mut self, mut path: ReflectPath<'v>, value: Box<dyn Reflect>) -> ReflectResult<'v, ()> {
        if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }
        if let Ok(num) = value.impl_as_number(ReflectPath::new("")) {
            *self = num;
            Ok(())
        } else {
            Err(ReflectError::wrong_type(type_name::<Self>(), value.type_name()))
        }
    }

    fn duplicate(&self) -> Option<Box<dyn Reflect>> {
        Some(Box::new(*self))
    }

    fn impl_as_number<'v>(&self, mut path: ReflectPath<'v>) -> ReflectResult<'v, ReflectNumber> {
        if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }
        Ok(*self)   
    }

    fn impl_display<'v>(&self, mut path: ReflectPath<'v>, precision: Option<usize>) -> ReflectResult<'v, String> {
        if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }
        if let Some(precis) = precision {
            Ok(format!("{self:.*}", precis))
        } else {
            Ok(format!("{self}"))
        }
    }
    
    fn from_string(str: &str) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized {
        let num = str.parse::<Self>()
            .map_err(|_| ReflectError::NotANumber)?;
        Ok(Box::new(num))
    }
}


#[macro_export]
macro_rules! number_reflect_impl {
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

                if let Some(num) = value.downcast_ref::<ReflectNumber>() {
                    *self = (*num).into();
                    return Ok(());
                }

                value
                    .downcast::<$ty>()
                    .map(|a| *self = *a)
                    .map_err(|v| ReflectError::wrong_type(type_name::<$ty>(), v.type_name()))
            }
            
            fn impl_as_number<'v>(&self, mut path: ReflectPath<'v>) -> ReflectResult<'v, ReflectNumber> {
                if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }
                Ok(ReflectNumber::from(*self))
            }

            fn impl_display<'v>(&self, mut path: ReflectPath<'v>, precision: Option<usize>) -> ReflectResult<'v, String> {
                if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }
                
                if let Some(precis) = precision {
                    Ok(format!("{self:.*}", precis))
                } else {
                    Ok(format!("{self}"))
                }
            }

            fn duplicate(&self) -> Option<Box<dyn Reflect>> {
                Some(Box::new(*self))
            }

            fn from_string(str: &str) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized {
                Ok(Box::new(
                    str.parse::<$ty>().map_err(|_| ReflectError::wrong_type(stringify!($ty), "String"))?
                ))
            }
        }
    };

    ($($ty:ty),*) => { $( number_reflect_impl!(impl<> for $ty where ); )* };
}

number_reflect_impl!(
    u8,
    i8,
    u16,
    i16,
    u32,
    i32,
    u64,
    i64,
    u128,
    i128,
    usize,
    isize,
    f16,
    bf16,
    f32,
    f64
);
