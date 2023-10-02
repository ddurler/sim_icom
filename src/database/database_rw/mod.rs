//! Module pour la gestion des différents formats dans la [`Database`]

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
            TFormat::String(width) => {
                let value = if value.len() > width {
                    // Tronque si trop long
                    // /!\ format! ne le fait pas...
                    value[..width].to_string()
                } else {
                    format!("{value:width$}")
                };
                self.set_string_to_word_address(id_user, word_address, &value);
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
            TFormat::String(width) => TValue::String(
                width,
                self.get_string_from_word_address(id_user, word_address, width),
            ),
            TFormat::Unknown => TValue::String(3, "???".to_string()),
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
