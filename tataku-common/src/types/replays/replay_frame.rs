use crate::prelude::*;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ReplayFrame {
    pub time: f32,
    pub action: ReplayAction,
}
impl ReplayFrame {
    pub fn new(time: f32, action: ReplayAction) -> Self {
        Self {
            time,
            action
        }
    }
}

impl Serializable for ReplayFrame {
    fn read(sr: &mut SerializationReader) -> SerializationResult<Self> {
        Ok(Self {
            time: sr.read()?,
            action: sr.read()?,
        })
    }

    fn write(&self, sw: &mut SerializationWriter) {
        sw.write(&self.time);
        sw.write(&self.action);
    }
}