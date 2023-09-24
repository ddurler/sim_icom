//! Identificateur pour référencer un tag de la database (zone + tag + indices)

#[derive(Clone, Copy, Debug)]
pub struct IdTag {
    zone: u8,
    tag: u16,
    indice_0: u8,
    indice_1: u8,
    indice_2: u8,
}
