use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct ReflectPath<'a>(VecDeque<&'a str>);
impl<'a> ReflectPath<'a> {
    pub fn new(s: &'a str) -> Self {
        Self(s.split(".").filter(|s| !s.is_empty()).collect())
    }

    pub fn has_next(&self) -> bool { !self.0.is_empty() }
}
impl<'a> Iterator for ReflectPath <'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }
}

impl<'a> From<&'a str> for ReflectPath<'a> {
    fn from(value: &'a str) -> Self {
        Self::new(value)
    }
}

impl<'a> From<&'a String> for ReflectPath<'a> {
    fn from(value: &'a String) -> Self {
        Self::new(value)
    }
}

