//! Accès aux données au format bool dans la [`Database`]

#[cfg(test)]
use super::{Tag, ID_ANONYMOUS_USER};

use super::{Database, IdTag, IdUser, WordAddress};

impl Database {
    /// Getter selon [`WordAddress`]
    #[allow(dead_code)]
    pub fn get_bool_from_word_address(&self, id_user: IdUser, word_address: WordAddress) -> bool {
        self.get_vec_u8_from_word_address(id_user, word_address, 1)[0] != 0
    }

    /// Setter selon [`WordAddress`]
    #[allow(dead_code)]
    pub fn set_bool_to_word_address(
        &mut self,
        id_user: IdUser,
        word_address: WordAddress,
        value: bool,
    ) {
        let vec_u8 = vec![u8::from(value)];
        self.set_vec_u8_to_word_address(id_user, word_address, &vec_u8);
    }

    /// Getter selon l'[`IdTag`]
    #[allow(dead_code)]
    pub fn get_bool_from_id_tag(&self, id_user: IdUser, id_tag: IdTag) -> bool {
        match self.get_tag_from_id_tag(id_tag) {
            Some(id_tag) => self.get_bool_from_word_address(id_user, id_tag.word_address),
            None => bool::default(),
        }
    }

    /// Setter selon l'[`IdTag`]
    #[allow(dead_code)]
    pub fn set_bool_to_id_tag(&mut self, id_user: IdUser, id_tag: IdTag, value: bool) {
        if let Some(id_tag) = self.get_tag_from_id_tag(id_tag) {
            self.set_bool_to_word_address(id_user, id_tag.word_address, value);
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
    fn test_address_bool() {
        let mut db = Database::default();
        let (addr, _) = test_setup(&mut db);

        assert_eq!(
            db.get_bool_from_word_address(ID_ANONYMOUS_USER, addr),
            bool::default()
        );

        db.set_bool_to_word_address(ID_ANONYMOUS_USER, addr, true);
        assert!(db.get_bool_from_word_address(ID_ANONYMOUS_USER, addr));

        db.set_bool_to_word_address(ID_ANONYMOUS_USER, addr, false);
        assert!(!db.get_bool_from_word_address(ID_ANONYMOUS_USER, addr));
    }

    #[test]
    fn test_id_tag_bool() {
        let mut db = Database::default();
        let (_, id_tag) = test_setup(&mut db);

        assert_eq!(
            db.get_bool_from_id_tag(ID_ANONYMOUS_USER, id_tag),
            bool::default()
        );

        db.set_bool_to_id_tag(ID_ANONYMOUS_USER, id_tag, true);
        assert!(db.get_bool_from_id_tag(ID_ANONYMOUS_USER, id_tag));

        db.set_bool_to_id_tag(ID_ANONYMOUS_USER, id_tag, false);
        assert!(!db.get_bool_from_id_tag(ID_ANONYMOUS_USER, id_tag));
    }
}
