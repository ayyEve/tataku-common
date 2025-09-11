use crate::reflection::*;

#[derive(Default)]
pub struct ReflectIter<'a> {
    pub(crate) iter: VecDeque<ReflectIterEntry<'a>>,
}
impl<'a> ReflectIter<'a> {
    pub fn new(iter: impl Iterator<Item=ReflectIterEntry<'a>>) -> Self {
        Self {
            iter: iter.collect(),
        }
    }
}
impl<'a> Iterator for ReflectIter<'a> {
    type Item = ReflectIterEntry<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.pop_front()
    }
}
#[derive(Default)]
pub struct ReflectIterMut<'a> {
    pub(crate) iter: VecDeque<ReflectIterMutEntry<'a>>,
}
impl<'a> ReflectIterMut<'a> {
    pub fn new(iter: impl Iterator<Item=ReflectIterMutEntry<'a>>) -> Self {
        Self {
            iter: iter.collect(),
        }
    }
}
impl<'a> Iterator for ReflectIterMut<'a> {
    type Item = ReflectIterMutEntry<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.pop_front()
    }
}

// /// currently does not do lazy iteration
// #[derive(Default)]
// pub struct ReflectIter<'a> {
//     items: std::vec::IntoIter<&'a dyn Reflect>
// }
// impl<'a> From<Vec<&'a dyn Reflect>> for ReflectIter<'a> {
//     fn from(items: Vec<&'a dyn Reflect>) -> Self {
//         Self {
//             items: items.into_iter(),
//         }
//     }
// }
// impl<'a> Iterator for ReflectIter<'a> {
//     type Item = &'a dyn Reflect;
//     fn next(&mut self) -> Option<Self::Item> {
//         self.items.next()
//     }
// }

// /// currently does not do lazy iteration
// #[derive(Default)]
// pub struct ReflectIterMut<'a> {
//     items: std::vec::IntoIter<&'a mut dyn Reflect>,
// }
// impl<'a> From<Vec<&'a mut dyn Reflect>> for ReflectIterMut<'a> {
//     fn from(items: Vec<&'a mut dyn Reflect>) -> Self {
//         Self {
//             items: items.into_iter(),
//         }
//     }
// }
// impl<'a> Iterator for ReflectIterMut<'a> {
//     type Item = &'a mut dyn Reflect;
//     fn next(&mut self) -> Option<Self::Item> {
//         self.items.next()
//     }
// }




use std::collections::VecDeque;
use std::ops::{ Deref, DerefMut };

pub struct ReflectIterEntry<'a> {
    pub item: &'a dyn Reflect,
    pub index: Option<ReflectItemIndex<'a>>,
}
impl Deref for ReflectIterEntry<'_> {
    type Target = dyn Reflect;
    fn deref(&self) -> &Self::Target {
        self.item
    }
}

pub struct ReflectIterMutEntry<'a> {
    pub item: &'a mut dyn Reflect,
    pub index: Option<ReflectItemIndex<'a>>,
}
impl Deref for ReflectIterMutEntry<'_> {
    type Target = dyn Reflect;
    fn deref(&self) -> &Self::Target {
        self.item
    }
}
impl DerefMut for ReflectIterMutEntry<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item
    }
}

pub enum ReflectItemIndex<'a> {
    Number(usize),
    Value(&'a dyn Reflect),
}