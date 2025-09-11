use crate::reflection::*;
use std::any::type_name;

pub use super::reflect_error::Result;

pub trait Reflect: downcast_rs::DowncastSync {
    fn type_name(&self) -> &'static str {
        type_name::<Self>()
    }

    fn as_dyn(&self) -> &dyn Reflect where Self: Sized { self }
    fn as_dyn_mut(&mut self) -> &mut dyn Reflect where Self: Sized { self }

    fn impl_get<'s, 'v>(&'s self, path: ReflectPath<'v>) -> Result<'v, MaybeOwnedReflect<'s>>;
    fn impl_get_mut<'s, 'v>(&'s mut self, path: ReflectPath<'v>) -> Result<'v, &'s mut dyn Reflect>;

    fn impl_insert<'v>(&mut self, path: ReflectPath<'v>, value: Box<dyn Reflect>) -> Result<'v, ()>;

    fn impl_iter<'s, 'v>(&'s self, _path: ReflectPath<'v>) -> Result<'v, ReflectIter<'s>> {
        Err(ReflectError::NoIter)
    }
    fn impl_iter_mut<'s, 'v>(&'s mut self, _path: ReflectPath<'v>) -> Result<'v, ReflectIterMut<'s>> {
        Err(ReflectError::NoIter)
    }

    fn impl_display<'v>(&self, path: ReflectPath<'v>, precision: Option<usize>) -> Result<'v, String> {
        if !path.has_next() {
            return Err(ReflectError::NoDisplay);
        }
        match self.impl_get(path)? {
            MaybeOwnedReflect::Borrowed(reflect) => reflect.reflect_display(ReflectPath::EMPTY, precision),
            MaybeOwnedReflect::Owned(reflect) => reflect.reflect_display(ReflectPath::EMPTY, precision),
        }
    }
    
    fn impl_as_number<'v>(&self, path: ReflectPath<'v>) -> Result<'v, ReflectNumber> {
        if !path.has_next() {
            return Err(ReflectError::NotANumber);
        }
        match self.impl_get(path)? {
            MaybeOwnedReflect::Borrowed(reflect) => reflect.reflect_as_number(ReflectPath::EMPTY),
            MaybeOwnedReflect::Owned(reflect) => reflect.reflect_as_number(ReflectPath::EMPTY),
        }
    }
    
    fn duplicate(&self) -> Option<Box<dyn Reflect>>;

}

impl dyn Reflect {
    pub fn reflect_get<'a, 'b, T: Reflect + 'static>(
        &'b self, 
        path: impl Into<ReflectPath<'a>>
    ) -> Result<'a, MaybeOwned<'b, T>> {
        let a = self.impl_get(path.into())?;
        let wrong_type = ReflectError::wrong_type(Reflect::type_name(a.as_ref()), type_name::<T>());
        match a {
            MaybeOwnedReflect::Borrowed(b) => b.downcast_ref::<T>().ok_or(wrong_type).map(MaybeOwned::Borrowed),
            MaybeOwnedReflect::Owned(b) => b.downcast().map_err(|_| wrong_type).map(|i| MaybeOwned::Owned(*i)),
        }
    }

    pub fn reflect_get_mut<'a, T: Reflect + 'static>(
        &mut self, 
        path: impl Into<ReflectPath<'a>>
    ) -> Result<'a, &mut T> {
        let a = self.impl_get_mut(path.into())?;
        let name = a.type_name();
        a.downcast_mut::<T>()
            .ok_or(ReflectError::wrong_type(name, type_name::<T>()))
    }

    pub fn reflect_insert<'a, T: Reflect + 'static>(&mut self, path: impl Into<ReflectPath<'a>>, value: T) -> Result<'a, ()> {
        self.impl_insert(path.into(), Box::new(value))
    }

    pub fn reflect_iter<'a>(&self, path: impl Into<ReflectPath<'a>>) -> Result<'a, ReflectIter<'_>> {
        self.impl_iter(path.into())
    }

    pub fn reflect_iter_mut<'a>(&mut self, path: impl Into<ReflectPath<'a>>) -> Result<'a, ReflectIterMut<'_>> {
        self.impl_iter_mut(path.into())
    }

    pub fn reflect_as_number<'a>(&self, path: impl Into<ReflectPath<'a>>) -> Result<'a, ReflectNumber> {
        self.impl_as_number(path.into())
    }

    pub fn reflect_display<'a>(&self, path: impl Into<ReflectPath<'a>>, precision: Option<usize>) -> Result<'a, String> {
        self.impl_display(path.into(), precision)
    }  
}
downcast_rs::impl_downcast!(sync Reflect);

macro_rules! base_reflect_impl {
    (impl<$($g:ty),*> for $ty:ty where $($where:tt)*) => {
        impl<$($g),*> Reflect for $ty where $($where)* {
            fn impl_get<'a, 's>(&'s self, mut path: ReflectPath<'a>) -> Result<'a, MaybeOwnedReflect<'s>> {
                if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }

                Ok((self as &dyn Reflect).into())
            }

            fn impl_get_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> Result<'a, &mut dyn Reflect> {
                if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }

                Ok(self as &mut dyn Reflect)
            }

            fn impl_insert<'a>(&mut self, mut path: ReflectPath<'a>, value: Box<dyn Reflect>) -> Result<'a, ()> {
                if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }

                value
                    .downcast::<$ty>()
                    .map(|a| *self = *a)
                    .map_err(|v| ReflectError::wrong_type(type_name::<$ty>(), v.type_name()))
            }


            fn impl_display<'a>(&self, mut path: ReflectPath<'a>, _precision: Option<usize>) -> Result<'a, String> {
                if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }
                Ok(format!("{self}"))
            }

            fn duplicate(&self) -> Option<Box<dyn Reflect>> {
                Some(Box::new(self.clone()))
            }
        }
    };

    ($($ty:ty),*) => { $( base_reflect_impl!(impl<> for $ty where ); )* };
}
base_reflect_impl!(
    bool,
    String
);

impl<T:Reflect+Clone> Reflect for Option<T> {
    fn impl_get<'a, 's>(&'s self, path: ReflectPath<'a>) -> Result<'a, MaybeOwnedReflect<'s>> {
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

    fn impl_get_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> Result<'a, &mut dyn Reflect> {
        if let Some(next) = path.next() { return Err(ReflectError::entry_not_exist(next)) }

        Ok(self as &mut dyn Reflect)
    }

    fn impl_insert<'a>(&mut self, mut path: ReflectPath<'a>, value: Box<dyn Reflect>) -> Result<'a, ()> {
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


#[cfg(test)]
mod test {
    use crate::macros::Reflect;
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
