//! Accès aux données au format `f32` dans la [`Database`]

#[cfg(test)]
use super::{Tag, ID_ANONYMOUS_USER};

use super::{Database, IdTag, IdUser, WordAddress};

impl Database {
    /// Getter selon [`WordAddress`]
    #[allow(dead_code)]
    pub fn get_f32_from_word_address(&self, id_user: IdUser, word_address: WordAddress) -> f32 {
        let vec_u8 = self.get_vec_u8_from_word_address(id_user, word_address, 4);
        let vec_u8: [u8; 4] = vec_u8.try_into().unwrap();
        f32::from_be_bytes(vec_u8)
    }

    /// Setter selon [`WordAddress`]
    #[allow(dead_code)]
    pub fn set_f32_to_word_address(
        &mut self,
        id_user: IdUser,
        word_address: WordAddress,
        value: f32,
    ) {
        let vec_u8 = value.to_be_bytes();
        self.set_vec_u8_to_word_address(id_user, word_address, &vec_u8);
    }

    /// Getter selon l'[`IdTag`]
    #[allow(dead_code)]
    pub fn get_f32_from_id_tag(&self, id_user: IdUser, id_tag: IdTag) -> f32 {
        match self.get_tag_from_id_tag(id_tag) {
            Some(id_tag) => self.get_f32_from_word_address(id_user, id_tag.word_address),
            None => f32::default(),
        }
    }

    /// Setter selon l'[`IdTag`]
    #[allow(dead_code)]
    pub fn set_f32_to_id_tag(&mut self, id_user: IdUser, id_tag: IdTag, value: f32) {
        if let Some(id_tag) = self.get_tag_from_id_tag(id_tag) {
            self.set_f32_to_word_address(id_user, id_tag.word_address, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_float_eq::*;

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
    fn test_address_f32() {
        let mut db = Database::default();
        let (addr, _) = test_setup(&mut db);

        assert_f32_near!(
            db.get_f32_from_word_address(ID_ANONYMOUS_USER, addr),
            f32::default()
        );

        for value in [-1000.0, 0.0, 1000.0] {
            db.set_f32_to_word_address(ID_ANONYMOUS_USER, addr, value);
            assert_f32_near!(db.get_f32_from_word_address(ID_ANONYMOUS_USER, addr), value);

            db.set_f32_to_word_address(ID_ANONYMOUS_USER, addr, value + 1.0);
            assert_f32_near!(
                db.get_f32_from_word_address(ID_ANONYMOUS_USER, addr),
                value + 1.0
            );
        }
    }

    #[test]
    fn test_id_tag_f32() {
        let mut db = Database::default();
        let (_, id_tag) = test_setup(&mut db);

        assert_f32_near!(
            db.get_f32_from_id_tag(ID_ANONYMOUS_USER, id_tag),
            f32::default()
        );

        for value in [-1000.0, 0.0, 1000.0] {
            db.set_f32_to_id_tag(ID_ANONYMOUS_USER, id_tag, value);
            assert_f32_near!(db.get_f32_from_id_tag(ID_ANONYMOUS_USER, id_tag), value);

            db.set_f32_to_id_tag(ID_ANONYMOUS_USER, id_tag, value + 1.0);
            assert_f32_near!(
                db.get_f32_from_id_tag(ID_ANONYMOUS_USER, id_tag),
                value + 1.0
            );
        }
    }
}
