//! Donnée atomique de la database

use super::IdTag;

/// Donnée atomique détenue dans la database
#[derive(Clone, Debug, Default)]
pub struct Tag {
    /// Adresse MODBUS du tag
    pub address: u16,

    /// IdTag du tag
    pub id_tag: IdTag,

    /// true s'il s'agit d'un tag interne pour l'usage de l'ICOM
    pub is_internal: bool,
}
