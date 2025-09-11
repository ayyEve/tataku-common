use crate::serialization::*;

#[derive(Default)]
pub struct SerializationWriter {
    pub(crate) data: Vec<u8>
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

/// helper for inline-writing data
#[derive(Default)]
pub struct SimpleWriter {
    writer: SerializationWriter
}
impl SimpleWriter {
    pub fn new() -> Self { 
        Self { 
            writer: SerializationWriter::new() 
        }
    }
    pub fn done(self) -> Vec<u8> { 
        self.writer.data() 
    }

    pub fn write<W:Serializable>(mut self, s: impl std::borrow::Borrow<W>) -> Self {
        self.writer.write(s.borrow());
        self
    }
}
