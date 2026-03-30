//! Builder patterns.
use std::collections::HashMap;

pub struct Builder {
    fields: HashMap<String, String>,
}

impl Builder {
    pub fn new() -> Self {
        Self { fields: HashMap::new() }
    }

    pub fn with_field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }

    pub fn build_with<T, F: FnOnce(&HashMap<String, String>) -> T>(self, f: F) -> T {
        f(&self.fields)
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct StateBuilder<T> {
    state: T,
}

impl<T> StateBuilder<T> {
    pub fn new(initial: T) -> Self {
        Self { state: initial }
    }

    pub fn with<F: FnOnce(&mut T)>(mut self, f: F) -> Self {
        f(&mut self.state);
        self
    }

    pub fn build(self) -> T {
        self.state
    }
}
