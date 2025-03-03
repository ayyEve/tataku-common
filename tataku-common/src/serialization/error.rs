use crate::prelude::*;
use std::{ 
    num::ParseIntError, 
    string::FromUtf8Error
};

pub type SerializationResult<S> = Result<S, SerializationError>;

#[derive(Clone, Debug)]
pub struct SerializationError {
    pub inner: SerializationErrorEnum,
    pub stack: Vec<StackData>,
}
impl SerializationError {
    pub fn with_stack(mut self, stack: Vec<StackData>) -> Self {
        self.stack = stack;
        self
    }

    pub fn format_stack(&self) -> String {
        const INDENT: &str = "   ";
        self.stack.iter()
            .map(|StackData { depth, name, entries }| 
                format!(
                    "{}{name}\n{}", 
                    INDENT.repeat(*depth), 
                    entries.iter().map(|e| format!("{}-> {e}", INDENT.repeat(*depth + 1)))
                    .collect::<Vec<_>>().join("\n")
                )
            )
            .collect::<Vec<_>>().join("\n")
    }
}
impl From<SerializationErrorEnum> for SerializationError {
    fn from(value: SerializationErrorEnum) -> Self {
        Self {
            inner: value,
            stack: Vec::new()
        }
    }
}
impl From<FromUtf8Error> for SerializationError {
    fn from(utf8err: FromUtf8Error) -> Self {
        Self {
            inner: SerializationErrorEnum::FromUtf8Error(utf8err),
            stack: Vec::new()
        }
    }
}
impl From<ParseIntError> for SerializationError {
    fn from(interr: ParseIntError) -> Self {
        Self {
            inner: SerializationErrorEnum::ParseIntError(interr),
            stack: Vec::new()
        }
    }
}

impl PartialEq for SerializationError {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl Eq for SerializationError {}

impl core::fmt::Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}, stack: {}", self.inner, self.format_stack())
    }
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SerializationErrorEnum {
    OutOfBounds,
    FromUtf8Error(FromUtf8Error),
    ParseIntError(ParseIntError),
}
