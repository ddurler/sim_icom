//! Database de l'ICOM

mod id_tag;
pub use id_tag::IdTag;

mod tag;
pub use tag::Tag;

mod t_value;
pub use t_value::TValue;

/// Database de l'ICOM
#[derive(Debug)]
pub struct Database {
    pub counter: usize,
}

impl Default for Database {
    fn default() -> Self {
        Self { counter: 1 }
    }
}
