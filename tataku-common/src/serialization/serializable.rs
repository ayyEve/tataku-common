use std::{
    rc::Rc,
    sync::Arc, 
    convert::TryInto, 
    collections::HashSet,
    collections::HashMap,
};

use crate::serialization::*;

pub trait Serializable: core::fmt::Debug {
    fn read(sr: &mut SerializationReader) -> SerializationResult<Self> where Self: Sized;
    fn write(&self, sw: &mut SerializationWriter);
}
impl Serializable for String {
    fn read(sr: &mut SerializationReader) -> SerializationResult<Self> {
        let len = usize::read(sr)?;
        let bytes = sr.read_slice(len)?.to_vec();
        Ok(String::from_utf8(bytes)?)
    }

    fn write(&self, sw: &mut SerializationWriter) {
        let bytes = self.as_bytes();
        sw.write(&(bytes.len() as u64));
        sw.write_raw_bytes(bytes);
    }
}


macro_rules! impl_for_num {
    ($($t:ty),+) => { $(
        impl Serializable for $t {
            fn read(sr: &mut SerializationReader) -> SerializationResult<Self> {
                let bytes = sr.read_slice(std::mem::size_of::<$t>())?;
                Ok(Self::from_le_bytes(bytes.try_into().unwrap()))
            }

            fn write(&self, sw: &mut SerializationWriter) {
                sw.data.extend(self.to_le_bytes())
            }
        } )+
    }
}
impl_for_num![u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, f32, f64];

// usize is read as a u64
impl Serializable for usize {
    fn read(sr: &mut SerializationReader) -> SerializationResult<Self> {
        u64::read(sr).map(|n| n as usize)
    }

    fn write(&self, sw: &mut SerializationWriter) {
        (*self as u64).write(sw)
    }
}
impl Serializable for bool {
    fn read(sr: &mut SerializationReader) -> SerializationResult<Self> {
        Ok(u8::read(sr)? & 1 == 1)
    }

    fn write(&self, sw: &mut SerializationWriter) {
        sw.write::<u8>(&if *self {1} else {0});
    }
}

// serialization for tuples
impl<T:Serializable, T2:Serializable> Serializable for (T, T2) {
    fn read(sr: &mut SerializationReader) -> SerializationResult<Self> {
        Ok((T::read(sr)?, T2::read(sr)?))
    }

    fn write(&self, sw: &mut SerializationWriter) {
        sw.write(&self.0);
        sw.write(&self.1);
    }
}

// serialization for vecs
impl<T:Serializable> Serializable for &Vec<T> {
    fn read(_sr: &mut SerializationReader) -> SerializationResult<Self> { unimplemented!("cant read to a borrow lol") }

    fn write(&self, sw: &mut SerializationWriter) {
        sw.write(&self.len());
        for i in self.iter() {
            sw.write(i)
        }
    }
}
impl<T:Serializable> Serializable for Vec<T> {
    fn read(sr: &mut SerializationReader) -> SerializationResult<Self> {
        let count = usize::read(sr)?; //sr.read_u64("Vec len")?;
        let mut out:Vec<T> = Vec::with_capacity(count);
        for n in 0..count { out.push(sr.read(format!("Vec item #{n}"))?) }
        Ok(out)
    }

    fn write(&self, sw: &mut SerializationWriter) {
        sw.write(&self.len());

        for i in self.iter() {
            sw.write(i)
        }
    }
}   

// serialization for options
impl<T:Serializable> Serializable for Option<T> {
    fn read(sr: &mut SerializationReader) -> SerializationResult<Self> {
        if bool::read(sr)? { 
            Ok(Some(T::read(sr)?)) 
        } else { 
            Ok(None) 
        }
    }

    fn write(&self, sw: &mut SerializationWriter) {
        sw.write(&self.is_some());
        if let Some(t) = self { sw.write(t) }
    }
}

// serialization for hashmap and hashsedt
impl<A:Serializable+core::hash::Hash+Eq, B:Serializable> Serializable for HashMap<A, B> {
    fn read(sr: &mut SerializationReader) -> SerializationResult<Self> {
        let count = u64::read(sr)?;

        let mut hashmap = HashMap::new();
        for _ in 0..count {
            let key = A::read(sr)?; // sr.read(format!("HashMap key #{n}"))?;
            let val = B::read(sr)?; // sr.read(format!("HashMap value #{n}"))?;
            hashmap.insert(key, val);
        }

        Ok(hashmap)
    }

    fn write(&self, sw: &mut SerializationWriter) {
        sw.write(&self.len());

        for (key, val) in self {
            sw.write(key);
            sw.write(val);
        }
    }
}

impl<T:Serializable+core::hash::Hash+Eq> Serializable for HashSet<T> {
    fn read(sr: &mut SerializationReader) -> SerializationResult<Self> {
        let count = u64::read(sr)?; 
        let mut out: HashSet<T> = HashSet::new();
        for n in 0..count { out.insert(sr.read(format!("HashSet value #{n}"))?); }
        Ok(out)
    }

    fn write(&self, sw: &mut SerializationWriter) {
        sw.write(&(self.len() as u64));
        for i in self.iter() {
            sw.write(i)
        }
    }
}   


// implement for wrapper types
macro_rules! impl_wrapper {
    ($($t:ident),+) => {
        $(impl<T:Serializable> Serializable for $t<T> {
            fn read(sr: &mut SerializationReader) -> SerializationResult<Self> {
                Ok(Self::new(T::read(sr)?))
            }

            fn write(&self, sw: &mut SerializationWriter) {
                self.as_ref().write(sw)
            }
        })+
    };
}
impl_wrapper!(Box, Rc, Arc);


// TODO: is there a better way of doing this?
#[derive(Default, Clone, Debug)]
pub struct StackData {
    pub depth: usize,
    pub name: String,
    pub entries: Vec<String>,
}

#[allow(unused_imports)]
mod test {
    use super::{ SerializationReader, SerializationWriter };

    #[test]
    fn writer_test() {
        let mut writer = SerializationWriter::new();
        writer.write(&1usize);
        writer.write(&("hello".to_owned()));

        let line = writer.data().iter().map(|b|format!("{:#x}", b)).collect::<Vec<String>>().join(", ");
        println!("{}", line);
    }
}
