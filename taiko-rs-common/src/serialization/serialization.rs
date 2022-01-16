use std::{
    rc::Rc,
    sync::Arc, 
    convert::TryInto, 
};

pub trait Serializable {
    fn read(sr:&mut SerializationReader) -> Self;
    fn write(&self, sw:&mut SerializationWriter);
}
impl Serializable for String {
    fn read(sr:&mut SerializationReader) -> Self {
        sr.read_string()
    }

    fn write(&self, sw:&mut SerializationWriter) {
        sw.write_string(self.clone());
    }
}
impl Serializable for f32 {
    fn read(sr:&mut SerializationReader) -> Self {
        sr.read_f32()
    }

    fn write(&self, sw:&mut SerializationWriter) {
        sw.write_f32(self.clone());
    }
}
impl Serializable for f64 {
    fn read(sr:&mut SerializationReader) -> Self {
        sr.read_f64()
    }

    fn write(&self, sw:&mut SerializationWriter) {
        sw.write_f64(self.clone());
    }
}
impl Serializable for usize {
    fn read(sr:&mut SerializationReader) -> Self {
        sr.read_u64() as usize
    }
    fn write(&self, sw:&mut SerializationWriter) {
        sw.write_u64(self.clone() as u64);
    }
}
impl Serializable for bool {
    fn read(sr:&mut SerializationReader) -> Self {
        sr.read_u8() & 1 == 1
    }

    fn write(&self, sw:&mut SerializationWriter) {
        sw.write_u8(if *self {1} else {0});
    }
}

// helper for references
// serialization for tuples
impl<T:Serializable+Clone,T2:Serializable+Clone> Serializable for (T, T2) {
    fn read(sr:&mut SerializationReader) -> Self {
        (sr.read(), sr.read())
    }

    fn write(&self, sw:&mut SerializationWriter) {
        let (a, b) = self.clone();
        sw.write(a);
        sw.write(b);
    }
}
// serialization for vecs
impl<T:Serializable+Clone> Serializable for &Vec<T> {
    fn read(_sr:&mut SerializationReader) -> Self {todo!()}

    fn write(&self, sw:&mut SerializationWriter) {
        // println!("write vec start");
        sw.write_u64(self.len() as u64);
        for i in self.iter() {
            sw.write(i.clone())
        }
        // println!("write vec end");
    }
}
impl<T:Serializable+Clone> Serializable for Vec<T> {
    fn read(sr:&mut SerializationReader) -> Self {
        let mut out:Vec<T> = Vec::new();
        for _ in 0..sr.read_u64() {out.push(sr.read())}
        out
    }

    fn write(&self, sw:&mut SerializationWriter) {
        // println!("write vec start");
        sw.write_u64(self.len() as u64);
        for i in self.iter() {
            sw.write(i.clone())
        }
        // println!("write vec end");
    }
}   
// serialization for options
impl<T:Serializable+Clone> Serializable for Option<T> {
    fn read(sr:&mut SerializationReader) -> Self {
        if sr.read() {Some(sr.read())} else {None}
    }

    fn write(&self, sw:&mut SerializationWriter) {
        sw.write(self.is_some());
        if let Some(t) = self {sw.write(t.clone())}
    }
}

// implement for wrapper types
macro_rules! impl_wrapper {
    ($($t:ident),+) => {
        $(
            impl<T:Serializable> Serializable for $t<T> {
                fn read(sr:&mut SerializationReader) -> Self {
                    $t::new(sr.read())
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
                fn read(sr:&mut SerializationReader) -> Self {
                    use std::convert::TryInto;
                    
                    let diff = (<$t>::BITS / 8) as usize;
                    let t = <$t>::from_le_bytes(sr.data[sr.offset..(sr.offset + diff)].try_into().unwrap());
                    sr.offset += diff;
                    t
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

    pub fn read<R:Serializable>(&mut self) -> R {
        R::read(self)
    }
    pub fn can_read(&self) -> bool {
        self.data.len() - self.offset > 0
    }

    pub fn read_i8(&mut self) -> i8 {
        let b = self.data[self.offset];
        self.offset += 1;
        i8::from_le_bytes([b])
    }
    pub fn read_u8(&mut self) -> u8 {
        let b = self.data[self.offset];
        self.offset += 1;
        u8::from_le_bytes([b])
    }

    pub fn read_i16(&mut self) -> i16 {
        let s = &self.data[self.offset..(self.offset+2)];
        self.offset += 2;
        i16::from_le_bytes(s.try_into().unwrap())
    }
    pub fn read_u16(&mut self) -> u16 {
        let s = &self.data[self.offset..(self.offset+2)];
        self.offset += 2;
        u16::from_le_bytes(s.try_into().unwrap())
    }

    pub fn read_i32(&mut self) -> i32 {
        let s = &self.data[self.offset..(self.offset+4)];
        self.offset += 4;
        i32::from_le_bytes(s.try_into().unwrap())
    }
    pub fn read_u32(&mut self) -> u32 {
        let s = &self.data[self.offset..(self.offset+4)];
        self.offset += 4;
        u32::from_le_bytes(s.try_into().unwrap())
    }

    pub fn read_i64(&mut self) -> i64 {
        let s = &self.data[self.offset..(self.offset+8)];
        self.offset += 8;
        i64::from_le_bytes(s.try_into().unwrap())
    }
    pub fn read_u64(&mut self) -> u64 {
        let s = &self.data[self.offset..(self.offset+8)];
        self.offset += 8;
        u64::from_le_bytes(s.try_into().unwrap())
    }

    pub fn read_i128(&mut self) -> i128 {
        let s = &self.data[self.offset..(self.offset+16)];
        self.offset += 16;
        i128::from_le_bytes(s.try_into().unwrap())
    }
    pub fn read_u128(&mut self) -> u128 {
        let s = &self.data[self.offset..(self.offset+16)];
        self.offset += 16;
        u128::from_le_bytes(s.try_into().unwrap())
    }

    pub fn read_string(&mut self) -> String {
        let len = self.read_u64() as usize;
        let offset = self.offset as usize;
        let bytes = self.data[offset..len+offset].to_vec();
        self.offset += len;

        String::from_utf8(bytes).expect("Error parsing bytes to utf8")
    }
    pub fn read_f32(&mut self) -> f32 {
        let s = &self.data[self.offset..(self.offset+4)];
        self.offset += 4;
        f32::from_le_bytes(s.try_into().unwrap())
    }
    pub fn read_f64(&mut self) -> f64 {
        let s = &self.data[self.offset..(self.offset+8)];
        self.offset += 8;
        f64::from_le_bytes(s.try_into().unwrap())
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

    pub fn write<S>(&mut self, s:S) where S:Serializable {
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

        self.write(len);
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
        writer.write(1 as usize);
        writer.write("hello".to_owned());

        let line = writer.data().iter().map(|b|format!("{:#x}", b)).collect::<Vec<String>>().join(", ");
        println!("{}", line);
    }
}
