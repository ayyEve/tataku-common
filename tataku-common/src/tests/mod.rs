use crate::prelude::*;


#[derive(Debug, Clone)]
pub(crate) enum RawOrOther<'a, T> {
    Raw(Vec<u8>),
    Other(&'a T),
    OtherOwned(T),
}
impl<'a, T:Serializable> Serializable for RawOrOther<'a, T> {
    fn read(sr: &mut SerializationReader) -> SerializationResult<Self> where Self: Sized {
        T::read(sr).map(|a| Self::OtherOwned(a))
    }
    fn write(&self, sw: &mut SerializationWriter) {
        match self {
            Self::Raw(bytes) => sw.write_raw_bytes(bytes),
            Self::Other(t) => sw.write(*t),
            Self::OtherOwned(t) => sw.write(t),
        }
    }
}
impl<'a, T> From<Vec<u8>> for RawOrOther<'a, T> {
    fn from(value: Vec<u8>) -> Self {
        Self::Raw(value)
    }
}
impl<'a, T> From<&'a T> for RawOrOther<'a, T> {
    fn from(value: &'a T) -> Self {
        Self::Other(value)
    }
}



pub struct VersionedWriter {
    version: u16,
    writer: SerializationWriter
}
impl VersionedWriter {
    pub fn new(version: u16) -> Self { Self { version, writer: SerializationWriter::new() } }
    pub fn data(self) -> Vec<u8> { self.writer.data() }
    pub fn write(&mut self, version: u16, data: &(impl Serializable + core::fmt::Debug), debug_name: &str) {
        self.write_ranged(version.., data, debug_name);
    }

    pub fn write_ranged<S: Serializable + core::fmt::Debug>(&mut self, version: impl std::ops::RangeBounds<u16>, data: &S, debug_name: &str) {
        if !version.contains(&self.version) { return }

        println!("writing {debug_name} ({}) ({data:?})", std::any::type_name::<S>());
        self.writer.write(data);
    }
}

#[tokio::test]
pub async fn test_replays_from_folder() {
    use std::ffi::OsStr;
    let folder = "/tmp/replays";
    use std::io::Write;

    let stdout = std::io::stdout();
    let mut lock = stdout.lock();

    macro_rules! println {
        ($($arg:tt)*) => {
            lock.write_all(format!($($arg)*).as_bytes()).unwrap();
            lock.write_all(b"\n").unwrap();
        }
    }

    let mut entries = tokio::fs::read_dir(folder).await.unwrap();
    while let Ok(Some(file)) = entries.next_entry().await {
        let path = file.path();
        if path.is_file() && path.extension() == Some(OsStr::new("ttkr")) {
            println!("reading replay: {path:?}");

            let bytes = tokio::fs::read(&path).await.unwrap();

            let mut reader = SerializationReader::new(bytes).debug();
            if let Err(e) = Replay::try_read_replay(&mut reader) {
                println!("error reading replay: {e:?}");
            }

        }
    }

}

#[test]
fn test1() {
    // let bytes = std::fs::read("/tmp/replays/4fda112a7401e5b9a379adbaa14d3c5a.ttkr").unwrap();
    let bytes = std::fs::read("/tmp/replays/12eeb05fbfa169060b0f7597f3130057.ttkr").unwrap();

    let mut reader = SerializationReader::new(bytes).debug();
    if let Err(ReplayLoadError::SerializationError(e)) = Replay::try_read_replay(&mut reader) {
        // println!("error reading replay: {e:?}");
        panic!("{e}")
    }
}