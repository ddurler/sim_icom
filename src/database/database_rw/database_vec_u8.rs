//! Accès aux données au format `Vec_u8<u8>` dans la [`Database`]
//!
//! Le format `Vec<u8>` est la structure sous-jacente de la [`Database`]; Aussi la majorité
//! des primitives pour ce format est défini dans `super::mod.rs`

#[cfg(test)]
use super::{Tag, ID_ANONYMOUS_USER};

use super::{Database, IdTag, IdUser};

impl Database {
    // Getter selon [`WordAddress`]
    // Voir `get_vec_u8_from_word_address`

    // Setter selon [`WordAddress`]
    // Voir `set_vec_u8_to_word_address`

    /// Getter selon l'[`IdTag`]
    #[allow(dead_code)]
    pub fn get_vec_u8_from_id_tag(&self, id_user: IdUser, id_tag: IdTag, width: usize) -> Vec<u8> {
        match self.get_tag_from_id_tag(id_tag) {
            Some(id_tag) => self.get_vec_u8_from_word_address(id_user, id_tag.word_address, width),
            None => vec![],
        }
    }

    // Setter selon l'[`IdTag`]
    // Voir `set_vec_u8_to_id_tag`
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
    fn test_address_vec_u8() {
        let mut db = Database::default();
        let (addr, _) = test_setup(&mut db);

        assert_eq!(
            db.get_vec_u8_from_word_address(ID_ANONYMOUS_USER, addr, 4),
            vec![0, 0, 0, 0]
        );

        let value = vec![0x01_u8, 0x02_u8, 0x03_u8, 0x04_u8];
        db.set_vec_u8_to_word_address(ID_ANONYMOUS_USER, addr, &value);
        assert_eq!(
            db.get_vec_u8_from_word_address(ID_ANONYMOUS_USER, addr, 6),
            vec![0x01_u8, 0x02_u8, 0x03_u8, 0x04_u8, 0, 0]
        );
        assert_eq!(
            db.get_vec_u8_from_word_address(ID_ANONYMOUS_USER, addr, 4),
            vec![0x01_u8, 0x02_u8, 0x03_u8, 0x04_u8]
        );
        assert_eq!(
            db.get_vec_u8_from_word_address(ID_ANONYMOUS_USER, addr, 2),
            vec![0x01_u8, 0x02_u8]
        );
    }

    #[test]
    fn test_id_tag_vec_u8() {
        let mut db = Database::default();
        let (_, id_tag) = test_setup(&mut db);

        assert_eq!(
            db.get_vec_u8_from_id_tag(ID_ANONYMOUS_USER, id_tag, 4),
            vec![0, 0, 0, 0]
        );

        let value = vec![0x01_u8, 0x02_u8, 0x03_u8, 0x04_u8];
        db.set_vec_u8_to_id_tag(ID_ANONYMOUS_USER, id_tag, &value);
        assert_eq!(
            db.get_vec_u8_from_id_tag(ID_ANONYMOUS_USER, id_tag, 6),
            vec![0x01_u8, 0x02_u8, 0x03_u8, 0x04_u8, 0, 0]
        );
        assert_eq!(
            db.get_vec_u8_from_id_tag(ID_ANONYMOUS_USER, id_tag, 4),
            vec![0x01_u8, 0x02_u8, 0x03_u8, 0x04_u8]
        );
        assert_eq!(
            db.get_vec_u8_from_id_tag(ID_ANONYMOUS_USER, id_tag, 2),
            vec![0x01_u8, 0x02_u8]
        );
    }
}
