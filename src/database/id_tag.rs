//! Identificateur pour référencer un `Tag` de la database (zone + `num_tag` + indices)

use std::fmt;

/// Référence unique d'un `Tag` de la database (zone +  `num_tag` + indices)
#[derive(Copy, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct IdTag {
    pub zone: u8,
    pub num_tag: u16,
    pub indice_0: u8,
    pub indice_1: u8,
    pub indice_2: u8,
}

impl fmt::Display for IdTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}/{:04X}:{:02X}:{:02X}:{:02X}",
            self.zone, self.num_tag, self.indice_0, self.indice_1, self.indice_2
        )
    }
}

impl IdTag {
    pub fn new(zone: u8, tag: u16, indices: [u8; 3]) -> Self {
        Self {
            zone,
            num_tag: tag,
            indice_0: indices[0],
            indice_1: indices[1],
            indice_2: indices[2],
        }
    }
}
