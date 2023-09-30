//! Accès aux données au format `i32` dans la [`Database`]

#[cfg(test)]
use super::Tag;
use super::{Database, IdTag, WordAddress};

impl Database {
    /// Getter selon [`WordAddress`]
    #[allow(dead_code)]
    pub fn get_i32_from_word_address(&self, word_address: WordAddress) -> i32 {
        let vec_u8 = self.get_vec_u8_from_word_address(word_address, 4);
        let vec_u8: [u8; 4] = vec_u8.try_into().unwrap();
        i32::from_be_bytes(vec_u8)
    }

    /// Setter selon [`WordAddress`]
    #[allow(dead_code)]
    pub fn set_i32_to_word_address(&mut self, word_address: WordAddress, value: i32) {
        let vec_u8 = value.to_be_bytes();
        self.set_vec_u8_to_word_address(word_address, &vec_u8);
    }

    /// Getter selon l'[`IdTag`]
    #[allow(dead_code)]
    pub fn get_i32_from_id_tag(&self, id_tag: &IdTag) -> i32 {
        match self.get_tag_from_id_tag(id_tag) {
            Some(id_tag) => self.get_i32_from_word_address(id_tag.word_address),
            None => i32::default(),
        }
    }

    /// Setter selon l'[`IdTag`]
    #[allow(dead_code)]
    pub fn set_i32_to_id_tag(&mut self, id_tag: &IdTag, value: i32) {
        if let Some(id_tag) = self.get_tag_from_id_tag(id_tag) {
            self.set_i32_to_word_address(id_tag.word_address, value);
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
    fn test_address_i32() {
        let mut db = Database::default();
        let (addr, _) = test_setup(&mut db);

        assert_eq!(db.get_i32_from_word_address(addr), i32::default());

        for value in [-1_000_000_i32, -1_000_i32, 0_i32, 1_000_i32, 1_000_000_i32] {
            db.set_i32_to_word_address(addr, value);
            assert_eq!(db.get_i32_from_word_address(addr), value);

            db.set_i32_to_word_address(addr, value + 1);
            assert_eq!(db.get_i32_from_word_address(addr), value + 1);
        }
    }

    #[test]
    fn test_id_tag_i32() {
        let mut db = Database::default();
        let (_, id_tag) = test_setup(&mut db);

        assert_eq!(db.get_i32_from_id_tag(&id_tag), i32::default());

        for value in [-1_000_000_i32, -1_000_i32, 0_i32, 1_000_i32, 1_000_000_i32] {
            db.set_i32_to_id_tag(&id_tag, value);
            assert_eq!(db.get_i32_from_id_tag(&id_tag), value);

            db.set_i32_to_id_tag(&id_tag, value + 1);
            assert_eq!(db.get_i32_from_id_tag(&id_tag), value + 1);
        }
    }
}
