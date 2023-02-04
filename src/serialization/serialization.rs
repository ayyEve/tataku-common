use std::{
    rc::Rc,
    sync::Arc, 
    convert::TryInto, 
    collections::HashSet,
    collections::HashMap,
    string::FromUtf8Error, 
};

pub type SerializationResult<S> = Result<S, SerializationError>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SerializationError {
    OutOfBounds,
    FromUtf8Error(FromUtf8Error),
}
impl From<FromUtf8Error> for SerializationError {
    fn from(utf8err: FromUtf8Error) -> Self {
        SerializationError::FromUtf8Error(utf8err)
    }
}


pub trait Serializable {
    fn read(sr:&mut SerializationReader) -> SerializationResult<Self> where Self: Sized;
    fn write(&self, sw:&mut SerializationWriter);
}
impl Serializable for String {
    fn read(sr:&mut SerializationReader) -> SerializationResult<Self> {
        sr.read_string()
    }

    fn write(&self, sw:&mut SerializationWriter) {
        sw.write_string(self.clone());
    }
}
impl Serializable for f32 {
    fn read(sr:&mut SerializationReader) -> SerializationResult<Self> {
        sr.read_f32()
    }

    fn write(&self, sw:&mut SerializationWriter) {
        sw.write_f32(self.clone());
    }
}
impl Serializable for f64 {
    fn read(sr:&mut SerializationReader) -> SerializationResult<Self> {
        sr.read_f64()
    }

    fn write(&self, sw:&mut SerializationWriter) {
        sw.write_f64(self.clone());
    }
}
impl Serializable for usize {
    fn read(sr:&mut SerializationReader) -> SerializationResult<Self> {
        Ok(sr.read_u64()? as usize)
    }
    fn write(&self, sw:&mut SerializationWriter) {
        sw.write_u64(self.clone() as u64);
    }
}
impl Serializable for bool {
    fn read(sr:&mut SerializationReader) -> SerializationResult<Self> {
        Ok(sr.read_u8()? & 1 == 1)
    }

    fn write(&self, sw:&mut SerializationWriter) {
        sw.write_u8(if *self {1} else {0});
    }
}

// helper for references
// serialization for tuples
impl<T:Serializable+Clone,T2:Serializable+Clone> Serializable for (T, T2) {
    fn read(sr:&mut SerializationReader) -> SerializationResult<Self> {
        Ok((sr.read()?, sr.read()?))
    }

    fn write(&self, sw:&mut SerializationWriter) {
        let (a, b) = &self;
        sw.write(a);
        sw.write(b);
    }
}
// serialization for vecs
impl<T:Serializable+Clone> Serializable for &Vec<T> {
    fn read(_sr:&mut SerializationReader) -> SerializationResult<Self> {unimplemented!()}

    fn write(&self, sw:&mut SerializationWriter) {
        sw.write_u64(self.len() as u64);
        for i in self.iter() {
            sw.write(i)
        }
    }
}
impl<T:Serializable+Clone> Serializable for Vec<T> {
    fn read(sr:&mut SerializationReader) -> SerializationResult<Self> {
        let len = sr.read_u64()?;
        let mut out:Vec<T> = Vec::new();
        for _ in 0..len {out.push(sr.read()?)}
        Ok(out)
    }

    fn write(&self, sw:&mut SerializationWriter) {
        sw.write_u64(self.len() as u64);
        for i in self.iter() {
            sw.write(i)
        }
    }
}   

// serialization for options
impl<T:Serializable+Clone> Serializable for Option<T> {
    fn read(sr:&mut SerializationReader) -> SerializationResult<Self> {
        if sr.read()? {Ok(Some(sr.read()?))} else {Ok(None)}
    }

    fn write(&self, sw:&mut SerializationWriter) {
        sw.write(&self.is_some());
        if let Some(t) = self {sw.write(t)}
    }
}

// serialization for hashmap and hashsedt
impl<A:Serializable+core::hash::Hash+Eq+Clone, B:Serializable+Clone> Serializable for HashMap<A, B> {
    fn read(sr:&mut SerializationReader) -> SerializationResult<Self> {
        let count:usize = sr.read()?;

        let mut hashmap = HashMap::new();
        for _ in 0..count {
            let key = sr.read()?;
            let val = sr.read()?;
            hashmap.insert(key, val);
        }

        Ok(hashmap)
    }

    fn write(&self, sw:&mut SerializationWriter) {
        sw.write(&self.len());

        for (key, val) in self {
            sw.write(key);
            sw.write(val);
        }
    }
}

impl<T:Serializable+core::hash::Hash+Eq+Clone> Serializable for HashSet<T> {
    fn read(sr:&mut SerializationReader) -> SerializationResult<Self> {
        let len = sr.read_u64()?;
        let mut out:HashSet<T> = HashSet::new();
        for _ in 0..len { out.insert(sr.read()?); }
        Ok(out)
    }

    fn write(&self, sw:&mut SerializationWriter) {
        sw.write_u64(self.len() as u64);
        for i in self.iter() {
            sw.write(i)
        }
    }
}   


/// macro to help with checking if things are in range
macro_rules! check_range {
    ($self:expr, $offset:expr, $len: expr) => {
        if $self.data.len() < $offset + $len {return Err(SerializationError::OutOfBounds)}
    };
}
// implement for wrapper types
macro_rules! impl_wrapper {
    ($($t:ident),+) => {
        $(
            impl<T:Serializable> Serializable for $t<T> {
                fn read(sr:&mut SerializationReader) -> SerializationResult<Self> {
                    Ok($t::new(sr.read()?))
                }

                fn write(&self, sw:&mut SerializationWriter) {
                    self.as_ref().write(sw)
                }
            }
        )+
    };
}
impl_wrapper!(Box, Rc, Arc);

macro_rules! __impl_serializable_numbers {
    ($($t:ty),+) => {
        $(
            impl Serializable for $t {
                fn read(sr:&mut SerializationReader) -> SerializationResult<Self> {
                    use std::convert::TryInto;
                    let diff = (<$t>::BITS / 8) as usize;
                    check_range!(sr, sr.offset, diff);

                    let t = <$t>::from_le_bytes(sr.data[sr.offset..(sr.offset + diff)].try_into().unwrap());
                    sr.offset += diff;
                    Ok(t)
                }

                fn write(&self, sw:&mut SerializationWriter) {
                    sw.data.extend(self.to_le_bytes().iter());
                }
            }
        )+
    }
}
__impl_serializable_numbers![u8, i8, u16, i16, u32, i32, u64, i64, u128, i128];


pub struct SerializationReader {
    pub(self) data: Vec<u8>,
    pub(self) offset: usize,
}
#[allow(dead_code)]
impl SerializationReader {
    pub fn new(data:Vec<u8>) -> SerializationReader {
        SerializationReader {
            data,
            offset: 0
        }
    }

    pub fn read<R:Serializable>(&mut self) -> Result<R, SerializationError> {
        R::read(self)
    }
    pub fn can_read(&self) -> bool {
        self.data.len() - self.offset > 0
    }

    pub fn read_i8(&mut self) -> Result<i8, SerializationError> {
        check_range!(self, self.offset, 1);
        let b = self.data[self.offset];
        self.offset += 1;
        Ok(i8::from_le_bytes([b]))
    }
    pub fn read_u8(&mut self) -> Result<u8, SerializationError> {
        check_range!(self, self.offset, 1);
        let b = self.data[self.offset];
        self.offset += 1;
        Ok(u8::from_le_bytes([b]))
    }

    pub fn read_i16(&mut self) -> Result<i16, SerializationError> {
        check_range!(self, self.offset, 2);
        let s = &self.data[self.offset..(self.offset+2)];
        self.offset += 2;
        Ok(i16::from_le_bytes(s.try_into().unwrap()))
    }
    pub fn read_u16(&mut self) -> Result<u16, SerializationError> {
        check_range!(self, self.offset, 2);
        let s = &self.data[self.offset..(self.offset+2)];
        self.offset += 2;
        Ok(u16::from_le_bytes(s.try_into().unwrap()))
    }

    pub fn read_i32(&mut self) -> Result<i32, SerializationError> {
        check_range!(self, self.offset, 4);
        let s = &self.data[self.offset..(self.offset+4)];
        self.offset += 4;
        Ok(i32::from_le_bytes(s.try_into().unwrap()))
    }
    pub fn read_u32(&mut self) -> Result<u32, SerializationError> {
        check_range!(self, self.offset, 4);
        let s = &self.data[self.offset..(self.offset+4)];
        self.offset += 4;
        Ok(u32::from_le_bytes(s.try_into().unwrap()))
    }

    pub fn read_i64(&mut self) -> Result<i64, SerializationError> {
        check_range!(self, self.offset, 8);
        let s = &self.data[self.offset..(self.offset+8)];
        self.offset += 8;
        Ok(i64::from_le_bytes(s.try_into().unwrap()))
    }
    pub fn read_u64(&mut self) -> Result<u64, SerializationError> {
        check_range!(self, self.offset, 8);
        let s = &self.data[self.offset..(self.offset+8)];
        self.offset += 8;
        Ok(u64::from_le_bytes(s.try_into().unwrap()))
    }

    pub fn read_i128(&mut self) -> Result<i128, SerializationError> {
        check_range!(self, self.offset, 16);
        let s = &self.data[self.offset..(self.offset+16)];
        self.offset += 16;
        Ok(i128::from_le_bytes(s.try_into().unwrap()))
    }
    pub fn read_u128(&mut self) -> Result<u128, SerializationError> {
        check_range!(self, self.offset, 16);
        let s = &self.data[self.offset..(self.offset+16)];
        self.offset += 16;
        Ok(u128::from_le_bytes(s.try_into().unwrap()))
    }

    pub fn read_string(&mut self) -> Result<String, SerializationError> {
        let len = self.read_u64()? as usize;
        let offset = self.offset as usize;
        check_range!(self, offset, len);

        let bytes = self.data[offset..len+offset].to_vec();
        self.offset += len;

        Ok(String::from_utf8(bytes)?)
    }
    pub fn read_f32(&mut self) -> Result<f32, SerializationError> {
        check_range!(self, self.offset, 4);
        let s = &self.data[self.offset..(self.offset+4)];
        self.offset += 4;
        Ok(f32::from_le_bytes(s.try_into().unwrap()))
    }
    pub fn read_f64(&mut self) -> Result<f64, SerializationError> {
        check_range!(self, self.offset, 4);
        let s = &self.data[self.offset..(self.offset+8)];
        self.offset += 8;
        Ok(f64::from_le_bytes(s.try_into().unwrap()))
    }
    
}

pub struct SerializationWriter {
    pub(self) data: Vec<u8>
}
#[allow(dead_code)]
impl SerializationWriter {
    pub fn new() -> SerializationWriter {
        SerializationWriter {
            data:Vec::new()
        }
    }
    pub fn data(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn write<S>(&mut self, s:&S) where S:Serializable {
        s.write(self);
    }

    pub fn write_i8(&mut self, n:i8) {
        self.data.extend(n.to_le_bytes().iter());
    }
    pub fn write_u8(&mut self, n:u8) {
        self.data.extend(n.to_le_bytes().iter());
    }

    pub fn write_i16(&mut self, n:i16) {
        self.data.extend(n.to_le_bytes().iter());
    }
    pub fn write_u16(&mut self, n:u16) {
        self.data.extend(n.to_le_bytes().iter());
    }

    pub fn write_i32(&mut self, n:i32) {
        self.data.extend(n.to_le_bytes().iter());
    }
    pub fn write_u32(&mut self, n:u32) {
        self.data.extend(n.to_le_bytes().iter());
    }

    pub fn write_i64(&mut self, n:i64) {
        self.data.extend(n.to_le_bytes().iter());
    }
    pub fn write_u64(&mut self, n:u64) {
        self.data.extend(n.to_le_bytes().iter());
    }

    pub fn write_i128(&mut self, n:i128) {
        self.data.extend(n.to_le_bytes().iter());
    }
    pub fn write_u128(&mut self, n:u128) {
        self.data.extend(n.to_le_bytes().iter());
    }

    pub fn write_string(&mut self, s:String) {
        let bytes = s.as_bytes();
        let len = bytes.len() as u64;

        self.write(&len);
        self.data.extend(bytes.iter());
    }
    pub fn write_f32(&mut self, n:f32) {
        self.data.extend(n.to_le_bytes().iter());
    }
    pub fn write_f64(&mut self, n:f64) {
        self.data.extend(n.to_le_bytes().iter());
    }
    
}




#[allow(unused_imports)]
mod test {
    use super::{SerializationReader, SerializationWriter};

    #[test]
    fn writer_test() {
        let mut writer = SerializationWriter::new();
        writer.write(&(1 as usize));
        writer.write(&("hello".to_owned()));

        let line = writer.data().iter().map(|b|format!("{:#x}", b)).collect::<Vec<String>>().join(", ");
        println!("{}", line);
    }
}
