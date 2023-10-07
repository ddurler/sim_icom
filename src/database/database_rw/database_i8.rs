//! Accès aux données au format `i8` dans la [`Database`]

#[cfg(test)]
use super::{Tag, ID_ANONYMOUS_USER};

use super::{Database, IdTag, IdUser, WordAddress};

impl Database {
    /// Getter selon [`WordAddress`]
    #[allow(dead_code)]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_wrap)]
    pub fn get_i8_from_word_address(&self, id_user: IdUser, word_address: WordAddress) -> i8 {
        let vec_u8 = self.get_vec_u8_from_word_address(id_user, word_address, 2);
        vec_u8[1] as i8
    }

    /// Setter selon [`WordAddress`]
    #[allow(dead_code)]
    #[allow(clippy::cast_sign_loss)]
    pub fn set_i8_to_word_address(
        &mut self,
        id_user: IdUser,
        word_address: WordAddress,
        value: i8,
    ) {
        let mut vec_u8 = self.get_vec_u8_from_word_address(id_user, word_address, 2);
        vec_u8[1] = value as u8;
        self.set_vec_u8_to_word_address(id_user, word_address, &vec_u8);
    }

    /// Getter selon l'[`IdTag`]
    #[allow(dead_code)]
    pub fn get_i8_from_id_tag(&self, id_user: IdUser, id_tag: IdTag) -> i8 {
        match self.get_tag_from_id_tag(id_tag) {
            Some(id_tag) => self.get_i8_from_word_address(id_user, id_tag.word_address),
            None => i8::default(),
        }
    }

    /// Setter selon l'[`IdTag`]
    #[allow(dead_code)]
    pub fn set_i8_to_id_tag(&mut self, id_user: IdUser, id_tag: IdTag, value: i8) {
        if let Some(id_tag) = self.get_tag_from_id_tag(id_tag) {
            self.set_i8_to_word_address(id_user, id_tag.word_address, value);
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
    fn test_address_i8() {
        let mut db = Database::default();
        let (addr, _) = test_setup(&mut db);

        assert_eq!(
            db.get_i8_from_word_address(ID_ANONYMOUS_USER, addr),
            i8::default()
        );

        for value in [-100_i8, -10_i8, 0_i8, 10_i8, 100_i8] {
            db.set_i8_to_word_address(ID_ANONYMOUS_USER, addr, value);
            assert_eq!(db.get_i8_from_word_address(ID_ANONYMOUS_USER, addr), value);

            db.set_i8_to_word_address(ID_ANONYMOUS_USER, addr, value + 1);
            assert_eq!(
                db.get_i8_from_word_address(ID_ANONYMOUS_USER, addr),
                value + 1
            );
        }
    }

    #[test]
    fn test_id_tag_i8() {
        let mut db = Database::default();
        let (_, id_tag) = test_setup(&mut db);

        assert_eq!(
            db.get_i8_from_id_tag(ID_ANONYMOUS_USER, id_tag),
            i8::default()
        );

        for value in [-100_i8, -10_i8, 0_i8, 10_i8, 100_i8] {
            db.set_i8_to_id_tag(ID_ANONYMOUS_USER, id_tag, value);
            assert_eq!(db.get_i8_from_id_tag(ID_ANONYMOUS_USER, id_tag), value);

            db.set_i8_to_id_tag(ID_ANONYMOUS_USER, id_tag, value + 1);
            assert_eq!(db.get_i8_from_id_tag(ID_ANONYMOUS_USER, id_tag), value + 1);
        }
    }
}
