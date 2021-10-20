pub mod compare;
pub mod query;
pub mod search;

use std::collections::HashMap;

use rustdoc_types::Crate;

#[derive(Debug, Default)]
pub struct Index {
    pub crates: HashMap<String, Crate>,
}
