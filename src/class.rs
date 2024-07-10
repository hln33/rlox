use std::fmt::Display;

#[derive(Clone, Debug)]
pub struct Class {
    name: String,
}

impl Class {
    pub fn new(name: String) -> Class {
        Class { name }
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
