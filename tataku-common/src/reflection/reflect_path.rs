use std::collections::VecDeque;

#[derive(Clone)]
pub struct ReflectPath<'a>(VecDeque<&'a str>);
impl<'a> ReflectPath<'a> {
    pub fn new(s: &'a str) -> Self {
        Self(s.split(".").filter(|s| !s.is_empty()).collect())
    }

    pub fn next(&mut self) -> Option<&'a str> { self.0.pop_front() }
    pub fn has_next(&self) -> bool { !self.0.is_empty() }
}

impl<'a> From<&'a str> for ReflectPath<'a> {
    fn from(value: &'a str) -> Self {
        Self::new(value)
    }
}
