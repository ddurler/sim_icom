//! Donnée atomique de la database

use std::fmt;

use super::IdTag;
use super::TFormat;

/// Donnée atomique détenue dans la database
#[derive(Clone, Debug, Default)]
pub struct Tag {
    /// Adresse MODBUS du tag
    pub address: u16,

    /// IdTag du tag
    pub id_tag: IdTag,

    /// true s'il s'agit d'un tag interne pour l'usage de l'ICOM
    pub is_internal: bool,

    /// Format de la donnée dans la database
    pub t_format: TFormat,

    /// Unité de la grandeur (si existe)
    pub unity: String,

    /// Libellé de la donnée (si défini)
    pub label: String,

    /// true si champ possible en écriture par un client extern
    pub is_write: bool,

    /// Valeur par défaut (au format string)
    pub default_value: String,
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "@{:04X}: {} - {}", self.address, self.id_tag, self.label)
    }
}
