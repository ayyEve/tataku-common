use crate::prelude::*;

#[derive(Default)]
pub struct DynMap {
    map: HashMap<String, Box<dyn Reflect>>
}
impl DynMap {
    pub fn set_chained(mut self, key: impl ToString, value: impl Reflect + 'static) -> Self {
        self.as_dyn_mut().reflect_insert(&key.to_string(), Box::new(value)).unwrap();
        self
    }
}

impl Reflect for DynMap {
    fn impl_get<'v>(&self, mut path: ReflectPath<'v>) -> Result<&dyn Reflect, ReflectError<'v>> {
        match path.next() {
            None => Ok(self.as_dyn()),
            Some(path) => self.map.get(path).map(|v| &**v)
                .ok_or(ReflectError::entry_not_exist(path)),
        }
    }

    fn impl_get_mut<'v>(&mut self, mut path: ReflectPath<'v>) -> Result<&mut dyn Reflect, ReflectError<'v>> {
        match path.next() {
            None => Ok(self.as_dyn_mut()),
            Some(path) => self.map.get_mut(path).map(|v| &mut **v)
                .ok_or(ReflectError::entry_not_exist(path)),
        }
    }

    fn impl_insert<'v>(&mut self, mut path: ReflectPath<'v>, value: Box<dyn Reflect>) -> Result<(), ReflectError<'v>> {
        match path.next() {
            None => value.downcast::<Self>().map(|v| *self = *v)
                .map_err(|v| ReflectError::wrong_type(std::any::type_name::<Self>(), v.type_name())),
            Some(p) => self.map.get_mut(p).map(|v| &mut **v)
                .ok_or(ReflectError::entry_not_exist(p))
                .and_then(|v| v.impl_insert(path, value)),
        }
    }

    fn impl_iter<'v>(&self, mut path: ReflectPath<'v>) -> Result<IterThing<'_>, ReflectError<'v>> {
        match path.next() {
            None => Ok(self.map.values()
                .map(|v| &**v)
                .collect::<Vec<_>>()
                .into()
            ),
            Some(p) => self.map.get(p)
                .ok_or(ReflectError::entry_not_exist(p))
                .and_then(|v| v.impl_iter(path)),
        }
    }

    fn impl_iter_mut<'v>(&mut self, mut path: ReflectPath<'v>) -> Result<IterThingMut<'_>, ReflectError<'v>> {
        match path.next() {
            None => Ok(self.map.values_mut()
                .map(|v| &mut **v)
                .collect::<Vec<_>>()
                .into()
            ),
            Some(p) => self.map.get_mut(p)
                .ok_or(ReflectError::entry_not_exist(p))
                .and_then(|v| v.impl_iter_mut(path)),
        }
    }
}