//! Accès aux données au format `i16` dans la database

#[cfg(test)]
use super::Tag;
use super::{Database, IdTag};

impl Database {
    /// Getter selon l'adresse MODBUS u16
    #[allow(dead_code)]
    pub fn get_i16_from_address(&self, addr: u16) -> i16 {
        let vec_u8 = self.get_vec_u8_from_address(addr, 2);
        let vec_u8: [u8; 2] = vec_u8.try_into().unwrap();
        i16::from_be_bytes(vec_u8)
    }

    /// Setter selon l'adresse MODBUS u16
    #[allow(dead_code)]
    pub fn set_i16_to_address(&mut self, addr: u16, value: i16) {
        let vec_u8 = value.to_be_bytes();
        self.set_vec_u8_to_address(addr, &vec_u8);
    }

    /// Getter selon l'`IdTag`
    #[allow(dead_code)]
    pub fn get_i16_from_id_tag(&self, id_tag: &IdTag) -> i16 {
        match self.get_tag_from_id_tag(id_tag) {
            Some(id_tag) => self.get_i16_from_address(id_tag.address),
            None => i16::default(),
        }
    }

    /// Setter selon l'`IdTag`
    #[allow(dead_code)]
    pub fn set_i16_to_id_tag(&mut self, id_tag: &IdTag, value: i16) {
        if let Some(id_tag) = self.get_tag_from_id_tag(id_tag) {
            self.set_i16_to_address(id_tag.address, value);
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
            address,
            ..Default::default()
        };
        db.add_tag(&tag);
        (address, id_tag)
    }

    #[test]
    fn test_address_i16() {
        let mut db = Database::default();
        let (addr, _) = test_setup(&mut db);

        assert_eq!(db.get_i16_from_address(addr), i16::default());

        for value in [-10_000_i16, -1_000_i16, 0_i16, 1_000_i16, 10_000_i16] {
            db.set_i16_to_address(addr, value);
            assert_eq!(db.get_i16_from_address(addr), value);

            db.set_i16_to_address(addr, value + 1);
            assert_eq!(db.get_i16_from_address(addr), value + 1);
        }
    }

    #[test]
    fn test_id_tag_i16() {
        let mut db = Database::default();
        let (_, id_tag) = test_setup(&mut db);

        assert_eq!(db.get_i16_from_id_tag(&id_tag), i16::default());

        for value in [-10_000_i16, -1_000_i16, 0_i16, 1_000_i16, 10_000_i16] {
            db.set_i16_to_id_tag(&id_tag, value);
            assert_eq!(db.get_i16_from_id_tag(&id_tag), value);

            db.set_i16_to_id_tag(&id_tag, value + 1);
            assert_eq!(db.get_i16_from_id_tag(&id_tag), value + 1);
        }
    }
}
