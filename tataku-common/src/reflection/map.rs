use crate::prelude::*;

#[derive(Default)]
pub struct DynMap {
    map: HashMap<String, Box<dyn Reflect>>
}
impl DynMap {
    pub fn set_chained(mut self, key: impl ToString, value: impl Reflect + 'static) -> Self {
        self.as_dyn_mut()
            .reflect_insert(&key.to_string(), Box::new(value))
            .unwrap();
        self
    }
}

impl Reflect for DynMap {
    fn impl_get<'v, 's>(&'s self, mut path: ReflectPath<'v>) -> ReflectResult<'v, MaybeOwnedReflect<'s>> {
        match path.next() {
            None => Ok(self.as_dyn().into()),
            Some(p) => self.map.get(p)
                .ok_or(ReflectError::entry_not_exist(p))
                .map(|v| &**v)
                .and_then(|v| v.impl_get(path)),
        }
    }

    fn impl_get_mut<'v>(&mut self, mut path: ReflectPath<'v>) -> ReflectResult<'v, &mut dyn Reflect> {
        match path.next() {
            None => Ok(self.as_dyn_mut()),
            Some(p) => self.map.get_mut(p)
                .map(|v| &mut **v)
                .ok_or(ReflectError::entry_not_exist(p))
                .and_then(|v| v.impl_get_mut(path)),
        }
    }

    fn impl_insert<'v>(&mut self, mut path: ReflectPath<'v>, value: Box<dyn Reflect>) -> ReflectResult<'v, ()> {
        match path.next() {
            None => value
                .downcast::<Self>()
                .map(|v| *self = *v)
                .map_err(|v| ReflectError::wrong_type(std::any::type_name::<Self>(), v.type_name())),
            
            Some(p) => {
                if path.has_next() {
                    self.map
                        .entry(p.to_owned())
                        .or_insert_with(|| Box::new(DynMap::default()))
                        .impl_insert(path, value)
                } else {
                    self.map.insert(p.to_owned(), value);
                    Ok(())
                }
            }
        }
    }

    fn impl_iter<'v>(&self, mut path: ReflectPath<'v>) -> ReflectResult<'v, ReflectIter<'_>> {
        match path.next() {
            None => Ok(ReflectIter::new(
                self.map
                .iter()
                .map(|(k, v)| ReflectIterEntry {
                    item: &**v,
                    index: Some(ReflectItemIndex::Value(k as &dyn Reflect))
                })
            )),
            Some(p) => self.map.get(p)
                .map(|v| &**v)
                .ok_or(ReflectError::entry_not_exist(p))
                .and_then(|v| v.impl_iter(path)),
        }
    }

    fn impl_iter_mut<'v>(&mut self, mut path: ReflectPath<'v>) -> ReflectResult<'v, ReflectIterMut<'_>> {
        match path.next() {
            None => Ok(ReflectIterMut::new(
                self.map
                .iter_mut()
                .map(|(k, v)| ReflectIterMutEntry {
                    item: &mut **v,
                    index: Some(ReflectItemIndex::Value(k as &dyn Reflect))
                })
            )),
            Some(p) => self.map.get_mut(p)
                .map(|v| &mut **v)
                .ok_or(ReflectError::entry_not_exist(p))
                .and_then(|v| v.impl_iter_mut(path)),
        }
    }

    fn duplicate(&self) -> Option<Box<dyn Reflect>> {
        Some(Box::new(self.map
            .iter()
            .filter_map(|(k, v)| 
                v.duplicate().map(|v| (k.clone(), v))
            )
            .collect::<HashMap<String, Box<dyn Reflect>>>()
        ))
    }

    fn from_string(_: &str) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized {
        Err(ReflectError::NoFromString)
    }

}
