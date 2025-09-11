use crate::serialization::*;

pub struct SerializationReader {
    pub(self) data: Vec<u8>,
    pub(self) offset: usize,
    pub(self) stack: Vec<StackData>,
    pub(self) stack_depth: usize,
    pub debug: bool,
}
impl SerializationReader {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
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

    /// read from the data but reset the offset back to where it was before the read
    pub fn peek<R:Serializable>(&mut self, name: impl ToString) -> SerializationResult<R> {
        let offset = self.offset;
        let read = self.read::<R>(name);
        self.offset = offset;
        read
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
