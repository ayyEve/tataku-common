use crate::prelude::*;
use std::any::type_name;

macro_rules! impl_reflect_tuple {
    ($($g:ident $ty:tt => $v:literal => $i:tt),+) => {
        impl<$($g: Reflect),+> Reflect for ($($ty),+ ,) {
            fn impl_get<'a, 's>(&'s self, mut path: ReflectPath<'a>) -> ReflectResult<'a, MaybeOwnedReflect<'s>> {
                match path.next() {
                    None => Ok(self.as_dyn().into()),
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

            fn impl_get_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> ReflectResult<'a, &mut dyn Reflect> {
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

            fn impl_insert<'a>(&mut self, mut path: ReflectPath<'a>, value: Box<dyn Reflect>) -> ReflectResult<'a, ()> {
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


            fn impl_iter<'a>(&self, mut path: ReflectPath<'a>) -> ReflectResult<'a, ReflectIter<'_>> {
                match path.next() {
                    None => Ok(ReflectIter { 
                        iter: vec![
                            $( self.$i.as_dyn() ),+
                        ].into_iter().enumerate().map(|(k, v)| {
                            ReflectIterEntry {
                                item: v as &dyn Reflect,
                                index: Some(ReflectItemIndex::Number(k))
                            }
                        })
                        .collect()
                    }),
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
            fn impl_iter_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> ReflectResult<'a, ReflectIterMut<'_>> {
                match path.next() {
                    None => Ok(ReflectIterMut { 
                        iter: vec![
                            $( self.$i.as_dyn_mut() ),+
                        ].into_iter().enumerate().map(|(k, v)| {
                            ReflectIterMutEntry {
                                item: v as &mut dyn Reflect,
                                index: Some(ReflectItemIndex::Number(k))
                            }
                        })
                        .collect()
                    }),
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
