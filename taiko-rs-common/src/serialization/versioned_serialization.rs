use super::{Serializable, SerializationReader, SerializationWriter};

pub struct VersionedSerializationReader<V:PartialOrd+Serializable> {
    pub version: V,
    reader: SerializationReader
}
#[allow(dead_code)]
impl<V:PartialOrd+Serializable> VersionedSerializationReader<V> {
    pub fn new(data:Vec<u8>) -> VersionedSerializationReader<V> {

        let mut reader = SerializationReader::new(data);
        let version = reader.read::<V>();

        VersionedSerializationReader {
            version,
            reader
        }
    }

    pub fn read<R:Serializable>(&mut self, version_available:V, default:R) -> R {
        if self.version >= version_available {
            R::read(&mut self.reader)
        } else {
            default
        }
    }
}

pub struct VersionedSerializationWriter {
    writer: SerializationWriter,
}
#[allow(dead_code)]
impl VersionedSerializationWriter {
    pub fn new<V:PartialEq+Serializable>(version:V) -> VersionedSerializationWriter {
        let mut vsw = VersionedSerializationWriter {
            writer: SerializationWriter::new()
        };
        vsw.write(version);
        vsw
    }

    pub fn data(&self) -> Vec<u8> {self.writer.data()}
    pub fn write<S>(&mut self, s:S) where S:Serializable {s.write(&mut self.writer)}
}



#[allow(unused_imports)]
mod test {
    use super::{VersionedSerializationReader, VersionedSerializationWriter};

    #[test]
    fn reader_test() {
        let data:Vec<u8> = [0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x5, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x68, 0x65, 0x6c, 0x6c, 0x6f].to_vec();
        let mut reader:VersionedSerializationReader<usize> = VersionedSerializationReader::new(data.clone());
        let hello = reader.read(1, "goodbye".to_owned());
        assert_eq!(hello, "hello".to_owned());

        let mut reader:VersionedSerializationReader<usize> = VersionedSerializationReader::new(data.clone());
        let hello = reader.read(2, "goodbye".to_owned());
        assert_eq!(hello, "goodbye".to_owned());
    }
}