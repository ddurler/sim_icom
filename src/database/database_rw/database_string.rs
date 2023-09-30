//! Accès aux données au format `String` dans la [`Database`]

#[cfg(test)]
use super::Tag;
use super::{Database, IdTag, WordAddress};

impl Database {
    /// Getter selon [`WordAddress`]
    #[allow(dead_code)]
    pub fn get_string_from_word_address(&self, word_address: WordAddress, width: usize) -> String {
        let vec_u8 = self.get_vec_u8_from_word_address(word_address, width);
        let t: String = String::from_utf8_lossy(&vec_u8).into();
        t
    }

    /// Setter selon [`WordAddress`]
    #[allow(dead_code)]
    pub fn set_string_to_word_address(&mut self, word_address: WordAddress, value: &str) {
        let vec_u8 = value.as_bytes().to_vec();
        self.set_vec_u8_to_word_address(word_address, &vec_u8);
    }

    /// Getter selon l'[`IdTag`]
    #[allow(dead_code)]
    pub fn get_string_from_id_tag(&self, id_tag: &IdTag, width: usize) -> String {
        match self.get_tag_from_id_tag(id_tag) {
            Some(id_tag) => self.get_string_from_word_address(id_tag.word_address, width),
            None => String::default(),
        }
    }

    /// Setter selon l'[`IdTag`]
    #[allow(dead_code)]
    pub fn set_string_to_id_tag(&mut self, id_tag: &IdTag, value: &str) {
        if let Some(id_tag) = self.get_tag_from_id_tag(id_tag) {
            self.set_string_to_word_address(id_tag.word_address, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Création d'une database de test
    fn test_setup(db: &mut Database) -> (u16, IdTag) {
        let address: u16 = 0x1234;
        let id_tag = IdTag::default();
        let tag = Tag {
            word_address: address,
            ..Default::default()
        };
        db.add_tag(&tag);
        (address, id_tag)
    }

    #[test]
    fn test_address_string() {
        let mut db = Database::default();
        let (addr, _) = test_setup(&mut db);

        assert_eq!(db.get_string_from_word_address(addr, 4), "\0\0\0\0");

        let value = "TOTO";
        db.set_string_to_word_address(addr, value);
        assert_eq!(db.get_string_from_word_address(addr, 6), "TOTO\0\0");
        assert_eq!(db.get_string_from_word_address(addr, 4), "TOTO");
        assert_eq!(db.get_string_from_word_address(addr, 2), "TO");
    }

    #[test]
    fn test_id_tag_string() {
        let mut db = Database::default();
        let (_, id_tag) = test_setup(&mut db);

        assert_eq!(db.get_string_from_id_tag(&id_tag, 4), "\0\0\0\0");

        let value = "TOTO";
        db.set_string_to_id_tag(&id_tag, value);
        assert_eq!(db.get_string_from_id_tag(&id_tag, 6), "TOTO\0\0");
        assert_eq!(db.get_string_from_id_tag(&id_tag, 4), "TOTO");
        assert_eq!(db.get_string_from_id_tag(&id_tag, 2), "TO");
    }
}
