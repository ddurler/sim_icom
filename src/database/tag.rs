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
    /// Retourne true si le [`Tag`] (ou une partie du [`Tag`]) utilise cette [`WordAddress`] + `nb_words`
    pub fn contains_word_address_area(&self, word_address: WordAddress, nb_words: usize) -> bool {
        // Tous les calculs se font en usize
        let word_address_start = word_address as usize;
        let word_address_end = word_address_start + nb_words - 1;

        let tag_address_start = self.word_address as usize;
        let tag_nb_words = self.t_format.nb_words();
        let tag_address_end = tag_address_start + tag_nb_words - 1;

        word_address_end >= tag_address_start && word_address_start <= tag_address_end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_contains_word_address_area() {
        let mut tag = Tag::default();
        let _ = format!("{tag}");

        // Test un bool à l'adresse 0x0010 (1 word)
        tag.word_address = 0x0010;
        tag.t_format = TFormat::Bool;
        // avant
        assert!(!tag.contains_word_address_area(0x000F, 1));
        // avant mais empiète...
        assert!(tag.contains_word_address_area(0x000F, 2));
        assert!(tag.contains_word_address_area(0x000F, 3));
        // dedans
        assert!(tag.contains_word_address_area(0x0010, 1));
        // dedans et dépasse
        assert!(tag.contains_word_address_area(0x0010, 2));
        // après
        assert!(!tag.contains_word_address_area(0x0011, 1));
        assert!(!tag.contains_word_address_area(0x0011, 2));

        // Test un F64 à l'adresse 0x0020 (4 words)
        tag.word_address = 0x0020;
        tag.t_format = TFormat::F64;
        // avant
        assert!(!tag.contains_word_address_area(0x001E, 1));
        assert!(!tag.contains_word_address_area(0x001E, 2));
        // avant mais empiète...
        assert!(tag.contains_word_address_area(0x001E, 3));
        assert!(tag.contains_word_address_area(0x001E, 4));
        assert!(tag.contains_word_address_area(0x001E, 5));
        assert!(tag.contains_word_address_area(0x001E, 6));
        assert!(tag.contains_word_address_area(0x001E, 7));
        assert!(tag.contains_word_address_area(0x001E, 8));
        // dedans
        assert!(tag.contains_word_address_area(0x0020, 1));
        assert!(tag.contains_word_address_area(0x0020, 2));
        assert!(tag.contains_word_address_area(0x0020, 3));
        assert!(tag.contains_word_address_area(0x0020, 4));
        assert!(tag.contains_word_address_area(0x0020, 5));
        // dedans et dépasse
        assert!(tag.contains_word_address_area(0x0022, 1));
        assert!(tag.contains_word_address_area(0x0022, 2));
        assert!(tag.contains_word_address_area(0x0022, 3));

        // après
        assert!(!tag.contains_word_address_area(0x0024, 1));
        assert!(!tag.contains_word_address_area(0x0024, 2));
        assert!(!tag.contains_word_address_area(0x0024, 3));
    }
}
