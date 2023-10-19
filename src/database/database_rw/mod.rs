//! Module pour la gestion des différents formats dans la [`Database`]

use crate::t_data::string_to_vec_u8;

#[cfg(test)]
use super::ID_ANONYMOUS_USER;

use super::{Database, IdTag, IdUser, TFormat, TValue, Tag, WordAddress};

mod database_bool;
mod database_f32;
mod database_f64;
mod database_i16;
mod database_i32;
mod database_i64;
mod database_i8;
mod database_string;
mod database_u16;
mod database_u32;
mod database_u64;
mod database_u8;
mod database_vec_u8;

impl Database {
    /// Ecrire la [`Database`] avec une valeur (String) par défaut
    pub fn set_value(&mut self, id_user: IdUser, tag: &Tag, value: &str) {
        let word_address = tag.word_address;
        match tag.t_format {
            TFormat::Bool => {
                if let Ok(value) = value.parse::<bool>() {
                    self.set_bool_to_word_address(id_user, word_address, value);
                }
            }
            TFormat::U8 => {
                if let Ok(value) = value.parse::<u8>() {
                    self.set_u8_to_word_address(id_user, word_address, value);
                }
            }
            TFormat::I8 => {
                if let Ok(value) = value.parse::<i8>() {
                    self.set_i8_to_word_address(id_user, word_address, value);
                }
            }
            TFormat::U16 => {
                if let Ok(value) = value.parse::<u16>() {
                    self.set_u16_to_word_address(id_user, word_address, value);
                }
            }
            TFormat::I16 => {
                if let Ok(value) = value.parse::<i16>() {
                    self.set_i16_to_word_address(id_user, word_address, value);
                }
            }
            TFormat::U32 => {
                if let Ok(value) = value.parse::<u32>() {
                    self.set_u32_to_word_address(id_user, word_address, value);
                }
            }
            TFormat::I32 => {
                if let Ok(value) = value.parse::<i32>() {
                    self.set_i32_to_word_address(id_user, word_address, value);
                }
            }
            TFormat::U64 => {
                if let Ok(value) = value.parse::<u64>() {
                    self.set_u64_to_word_address(id_user, word_address, value);
                }
            }
            TFormat::I64 => {
                if let Ok(value) = value.parse::<i64>() {
                    self.set_i64_to_word_address(id_user, word_address, value);
                }
            }
            TFormat::F32 => {
                if let Ok(value) = value.parse::<f32>() {
                    self.set_f32_to_word_address(id_user, word_address, value);
                }
            }
            TFormat::F64 => {
                if let Ok(value) = value.parse::<f64>() {
                    self.set_f64_to_word_address(id_user, word_address, value);
                }
            }
            TFormat::VecU8(len) => {
                let value = value.as_bytes().to_vec();
                let value = if value.len() >= len {
                    value[..len].to_vec()
                } else {
                    let mut v = value.clone();
                    while v.len() < len {
                        v.push(0);
                    }
                    v
                };
                self.set_vec_u8_to_word_address(id_user, word_address, &value);
            }
            TFormat::Unknown => (),
        }
    }

    /// Extrait une valeur [`TValue`] selon le [`Tag`]
    pub fn get_t_value_from_tag(&self, id_user: IdUser, tag: &Tag) -> TValue {
        let word_address = tag.word_address;
        match tag.t_format {
            TFormat::Bool => TValue::Bool(self.get_bool_from_word_address(id_user, word_address)),
            TFormat::U8 => TValue::U8(self.get_u8_from_word_address(id_user, word_address)),
            TFormat::I8 => TValue::I8(self.get_i8_from_word_address(id_user, word_address)),
            TFormat::U16 => TValue::U16(self.get_u16_from_word_address(id_user, word_address)),
            TFormat::I16 => TValue::I16(self.get_i16_from_word_address(id_user, word_address)),
            TFormat::U32 => TValue::U32(self.get_u32_from_word_address(id_user, word_address)),
            TFormat::I32 => TValue::I32(self.get_i32_from_word_address(id_user, word_address)),
            TFormat::U64 => TValue::U64(self.get_u64_from_word_address(id_user, word_address)),
            TFormat::I64 => TValue::I64(self.get_i64_from_word_address(id_user, word_address)),
            TFormat::F32 => TValue::F32(self.get_f32_from_word_address(id_user, word_address)),
            TFormat::F64 => TValue::F64(self.get_f64_from_word_address(id_user, word_address)),
            TFormat::VecU8(len) => TValue::VecU8(
                len,
                self.get_vec_u8_from_word_address(id_user, word_address, len),
            ),
            TFormat::Unknown => TValue::VecU8(3, string_to_vec_u8("???")),
        }
    }

    /// Extrait un `Vec<u8>` de la [`Database`] selon [`WordAddress`]
    pub fn get_vec_u8_from_word_address(
        &self,
        _id_user: IdUser,
        word_address: WordAddress,
        nb_u8: usize,
    ) -> Vec<u8> {
        let mut ret = vec![];
        let word_address_usize = word_address as usize;
        for n in 2 * word_address_usize..2 * word_address_usize + nb_u8 {
            ret.push(self.vec_u8[n]);
        }

        ret
    }

    /// Copie un `&[u8]` dans la [`Database`] selon [`IdTag`]
    /// (Helper pour le `TValue::String`)
    pub fn set_vec_u8_to_id_tag(&mut self, id_user: IdUser, id_tag: IdTag, value: &[u8]) {
        if let Some(id_tag) = self.get_tag_from_id_tag(id_tag) {
            self.set_vec_u8_to_word_address(id_user, id_tag.word_address, value);
        }
    }

    /// Copie un `&[u8]` dans la [`Database`] selon [`WordAddress`]
    /// Cette fonction est le seul point d'entrée pour modifier le contenu de la [`Database`]
    pub fn set_vec_u8_to_word_address(
        &mut self,
        id_user: IdUser,
        word_address: WordAddress,
        vec_u8: &[u8],
    ) {
        let mut u8_address = 2 * word_address as usize;
        for value in vec_u8 {
            self.vec_u8[u8_address] = *value;
            u8_address += 1;
        }

        // Notification de la mise à jour
        let nb_words = (vec_u8.len() + 1) / 2;
        let tags = self.get_tags_from_word_address_area(word_address, nb_words);
        for tag in tags {
            self.user_write_tag(id_user, &tag);
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_float_eq::*;

    use super::*;

    #[test]
    #[allow(clippy::similar_names)]
    fn test_set_value() {
        let mut db = Database::default();

        // Création d'un tag U16
        let tag_u16 = Tag {
            word_address: 0x0010,
            id_tag: IdTag::new(1, 1, [0, 0, 0]),
            t_format: TFormat::U16,
            ..Default::default()
        };
        db.add_tag(&tag_u16);

        // Création d'un tag I16
        let tag_i16 = Tag {
            word_address: 0x0020,
            id_tag: IdTag::new(2, 2, [0, 0, 0]),
            t_format: TFormat::I16,
            ..Default::default()
        };
        db.add_tag(&tag_i16);

        // Création d'un tag f32
        let tag_f32 = Tag {
            word_address: 0x0030,
            id_tag: IdTag::new(3, 3, [0, 0, 0]),
            t_format: TFormat::F32,
            ..Default::default()
        };
        db.add_tag(&tag_f32);

        // Création d'un tag VecU8(5)
        let tag_vec_u8 = Tag {
            word_address: 0x0040,
            id_tag: IdTag::new(4, 4, [0, 0, 0]),
            t_format: TFormat::VecU8(5),
            ..Default::default()
        };
        db.add_tag(&tag_vec_u8);

        // Init de tag_u16
        db.set_value(ID_ANONYMOUS_USER, &tag_u16, "123");
        assert_eq!(
            db.get_u16_from_id_tag(ID_ANONYMOUS_USER, tag_u16.id_tag),
            123
        );

        // Init de tag_i16
        db.set_value(ID_ANONYMOUS_USER, &tag_i16, "-123");
        assert_eq!(
            db.get_i16_from_id_tag(ID_ANONYMOUS_USER, tag_i16.id_tag),
            -123
        );

        // Init de tag_f32
        db.set_value(ID_ANONYMOUS_USER, &tag_f32, "-123.4");
        assert_f32_near!(
            db.get_f32_from_id_tag(ID_ANONYMOUS_USER, tag_f32.id_tag),
            -123.4
        );

        // Init de tag_vec_u8
        db.set_value(ID_ANONYMOUS_USER, &tag_vec_u8, "TOTO");
        assert_eq!(
            db.get_vec_u8_from_id_tag(ID_ANONYMOUS_USER, tag_vec_u8.id_tag, 5),
            vec![b'T', b'O', b'T', b'O', 0x00]
        );
    }
}
