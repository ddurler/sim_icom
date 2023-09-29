//! Accès aux données au format `u32` dans la database

#[cfg(test)]
use super::Tag;
use super::{Database, IdTag};

impl Database {
    /// Getter selon l'adresse MODBUS u16
    #[allow(dead_code)]
    pub fn get_u32_from_address(&self, addr: u16) -> u32 {
        let vec_u8 = self.get_vec_u8_from_address(addr, 4);
        let vec_u8: [u8; 4] = vec_u8.try_into().unwrap();
        u32::from_be_bytes(vec_u8)
    }

    /// Setter selon l'adresse MODBUS u16
    #[allow(dead_code)]
    pub fn set_u32_to_address(&mut self, addr: u16, value: u32) {
        let vec_u8 = value.to_be_bytes();
        self.set_vec_u8_to_address(addr, &vec_u8);
    }

    /// Getter selon l'`IdTag`
    #[allow(dead_code)]
    pub fn get_u32_from_id_tag(&self, id_tag: &IdTag) -> u32 {
        match self.get_tag_from_id_tag(id_tag) {
            Some(id_tag) => self.get_u32_from_address(id_tag.address),
            None => u32::default(),
        }
    }

    /// Setter selon l'`IdTag`
    #[allow(dead_code)]
    pub fn set_u32_to_id_tag(&mut self, id_tag: &IdTag, value: u32) {
        if let Some(id_tag) = self.get_tag_from_id_tag(id_tag) {
            self.set_u32_to_address(id_tag.address, value);
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
    fn test_address_u32() {
        let mut db = Database::default();
        let (addr, _) = test_setup(&mut db);

        assert_eq!(db.get_u32_from_address(addr), u32::default());

        for value in [0_u32, 1_000_u32, 1_000_000_u32] {
            db.set_u32_to_address(addr, value);
            assert_eq!(db.get_u32_from_address(addr), value);

            db.set_u32_to_address(addr, value + 1);
            assert_eq!(db.get_u32_from_address(addr), value + 1);
        }
    }

    #[test]
    fn test_id_tag_u32() {
        let mut db = Database::default();
        let (_, id_tag) = test_setup(&mut db);

        assert_eq!(db.get_u32_from_id_tag(&id_tag), u32::default());

        for value in [0_u32, 1_000_u32, 1_000_000_u32] {
            db.set_u32_to_id_tag(&id_tag, value);
            assert_eq!(db.get_u32_from_id_tag(&id_tag), value);

            db.set_u32_to_id_tag(&id_tag, value + 1);
            assert_eq!(db.get_u32_from_id_tag(&id_tag), value + 1);
        }
    }
}
