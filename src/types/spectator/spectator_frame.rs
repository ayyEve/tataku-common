use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct SpectatorFrame {
    pub time: f32,
    pub action: SpectatorAction,
}
impl SpectatorFrame {
    pub fn new(time: f32, action: SpectatorAction) -> Self {
        Self {
            time,
            action
        }
    }
}
impl Serializable for SpectatorFrame {
    fn read(sr:&mut SerializationReader) -> SerializationResult<Self> where Self: Sized {
        Ok(Self {
            time: sr.read()?,
            action: sr.read()?,
        })
    }
    fn write(&self, sw:&mut SerializationWriter) {
        sw.write(&self.time);
        sw.write(&self.action);
    }
}
