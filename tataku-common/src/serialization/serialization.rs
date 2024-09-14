use std::{
    rc::Rc,
    sync::Arc, 
    convert::TryInto, 
    num::ParseIntError, 
    collections::HashSet,
    collections::HashMap,
    string::FromUtf8Error, 
};

pub type SerializationResult<S> = Result<S, SerializationError>;

#[derive(Clone, Debug)]
pub struct SerializationError {
    pub inner: SerializationErrorEnum,
    pub stack: Vec<StackData>,
}
impl SerializationError {
    pub fn with_stack(mut self, stack: Vec<StackData>) -> Self {
        self.stack = stack;
        self
    }

    pub fn format_stack(&self) -> String {
        const INDENT: &str = "   ";
        self.stack.iter()
            .map(|StackData { depth, name, entries }| format!(
                "{}{name}{}", INDENT.repeat(*depth), 
                entries.iter().map(|e| format!("{}-> {e}", INDENT.repeat(*depth + 1)))
                .collect::<Vec<_>>().join("\n")
            ))
            .collect::<Vec<_>>().join("\n")
    }
}
impl From<SerializationErrorEnum> for SerializationError {
    fn from(value: SerializationErrorEnum) -> Self {
        Self {
            inner: value,
            stack: Vec::new()
        }
    }
}
impl From<FromUtf8Error> for SerializationError {
    fn from(utf8err: FromUtf8Error) -> Self {
        Self {
            inner: SerializationErrorEnum::FromUtf8Error(utf8err),
            stack: Vec::new()
        }
    }
}
impl From<ParseIntError> for SerializationError {
    fn from(interr: ParseIntError) -> Self {
        Self {
            inner: SerializationErrorEnum::ParseIntError(interr),
            stack: Vec::new()
        }
    }
}

impl PartialEq for SerializationError {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl Eq for SerializationError {}

impl core::fmt::Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}, stack: {}", self.inner, self.format_stack())
    }
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SerializationErrorEnum {
    OutOfBounds,
    FromUtf8Error(FromUtf8Error),
    ParseIntError(ParseIntError),
}


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



#[derive(Default, Clone, Debug)]
pub struct StackData {
    depth: usize,
    name: String,
    entries: Vec<String>,
}

pub struct SerializationReader {
    pub(self) data: Vec<u8>,
    pub(self) offset: usize,
    pub(self) stack: Vec<StackData>,
    pub(self) stack_depth: usize,
    pub debug: bool,
}
#[allow(dead_code)]
impl SerializationReader {
    pub fn new(data: Vec<u8>) -> SerializationReader {
        SerializationReader {
            data,
            offset: 0,
            stack: Vec::new(),
            stack_depth: 0,
            debug: false,
        }
    }
    pub fn debug(mut self) -> Self {
        self.debug = true;
        self
    }

    pub fn push_parent(&mut self, name: impl ToString) {
        self.stack.push(StackData {
            name: name.to_string(),
            entries: Vec::new(),
            depth: self.stack_depth,
        });
        self.stack_depth += 1;
    }
    pub fn pop_parent(&mut self) {
        self.stack_depth -= 1;
    }
    fn push_stack(&mut self, name: impl ToString, ty: &str) {
        if self.stack.is_empty() {
            self.stack.push(StackData::default());
        }
        self.stack.last_mut().unwrap().entries.push(format!("{} ({ty})", name.to_string()));
    }

    fn check_bounds(&mut self, size: usize) -> SerializationResult<()> {
        if self.data.len() < self.offset + size { 
            // println!("trying to read {size} at offset {} when len is {}", self.offset, self.data.len());
            return Err(SerializationError {
                inner: SerializationErrorEnum::OutOfBounds,
                stack: self.stack.clone()
            })
        }

        Ok(())
    }

    pub fn read<R:Serializable>(&mut self, name: impl ToString) -> SerializationResult<R> {
        let type_name = std::any::type_name::<R>();
        self.push_stack(name, type_name);
        // self.check_bounds(std::mem::size_of::<R>())?; // this breaks when R is an enum with differently sized variants
        R::read(self)
            .map_err(|e| e.with_stack(self.stack.clone()))
            .map(|v| { if self.debug { println!("got {v:?} ({type_name})") }; v})
    }
    pub fn can_read(&self) -> bool {
        self.data.len() - self.offset > 0
    }

    pub fn read_slice(&mut self, size: usize) -> SerializationResult<&[u8]> {
        self.check_bounds(size)?;
        let slice = &self.data[self.offset..self.offset+size];
        self.offset += size;

        Ok(slice)
    }

    /// unread the amount of bytes provided
    pub fn unread(&mut self, len: usize) {
        self.offset -= len;
        self.stack.last_mut().map(|s| s.entries.pop());
    }
}



#[derive(Default)]
pub struct SerializationWriter {
    pub(self) data: Vec<u8>
}
#[allow(dead_code)]
impl SerializationWriter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn data(self) -> Vec<u8> {
        self.data
    }

    pub fn write<S:Serializable>(&mut self, s: &S) {
        s.write(self);
    }

    pub fn write_raw_bytes(&mut self, bytes: &[u8]) {
        self.data.extend(bytes);
    }
}

#[allow(unused_imports)]
mod test {
    use super::{SerializationReader, SerializationWriter};

    #[test]
    fn writer_test() {
        let mut writer = SerializationWriter::new();
        writer.write(&1usize);
        writer.write(&("hello".to_owned()));

        let line = writer.data().iter().map(|b|format!("{:#x}", b)).collect::<Vec<String>>().join(", ");
        println!("{}", line);
    }
}
