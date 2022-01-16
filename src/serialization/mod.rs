use std::{fs::File, io::{BufReader, Read, Result, Write}};

pub use serialization::*;
pub use versioned_serialization::*;

mod serialization;
mod versioned_serialization;




pub struct SimpleWriter {
    writer: SerializationWriter
}
impl SimpleWriter {
    pub fn new() -> Self {SimpleWriter{writer:SerializationWriter::new()}}
    pub fn done(self) -> Vec<u8> {self.writer.data()}
    pub fn write<W:Serializable>(mut self, s:W) -> Self {
        self.writer.write(s);
        self
    }
}

// save/load helpers

/// read database from file
pub fn open_database(filename:&str) -> Result<SerializationReader> {
    let file = File::open(filename)?; //.expect(&format!("Error opening database file {}", filename));
    let mut buf:Vec<u8> = Vec::new();
    BufReader::new(file).read_to_end(&mut buf)?; //.expect("error reading database file");
    Ok(SerializationReader::new(buf))
}

/// write database to file
pub fn save_database(filename:&str, writer:SerializationWriter) -> Result<()> {
    let bytes = writer.data();
    let bytes = bytes.as_slice();
    let mut f = File::create(filename)?;
    f.write_all(bytes)?;
    f.flush()?;
    Ok(())
}

