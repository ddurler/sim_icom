//! Module pour la gestion des différents formats dans la database

use super::{Database, IdTag};

#[cfg(test)]
use super::Tag;

mod database_bool;
mod database_f32;
mod database_f64;
mod database_i16;
mod database_i32;
mod database_i64;
mod database_i8;
mod database_string;
mod database_u16;
mod database_u32;
mod database_u64;
mod database_u8;
