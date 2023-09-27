//! Database de l'ICOM
//!
//! La database est une zone de 32768 mots dont le contenu peut être accédé via une
//! `address` (adresse MODBUS en `u16`) ou via un `IdTag` (zone+tag+indices).
//!
//! En interne, la database est un `vec<u8>` de 2 * 32736 bytes où les données sont encodées
//! en 'big endian'.
//!

use std::collections::HashMap;
use std::fmt;
use std::fs::read_to_string;

mod database_csv;

mod id_tag;
pub use id_tag::IdTag;

mod tag;
pub use tag::Tag;

mod database_bool;
mod database_u8;

/// Database de l'ICOM
#[derive(Debug)]
pub struct Database {
    /// Table `u8` de la table MODBUS
    /// L'adresse 'mot' (`u16`) dans cette table correspond aux 2 bytes consécutifs à l'offset 2 * addr et 2 * addr + 1
    /// avec un encodage 'big endian'.
    vec_u8: Vec<u8>,

    /// Correspondances 'address MODBUS' -> `IdTag`
    /// TODO utile ?
    hash_address: HashMap<u16, IdTag>,

    /// Correspondances `IdTag` -> Information du `Tag`
    hash_tag: HashMap<IdTag, Tag>,
}

impl Default for Database {
    fn default() -> Self {
        Self {
            vec_u8: [0_u8; 2 * 0x8000].to_vec(),
            hash_address: HashMap::new(),
            hash_tag: HashMap::new(),
        }
    }
}

impl fmt::Display for Database {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ret = String::new();
        for (id_tag, tag) in &self.hash_tag {
            ret += &format!("@{:04X}: {}\n", tag.address, id_tag);
        }
        write!(f, "{ret}")
    }
}

impl Database {
    /// Construction depuis un fichier datafile*.csv
    #[allow(dead_code)]
    pub fn from_file(filename: &str) -> Self {
        let mut db = Database::default();

        match read_to_string(filename) {
            Ok(content) => {
                for (n, line) in content.lines().enumerate() {
                    match database_csv::from_line_csv(line) {
                        Ok(option_tag) => {
                            if let Some((address, id_tag, tag)) = option_tag {
                                db.add_tag(address, &id_tag, &tag);
                            }
                        }
                        Err(msg) => {
                            eprintln!(
                                "\nErreur fichier '{}', line {} : {}\n",
                                filename,
                                n + 1,
                                msg
                            );
                            std::process::exit(1);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("\nErreur fichier '{filename}' : {e}\n");
                std::process::exit(1);
            }
        }

        db
    }

    /// Ajoute un tag à une adresse dans la database
    #[allow(dead_code)]
    pub fn add_tag(&mut self, address: u16, id_tag: &IdTag, tag: &Tag) {
        let mut tag = tag.clone();
        tag.address = address;
        tag.id_tag = id_tag.clone();
        self.hash_address.insert(address, id_tag.clone());
        self.hash_tag.insert(id_tag.clone(), tag);
    }

    /// Extrait un tag (non mutable) de la database selon son `id_tag`
    #[allow(dead_code)]
    pub fn get_tag_from_id_tag(&self, id_tag: &IdTag) -> Option<&Tag> {
        self.hash_tag.get(id_tag)
    }

    /// Extrait un tag (non mutable) de la database selon son `address`
    #[allow(dead_code)]
    pub fn get_tag_from_address(&self, address: u16) -> Option<&Tag> {
        let option_id_tag = self.hash_address.get(&address);
        match option_id_tag {
            Some(id_tag) => self.hash_tag.get(id_tag),
            None => None,
        }
    }

    /// Extrait un tag mutable de la database selon son `id_tag`
    #[allow(dead_code)]
    pub fn get_mut_tag_from_id_tag(&mut self, id_tag: &IdTag) -> Option<&mut Tag> {
        self.hash_tag.get_mut(id_tag)
    }

    /// Extrait un tag mutable de la database selon son `address`
    #[allow(dead_code)]
    pub fn get_mut_tag_from_address(&mut self, address: u16) -> Option<&mut Tag> {
        let option_id_tag = self.hash_address.get(&address);
        match option_id_tag {
            Some(id_tag) => self.hash_tag.get_mut(id_tag),
            None => None,
        }
    }

    /// Extrait un `Vec<u8>` de la database selon son `address`
    pub fn get_vec_u8_from_address(&self, addr: u16, nb_u8: usize) -> Vec<u8> {
        let mut ret = vec![];
        let addr = addr as usize;
        for n in 2 * addr..2 * addr + nb_u8 {
            ret.push(self.vec_u8[n]);
        }
        ret
    }

    /// Extrait un `Vec<u8>` mutable de la database selon son `address`
    pub fn get_mut_vec_u8_from_address(&mut self, addr: u16, nb_u8: usize) -> &mut [u8] {
        let addr = addr as usize;
        &mut self.vec_u8[2 * addr..2 * addr + nb_u8]
    }

    /// Copie un `&[u8]` dans la database selon son 'address
    pub fn set_vec_u8_to_address(&mut self, addr: u16, vec_u8: &[u8]) {
        let mut addr = 2 * addr as usize;
        for value in vec_u8 {
            self.vec_u8[addr] = *value;
            addr += 1;
        }
    }
}
