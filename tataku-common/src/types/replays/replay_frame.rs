use crate::prelude::*;

#[derive(Reflect)]
#[derive(Copy, Clone, Debug)]
#[derive(Serialize, Deserialize)]
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
        sr.push_parent("ReplayFrame");

        let a = Ok(Self {
            time: sr.read("time")?,
            action: sr.read("action")?,
        });

        sr.pop_parent();
        a
    }

    fn write(&self, sw: &mut SerializationWriter) {
        sw.write(&self.time);
        sw.write(&self.action);
    }
}
