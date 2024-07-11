use crate::prelude::*;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ReplayAction {
    Press(KeyPress),
    Release(KeyPress),
    MousePos(f32, f32)
}
impl Serializable for ReplayAction {
    fn read(sr:&mut SerializationReader) -> SerializationResult<Self> {
        sr.push_parent("ReplayAction");

        let a = Ok(match sr.read::<u8>("id")? {
            0 => ReplayAction::Press(sr.read("press")?),
            1 => ReplayAction::Release(sr.read("release")?),
            2 => ReplayAction::MousePos(sr.read("x")?, sr.read("y")?),
            _ => panic!("error reading replay frame type")
        });
        sr.pop_parent();

        a
    }

    fn write(&self, sw:&mut SerializationWriter) {
        match self {
            ReplayAction::Press(k) => {
                sw.write::<u8>(&0);
                sw.write(k);
            }
            ReplayAction::Release(k) => {
                sw.write::<u8>(&1);
                sw.write(k);
            }
            ReplayAction::MousePos(x, y) => {
                sw.write::<u8>(&2);
                sw.write(x);
                sw.write(y);
            }
        }
    }
}
