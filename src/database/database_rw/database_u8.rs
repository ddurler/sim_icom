//! Accès aux données au format `u8` dans la database

#[cfg(test)]
use super::Tag;
use super::{Database, IdTag};

impl Database {
    /// Getter selon l'adresse MODBUS u16
    #[allow(dead_code)]
    pub fn get_u8_from_address(&self, addr: u16) -> u8 {
        let vec_u8 = self.get_vec_u8_from_address(addr, 1);
        let vec_u8: [u8; 1] = vec_u8.try_into().unwrap();
        u8::from_be_bytes(vec_u8)
    }

    /// Setter selon l'adresse MODBUS u16
    #[allow(dead_code)]
    pub fn set_u8_to_address(&mut self, addr: u16, value: u8) {
        let vec_u8 = value.to_be_bytes();
        self.set_vec_u8_to_address(addr, &vec_u8);
    }

    /// Getter selon l'`IdTag`
    #[allow(dead_code)]
    pub fn get_u8_from_id_tag(&self, id_tag: &IdTag) -> u8 {
        match self.get_tag_from_id_tag(id_tag) {
            Some(id_tag) => self.get_u8_from_address(id_tag.address),
            None => u8::default(),
        }
    }

    /// Setter selon l'`IdTag`
    #[allow(dead_code)]
    pub fn set_u8_to_id_tag(&mut self, id_tag: &IdTag, value: u8) {
        if let Some(id_tag) = self.get_tag_from_id_tag(id_tag) {
            self.set_u8_to_address(id_tag.address, value);
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
    fn test_address_u8() {
        let mut db = Database::default();
        let (addr, _) = test_setup(&mut db);

        assert_eq!(db.get_u8_from_address(addr), u8::default());

        for value in [0_u8, 10_u8, 100_u8, 200_u8] {
            db.set_u8_to_address(addr, value);
            assert_eq!(db.get_u8_from_address(addr), value);

            db.set_u8_to_address(addr, value + 1);
            assert_eq!(db.get_u8_from_address(addr), value + 1);
        }
    }

    #[test]
    fn test_id_tag_u8() {
        let mut db = Database::default();
        let (_, id_tag) = test_setup(&mut db);

        assert_eq!(db.get_u8_from_id_tag(&id_tag), u8::default());

        for value in [0_u8, 10_u8, 100_u8, 200_u8] {
            db.set_u8_to_id_tag(&id_tag, value);
            assert_eq!(db.get_u8_from_id_tag(&id_tag), value);

            db.set_u8_to_id_tag(&id_tag, value + 1);
            assert_eq!(db.get_u8_from_id_tag(&id_tag), value + 1);
        }
    }
}
