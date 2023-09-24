//! Donnée atomique de la database

use super::TValue;

/// Donnée atomique détenue dans la database
#[derive(Clone, Debug, Default)]
pub struct Tag {
    pub t_value: TValue,
}
