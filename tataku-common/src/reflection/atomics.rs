use crate::prelude::*;
use std::any::type_name;
use std::sync::atomic::*;

const LOAD_ORDER: Ordering = Ordering::Acquire;
const STORE_ORDER: Ordering = Ordering::Release;

macro_rules! impl_atomic_number {
    ($(($atomic: ty, $ty: ty)), *$(,)?) => {
        $(
            impl Reflect for $atomic {
                fn impl_get<'s, 'v>(
                    &'s self, 
                    mut path: ReflectPath<'v>
                ) -> ReflectResult<'v, MaybeOwnedReflect<'s>> {
                    if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }
                    Ok(MaybeOwnedReflect::Owned(Box::new(self.load(LOAD_ORDER))))
                }

                fn impl_get_mut<'s, 'v>(
                    &'s mut self, 
                    mut path: ReflectPath<'v>
                ) -> ReflectResult<'v, &'s mut dyn Reflect> {
                    if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }
                    Ok(self.get_mut())
                }

                fn impl_insert<'v>(
                    &mut self, 
                    mut path: ReflectPath<'v>, 
                    value: Box<dyn Reflect>
                ) -> ReflectResult<'v, ()> {
                    if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }
                    if let Ok(num) = value.impl_as_number(ReflectPath::new("")) {
                        self.store(num.into(), STORE_ORDER);
                        Ok(())
                    } else {
                        Err(ReflectError::wrong_type(type_name::<Self>(), value.type_name()))
                    }
                }

                fn duplicate(&self) -> Option<Box<dyn Reflect>> {
                    Some(Box::new(Self::new(self.load(LOAD_ORDER))))
                }

                fn impl_as_number<'v>(
                    &self, 
                    mut path: ReflectPath<'v>
                ) -> ReflectResult<'v, ReflectNumber> {
                    if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }
                    Ok(self.load(LOAD_ORDER).into())   
                }

                fn impl_display<'v>(
                    &self, 
                    mut path: ReflectPath<'v>, 
                    precision: Option<usize>
                ) -> ReflectResult<'v, String> {
                    if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }
                    let value = self.load(LOAD_ORDER);
                    if let Some(precis) = precision {
                        Ok(format!("{value:.*}", precis))
                    } else {
                        Ok(value.to_string())
                    }
                }
                
                fn from_string(str: &str) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized {
                    let num = str.parse::<$ty>()
                        .map_err(|_| ReflectError::NotANumber)?;
                    Ok(Box::new(Self::new(num)))
                }
            }
        )*
    };
}

impl_atomic_number![
    (AtomicU8, u8),
    (AtomicI8, i8),
    
    (AtomicU16, u16),
    (AtomicI16, i16),

    (AtomicU32, u32),
    (AtomicI32, i32),

    (AtomicU64, u64),
    (AtomicI64, i64),

    (AtomicUsize, usize),
    (AtomicIsize, isize),
];

impl Reflect for AtomicBool {
    fn impl_get<'s, 'v>(
        &'s self, 
        mut path: ReflectPath<'v>
    ) -> ReflectResult<'v, MaybeOwnedReflect<'s>> {
        if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }
        Ok(MaybeOwnedReflect::Owned(Box::new(self.load(LOAD_ORDER))))
    }

    fn impl_get_mut<'s, 'v>(
        &'s mut self, 
        mut path: ReflectPath<'v>
    ) -> ReflectResult<'v, &'s mut dyn Reflect> {
        if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }
        Ok(self.get_mut())
    }

    fn impl_insert<'v>(
        &mut self, 
        mut path: ReflectPath<'v>, 
        value: Box<dyn Reflect>
    ) -> ReflectResult<'v, ()> {
        if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }

        let value = ReflectMultiparse::<AtomicBool>::parse_reflect_err(value, |v| 
            v
                .try_downcast::<bool>()?
                .try_downcast::<AtomicBool>()
        )?;

        self.store(value.load(LOAD_ORDER), STORE_ORDER);
        Ok(())
    }

    fn duplicate(&self) -> Option<Box<dyn Reflect>> {
        Some(Box::new(Self::new(self.load(LOAD_ORDER))))
    }

    fn impl_as_number<'v>(
        &self, 
        mut path: ReflectPath<'v>
    ) -> ReflectResult<'v, ReflectNumber> {
        if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }
        Ok(ReflectNumber::U8(self.load(LOAD_ORDER).into()))   
    }

    fn impl_display<'v>(
        &self, 
        mut path: ReflectPath<'v>, 
        precision: Option<usize>
    ) -> ReflectResult<'v, String> {
        if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }
        let value = self.load(LOAD_ORDER);
        if let Some(precis) = precision {
            Ok(format!("{value:.*}", precis))
        } else {
            Ok(value.to_string())
        }
    }
    
    fn from_string(str: &str) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized {
        let val = str.parse::<bool>()
            .map_err(|_| ReflectError::wrong_type("bool::from(String)", str))?;
        Ok(Box::new(Self::new(val)))
    }
}
