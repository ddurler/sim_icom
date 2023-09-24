//! Database de l'ICOM

use std::collections::HashMap;

mod id_tag;
pub use id_tag::IdTag;

mod tag;
pub use tag::Tag;

mod t_value;
pub use t_value::TValue;

/// Database de l'ICOM
#[derive(Debug, Default)]
pub struct Database {
    hash_tag: HashMap<IdTag, Tag>,
}

impl Database {
    pub fn push(&mut self, id_tag: &IdTag, tag: &Tag) {
        self.hash_tag.insert(id_tag.clone(), tag.clone());
    }

    pub fn get(&self, id_tag: &IdTag) -> Option<&Tag> {
        self.hash_tag.get(id_tag)
    }

    pub fn get_t_value(&self, id_tag: &IdTag) -> Option<&TValue> {
        match self.hash_tag.get(id_tag) {
            Some(tag) => Some(&tag.t_value),
            None => None,
        }
    }

    pub fn get_mut(&mut self, id_tag: &IdTag) -> Option<&mut Tag> {
        self.hash_tag.get_mut(id_tag)
    }

    pub fn get_mut_t_value(&mut self, id_tag: &IdTag) -> Option<&mut TValue> {
        match self.hash_tag.get_mut(id_tag) {
            Some(tag) => Some(&mut tag.t_value),
            None => None,
        }
    }
}
