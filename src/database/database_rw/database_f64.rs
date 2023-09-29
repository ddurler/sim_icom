//! Accès aux données au format `f64` dans la database

#[cfg(test)]
use super::Tag;
use super::{Database, IdTag};

impl Database {
    /// Getter selon l'adresse MODBUS u16
    #[allow(dead_code)]
    pub fn get_f64_from_address(&self, addr: u16) -> f64 {
        let vec_u8 = self.get_vec_u8_from_address(addr, 8);
        let vec_u8: [u8; 8] = vec_u8.try_into().unwrap();
        f64::from_be_bytes(vec_u8)
    }

    /// Setter selon l'adresse MODBUS u16
    #[allow(dead_code)]
    pub fn set_f64_to_address(&mut self, addr: u16, value: f64) {
        let vec_u8 = value.to_be_bytes();
        self.set_vec_u8_to_address(addr, &vec_u8);
    }

    /// Getter selon l'`IdTag`
    #[allow(dead_code)]
    pub fn get_f64_from_id_tag(&self, id_tag: &IdTag) -> f64 {
        match self.get_tag_from_id_tag(id_tag) {
            Some(id_tag) => self.get_f64_from_address(id_tag.address),
            None => f64::default(),
        }
    }

    /// Setter selon l'`IdTag`
    #[allow(dead_code)]
    pub fn set_f64_to_id_tag(&mut self, id_tag: &IdTag, value: f64) {
        if let Some(id_tag) = self.get_tag_from_id_tag(id_tag) {
            self.set_f64_to_address(id_tag.address, value);
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
            address,
            ..Default::default()
        };
        db.add_tag(&tag);
        (address, id_tag)
    }

    #[test]
    fn test_address_f64() {
        let mut db = Database::default();
        let (addr, _) = test_setup(&mut db);

        assert_f64_near!(db.get_f64_from_address(addr), f64::default());

        for value in [-1000.0, 0.0, 1000.0] {
            db.set_f64_to_address(addr, value);
            assert_f64_near!(db.get_f64_from_address(addr), value);

            db.set_f64_to_address(addr, value + 1.0);
            assert_f64_near!(db.get_f64_from_address(addr), value + 1.0);
        }
    }

    #[test]
    fn test_id_tag_f64() {
        let mut db = Database::default();
        let (_, id_tag) = test_setup(&mut db);

        assert_f64_near!(db.get_f64_from_id_tag(&id_tag), f64::default());

        for value in [-1000.0, 0.0, 1000.0] {
            db.set_f64_to_id_tag(&id_tag, value);
            assert_f64_near!(db.get_f64_from_id_tag(&id_tag), value);

            db.set_f64_to_id_tag(&id_tag, value + 1.0);
            assert_f64_near!(db.get_f64_from_id_tag(&id_tag), value + 1.0);
        }
    }
}
