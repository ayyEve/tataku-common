use crate::prelude::*;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ReplayAction {
    Press(KeyPress),
    Release(KeyPress),
    MousePos(f32, f32)
}
impl Serializable for ReplayAction {
    fn read(sr:&mut SerializationReader) -> SerializationResult<Self> {
        Ok(match sr.read_u8()? {
            0 => ReplayAction::Press(sr.read()?),
            1 => ReplayAction::Release(sr.read()?),
            2 => ReplayAction::MousePos(sr.read()?, sr.read()?),
            _ => panic!("error reading replay frame type")
        })
    }

    fn write(&self, sw:&mut SerializationWriter) {
        match self {
            ReplayAction::Press(k) => {
                sw.write_u8(0);
                sw.write(k);
            }
            ReplayAction::Release(k) => {
                sw.write_u8(1);
                sw.write(k);
            }
            ReplayAction::MousePos(x, y) => {
                sw.write_u8(2);
                sw.write(x);
                sw.write(y);
            }
        }
    }
}
