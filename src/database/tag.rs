//! Donnée atomique de la database

use std::fmt;

use super::IdTag;
use super::TFormat;
use super::WordAddress;

/// Donnée atomique détenue dans la database
#[derive(Clone, Debug, Default)]
pub struct Tag {
    /// [`WordAddress`] MODBUS du [`Tag`]
    pub word_address: WordAddress,

    /// [`IdTag`] du [`Tag`]
    pub id_tag: IdTag,

    /// true s'il s'agit d'un [`Tag`] interne pour l'usage de l'ICOM
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
        write!(
            f,
            "@{:04X}: {} - {}",
            self.word_address, self.id_tag, self.label
        )
    }
}

impl Tag {
    /// Retourne true si le [`Tag`] (ou une partie du [`Tag`]) utilise cette [`WordAddress`]
    #[allow(dead_code)]
    #[allow(clippy::cast_possible_truncation)]
    pub fn contains_word_address(&self, word_address: WordAddress) -> bool {
        let address_start = self.word_address;
        let nb_word = self.t_format.nb_words() as u16;
        address_start <= word_address && word_address < address_start + nb_word
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag() {
        let mut tag = Tag::default();
        let _ = format!("{tag}");

        // Test un bool à l'adresse 0x0010 (1 word)
        tag.word_address = 0x0010;
        tag.t_format = TFormat::Bool;
        assert!(!tag.contains_word_address(0x000F));
        assert!(tag.contains_word_address(0x0010));
        assert!(!tag.contains_word_address(0x0011));

        // Test un F64 à l'adresse 0x0020 (4 words)
        tag.word_address = 0x0020;
        tag.t_format = TFormat::F64;
        assert!(!tag.contains_word_address(0x001F));
        assert!(tag.contains_word_address(0x0020));
        assert!(tag.contains_word_address(0x0021));
        assert!(tag.contains_word_address(0x0022));
        assert!(tag.contains_word_address(0x0023));
        assert!(!tag.contains_word_address(0x0024));
    }
}
