use crate::prelude::*;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[derive(Reflect)]
pub enum ReplayAction {
    Press(KeyPress),
    Release(KeyPress),
    MousePos(f32, f32)
}
impl Serializable for ReplayAction {
    fn read(sr:&mut SerializationReader) -> SerializationResult<Self> {
        sr.push_parent("ReplayAction");

        let a = Ok(match sr.read::<u8>("id")? {
            0 => ReplayAction::Press(sr.read("press")?),
            1 => ReplayAction::Release(sr.read("release")?),
            2 => ReplayAction::MousePos(sr.read("x")?, sr.read("y")?),
            _ => panic!("error reading replay frame type")
        });
        sr.pop_parent();

        a
    }

    fn write(&self, sw:&mut SerializationWriter) {
        match self {
            ReplayAction::Press(k) => {
                sw.write::<u8>(&0);
                sw.write(k);
            }
            ReplayAction::Release(k) => {
                sw.write::<u8>(&1);
                sw.write(k);
            }
            ReplayAction::MousePos(x, y) => {
                sw.write::<u8>(&2);
                sw.write(x);
                sw.write(y);
            }
        }
    }
}

// impl Reflect for ReplayAction {
//     fn impl_get<'a>(
//         &self,
//         mut path: ReflectPath<'a>,
//     ) -> Result<&dyn Reflect, ReflectError<'a>> {
//         match path.next() {
//             None => Ok(self as &dyn Reflect),
//             Some("Press") => {
//                 match self {
//                     Self::Press(f0, ..) => {
//                         match path.next() {
//                             None => Ok(self as &dyn Reflect),
//                             Some("0") => f0.impl_get(path),
//                             Some(p) => Err(ReflectError::entry_not_exist(p)),
//                         }
//                     }
//                     Self::Release(..) => {
//                         Err(ReflectError::wrong_variant("Press", "Release"))
//                     }
//                     Self::MousePos(..) => {
//                         Err(ReflectError::wrong_variant("Press", "MousePos"))
//                     }
//                 }
//             }
//             Some("Release") => {
//                 match self {
//                     Self::Release(f0, ..) => {
//                         match path.next() {
//                             None => Ok(self as &dyn Reflect),
//                             Some("0") => f0.impl_get(path),
//                             Some(p) => Err(ReflectError::entry_not_exist(p)),
//                         }
//                     }
//                     Self::Press(..) => {
//                         Err(ReflectError::wrong_variant("Release", "Press"))
//                     }
//                     Self::MousePos(..) => {
//                         Err(ReflectError::wrong_variant("Release", "MousePos"))
//                     }
//                 }
//             }
//             Some("MousePos") => {
//                 match self {
//                     Self::MousePos(f0, f1, ..) => {
//                         match path.next() {
//                             None => Ok(self as &dyn Reflect),
//                             Some("0") => f0.impl_get(path),
//                             Some("1") => f1.impl_get(path),
//                             Some(p) => Err(ReflectError::entry_not_exist(p)),
//                         }
//                     }
//                     Self::Press(..) => {
//                         Err(ReflectError::wrong_variant("MousePos", "Press"))
//                     }
//                     Self::Release(..) => {
//                         Err(ReflectError::wrong_variant("MousePos", "Release"))
//                     }
//                 }
//             }
//             Some(p) => Err(ReflectError::entry_not_exist(p)),
//         }
//     }
//     fn impl_get_mut<'a>(
//         &mut self,
//         mut path: ReflectPath<'a>,
//     ) -> Result<&mut dyn Reflect, ReflectError<'a>> {
//         match path.next() {
//             None => Ok(self as &mut dyn Reflect),
//             Some("Press") => {
//                 match self {
//                     s @ Self::Press(..) => {
//                         match path.next() {
//                             None => Ok(s as &mut dyn Reflect),
//                             Some("0") => if let Self::Press(f0) = s { f0.impl_get_mut(path) } else { unreachable!(); },
//                             Some(p) => Err(ReflectError::entry_not_exist(p)),
//                         }
//                     }
//                     Self::Release(..) => {
//                         Err(ReflectError::wrong_variant("Press", "Release"))
//                     }
//                     Self::MousePos(..) => {
//                         Err(ReflectError::wrong_variant("Press", "MousePos"))
//                     }
//                 }
//             }
//             Some("Release") => {
//                 match self {
//                     Self::Release(f0, ..) => {
//                         match path.next() {
//                             None => Ok(self as &mut dyn Reflect),
//                             Some("0") => f0.impl_get_mut(path),
//                             Some(p) => Err(ReflectError::entry_not_exist(p)),
//                         }
//                     }
//                     Self::Press(..) => {
//                         Err(ReflectError::wrong_variant("Release", "Press"))
//                     }
//                     Self::MousePos(..) => {
//                         Err(ReflectError::wrong_variant("Release", "MousePos"))
//                     }
//                 }
//             }
//             Some("MousePos") => {
//                 match self {
//                     Self::MousePos(f0, f1, ..) => {
//                         match path.next() {
//                             None => Ok(self as &mut dyn Reflect),
//                             Some("0") => f0.impl_get_mut(path),
//                             Some("1") => f1.impl_get_mut(path),
//                             Some(p) => Err(ReflectError::entry_not_exist(p)),
//                         }
//                     }
//                     Self::Press(..) => {
//                         Err(ReflectError::wrong_variant("MousePos", "Press"))
//                     }
//                     Self::Release(..) => {
//                         Err(ReflectError::wrong_variant("MousePos", "Release"))
//                     }
//                 }
//             }
//             Some(p) => Err(ReflectError::entry_not_exist(p)),
//         }
//     }
//     fn impl_insert<'a>(
//         &mut self,
//         mut path: ReflectPath<'a>,
//         value: Box<dyn Reflect>,
//     ) -> Result<(), ReflectError<'a>> {
//         match path.next() {
//             None => {
//                 value
//                     .downcast::<Self>()
//                     .map(|v| *self = *v)
//                     .map_err(|_| ReflectError::wrong_type(
//                         std::any::type_name::<Self>(),
//                         "TODO: cry",
//                     ))
//             }
//             Some("Press") => {
//                 match self {
//                     Self::Press(f0, ..) => {
//                         match path.next() {
//                             None => Err(ReflectError::entry_not_exist("Press")),
//                             Some("0") => f0.impl_insert(path, value),
//                             Some(p) => Err(ReflectError::entry_not_exist(p)),
//                         }
//                     }
//                     Self::Release(..) => {
//                         Err(ReflectError::wrong_variant("Press", "Release"))
//                     }
//                     Self::MousePos(..) => {
//                         Err(ReflectError::wrong_variant("Press", "MousePos"))
//                     }
//                 }
//             }
//             Some("Release") => {
//                 match self {
//                     Self::Release(f0, ..) => {
//                         match path.next() {
//                             None => Err(ReflectError::entry_not_exist("Release")),
//                             Some("0") => f0.impl_insert(path, value),
//                             Some(p) => Err(ReflectError::entry_not_exist(p)),
//                         }
//                     }
//                     Self::Press(..) => {
//                         Err(ReflectError::wrong_variant("Release", "Press"))
//                     }
//                     Self::MousePos(..) => {
//                         Err(ReflectError::wrong_variant("Release", "MousePos"))
//                     }
//                 }
//             }
//             Some("MousePos") => {
//                 match self {
//                     Self::MousePos(f0, f1, ..) => {
//                         match path.next() {
//                             None => Err(ReflectError::entry_not_exist("MousePos")),
//                             Some("0") => f0.impl_insert(path, value),
//                             Some("1") => f1.impl_insert(path, value),
//                             Some(p) => Err(ReflectError::entry_not_exist(p)),
//                         }
//                     }
//                     Self::Press(..) => {
//                         Err(ReflectError::wrong_variant("MousePos", "Press"))
//                     }
//                     Self::Release(..) => {
//                         Err(ReflectError::wrong_variant("MousePos", "Release"))
//                     }
//                 }
//             }
//             Some(p) => Err(ReflectError::entry_not_exist(p)),
//         }
//     }
//     fn impl_iter<'a>(
//         &self,
//         mut path: ReflectPath<'a>,
//     ) -> Result<IterThing<'_>, ReflectError<'a>> {
//         match path.next() {
//             None => Ok(::alloc::vec::Vec::new().into()),
//             Some("Press") => {
//                 match self {
//                     Self::Press(f0, ..) => {
//                         match path.next() {
//                             None => {
//                                 Ok(
//                                     <[_]>::into_vec(
//                                             #[rustc_box]
//                                             ::alloc::boxed::Box::new([f0 as &dyn Reflect]),
//                                         )
//                                         .into(),
//                                 )
//                             }
//                             Some("0") => f0.impl_iter(path),
//                             Some(p) => Err(ReflectError::entry_not_exist(p)),
//                         }
//                     }
//                     Self::Release(..) => {
//                         Err(ReflectError::wrong_variant("Press", "Release"))
//                     }
//                     Self::MousePos(..) => {
//                         Err(ReflectError::wrong_variant("Press", "MousePos"))
//                     }
//                 }
//             }
//             Some("Release") => {
//                 match self {
//                     Self::Release(f0, ..) => {
//                         match path.next() {
//                             None => {
//                                 Ok(
//                                     <[_]>::into_vec(
//                                             #[rustc_box]
//                                             ::alloc::boxed::Box::new([f0 as &dyn Reflect]),
//                                         )
//                                         .into(),
//                                 )
//                             }
//                             Some("0") => f0.impl_iter(path),
//                             Some(p) => Err(ReflectError::entry_not_exist(p)),
//                         }
//                     }
//                     Self::Press(..) => {
//                         Err(ReflectError::wrong_variant("Release", "Press"))
//                     }
//                     Self::MousePos(..) => {
//                         Err(ReflectError::wrong_variant("Release", "MousePos"))
//                     }
//                 }
//             }
//             Some("MousePos") => {
//                 match self {
//                     Self::MousePos(f0, f1, ..) => {
//                         match path.next() {
//                             None => {
//                                 Ok(
//                                     <[_]>::into_vec(
//                                             #[rustc_box]
//                                             ::alloc::boxed::Box::new([
//                                                 f0 as &dyn Reflect,
//                                                 f1 as &dyn Reflect,
//                                             ]),
//                                         )
//                                         .into(),
//                                 )
//                             }
//                             Some("0") => f0.impl_iter(path),
//                             Some("1") => f1.impl_iter(path),
//                             Some(p) => Err(ReflectError::entry_not_exist(p)),
//                         }
//                     }
//                     Self::Press(..) => {
//                         Err(ReflectError::wrong_variant("MousePos", "Press"))
//                     }
//                     Self::Release(..) => {
//                         Err(ReflectError::wrong_variant("MousePos", "Release"))
//                     }
//                 }
//             }
//             Some(p) => Err(ReflectError::entry_not_exist(p)),
//         }
//     }
//     fn impl_iter_mut<'a>(
//         &mut self,
//         mut path: ReflectPath<'a>,
//     ) -> Result<IterThingMut<'_>, ReflectError<'a>> {
//         match path.next() {
//             None => Ok(::alloc::vec::Vec::new().into()),
//             Some("Press") => {
//                 match self {
//                     Self::Press(f0, ..) => {
//                         match path.next() {
//                             None => {
//                                 Ok(
//                                     <[_]>::into_vec(
//                                             #[rustc_box]
//                                             ::alloc::boxed::Box::new([f0 as &mut dyn Reflect]),
//                                         )
//                                         .into(),
//                                 )
//                             }
//                             Some("0") => f0.impl_iter_mut(path),
//                             Some(p) => Err(ReflectError::entry_not_exist(p)),
//                         }
//                     }
//                     Self::Release(..) => {
//                         Err(ReflectError::wrong_variant("Press", "Release"))
//                     }
//                     Self::MousePos(..) => {
//                         Err(ReflectError::wrong_variant("Press", "MousePos"))
//                     }
//                 }
//             }
//             Some("Release") => {
//                 match self {
//                     Self::Release(f0, ..) => {
//                         match path.next() {
//                             None => {
//                                 Ok(
//                                     <[_]>::into_vec(
//                                             #[rustc_box]
//                                             ::alloc::boxed::Box::new([f0 as &mut dyn Reflect]),
//                                         )
//                                         .into(),
//                                 )
//                             }
//                             Some("0") => f0.impl_iter_mut(path),
//                             Some(p) => Err(ReflectError::entry_not_exist(p)),
//                         }
//                     }
//                     Self::Press(..) => {
//                         Err(ReflectError::wrong_variant("Release", "Press"))
//                     }
//                     Self::MousePos(..) => {
//                         Err(ReflectError::wrong_variant("Release", "MousePos"))
//                     }
//                 }
//             }
//             Some("MousePos") => {
//                 match self {
//                     Self::MousePos(f0, f1, ..) => {
//                         match path.next() {
//                             None => {
//                                 Ok(
//                                     <[_]>::into_vec(
//                                             #[rustc_box]
//                                             ::alloc::boxed::Box::new([
//                                                 f0 as &mut dyn Reflect,
//                                                 f1 as &mut dyn Reflect,
//                                             ]),
//                                         )
//                                         .into(),
//                                 )
//                             }
//                             Some("0") => f0.impl_iter_mut(path),
//                             Some("1") => f1.impl_iter_mut(path),
//                             Some(p) => Err(ReflectError::entry_not_exist(p)),
//                         }
//                     }
//                     Self::Press(..) => {
//                         Err(ReflectError::wrong_variant("MousePos", "Press"))
//                     }
//                     Self::Release(..) => {
//                         Err(ReflectError::wrong_variant("MousePos", "Release"))
//                     }
//                 }
//             }
//             Some(p) => Err(ReflectError::entry_not_exist(p)),
//         }
//     }
// }