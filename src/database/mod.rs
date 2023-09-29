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
use std::fs::File;
use std::io::Read;

mod t_format;
pub use t_format::TFormat;

mod t_value;
pub use t_value::TValue;

mod database_csv;

mod id_tag;
pub use id_tag::IdTag;

mod tag;
pub use tag::Tag;

mod database_rw;

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
        let mut addrs: Vec<u16> = self.hash_address.keys().copied().collect();
        addrs.sort_unstable();
        for addr in addrs {
            if let Some(tag) = self.get_tag_from_address(addr) {
                let t_value = self.get_t_value_from_tag(tag);
                let unity = tag.unity.clone();
                ret += &format!("{tag} = {t_value} {unity}\n");
            }
        }
        write!(f, "{ret}")
    }
}

impl Database {
    /// Construction depuis un fichier datafile*.csv
    #[allow(dead_code)]
    pub fn from_file(filename: &str) -> Self {
        let mut db = Database::default();

        // Il se peut que le fichier issu de Windows ne contienne pas que de l'UTF-8...
        // Aussi on le 'parse' utf8_lossy....
        let mut file = match File::open(filename) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("\nErreur ouverture du fichier '{filename}' : {e}\n");
                std::process::exit(1);
            }
        };
        let mut buf = vec![];
        match file.read_to_end(&mut buf) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("\nErreur lecture du fichier '{filename}' : {e}\n");
                std::process::exit(1);
            }
        };
        let contents: String = String::from_utf8_lossy(&buf).into();

        for (n, line) in contents.lines().enumerate() {
            match database_csv::from_line_csv(line) {
                Ok(option_tag) => {
                    if let Some(tag) = option_tag {
                        // Ajout du tag dans la liste des tags connus
                        db.add_tag(&tag);

                        // Valeur par défaut ?
                        if !tag.default_value.is_empty() {
                            db.set_value(&tag, &tag.default_value);
                        }
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

        db
    }

    /// Ajoute un tag à une adresse dans la database
    pub fn add_tag(&mut self, tag: &Tag) {
        let tag = tag.clone();
        let address = tag.address;
        let id_tag = tag.id_tag.clone();
        self.hash_address.insert(address, id_tag.clone());
        self.hash_tag.insert(id_tag.clone(), tag);
    }

    /// Initialise la database avec une valeur (String) par défaut
    pub fn set_value(&mut self, tag: &Tag, value: &str) {
        let addr = tag.address;
        match tag.t_format {
            TFormat::Bool => {
                if let Ok(value) = value.parse::<bool>() {
                    self.set_bool_to_address(addr, value);
                }
            }
            TFormat::U8 => {
                if let Ok(value) = value.parse::<u8>() {
                    self.set_u8_to_address(addr, value);
                }
            }
            TFormat::I8 => {
                if let Ok(value) = value.parse::<i8>() {
                    self.set_i8_to_address(addr, value);
                }
            }
            TFormat::U16 => {
                if let Ok(value) = value.parse::<u16>() {
                    self.set_u16_to_address(addr, value);
                }
            }
            TFormat::I16 => {
                if let Ok(value) = value.parse::<i16>() {
                    self.set_i16_to_address(addr, value);
                }
            }
            TFormat::U32 => {
                if let Ok(value) = value.parse::<u32>() {
                    self.set_u32_to_address(addr, value);
                }
            }
            TFormat::I32 => {
                if let Ok(value) = value.parse::<i32>() {
                    self.set_i32_to_address(addr, value);
                }
            }
            TFormat::U64 => {
                if let Ok(value) = value.parse::<u64>() {
                    self.set_u64_to_address(addr, value);
                }
            }
            TFormat::I64 => {
                if let Ok(value) = value.parse::<i64>() {
                    self.set_i64_to_address(addr, value);
                }
            }
            TFormat::F32 => {
                if let Ok(value) = value.parse::<f32>() {
                    self.set_f32_to_address(addr, value);
                }
            }
            TFormat::F64 => {
                if let Ok(value) = value.parse::<f64>() {
                    self.set_f64_to_address(addr, value);
                }
            }
            TFormat::String(width) => {
                let value = if value.len() > width {
                    // Tronque si trop long
                    // /!\ format! ne le fait pas...
                    value[..width].to_string()
                } else {
                    format!("{value:width$}")
                };
                self.set_string_to_address(addr, &value);
            }
            TFormat::Unknown => (),
        }
    }

    /// Extrait une valeur selon le tag
    pub fn get_t_value_from_tag(&self, tag: &Tag) -> TValue {
        let addr = tag.address;
        match tag.t_format {
            TFormat::Bool => TValue::Bool(self.get_bool_from_address(addr)),
            TFormat::U8 => TValue::U8(self.get_u8_from_address(addr)),
            TFormat::I8 => TValue::I8(self.get_i8_from_address(addr)),
            TFormat::U16 => TValue::U16(self.get_u16_from_address(addr)),
            TFormat::I16 => TValue::I16(self.get_i16_from_address(addr)),
            TFormat::U32 => TValue::U32(self.get_u32_from_address(addr)),
            TFormat::I32 => TValue::I32(self.get_i32_from_address(addr)),
            TFormat::U64 => TValue::U64(self.get_u64_from_address(addr)),
            TFormat::I64 => TValue::I64(self.get_i64_from_address(addr)),
            TFormat::F32 => TValue::F32(self.get_f32_from_address(addr)),
            TFormat::F64 => TValue::F64(self.get_f64_from_address(addr)),
            TFormat::String(width) => {
                TValue::String(width, self.get_string_from_address(addr, width))
            }
            TFormat::Unknown => TValue::String(3, "???".to_string()),
        }
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
