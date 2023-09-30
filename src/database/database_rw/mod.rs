//! Module pour la gestion des différents formats dans la [`Database`]

use super::{Database, IdTag, TFormat, TValue, Tag, WordAddress};

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
    pub fn set_value(&mut self, tag: &Tag, value: &str) {
        let word_address = tag.word_address;
        match tag.t_format {
            TFormat::Bool => {
                if let Ok(value) = value.parse::<bool>() {
                    self.set_bool_to_word_address(word_address, value);
                }
            }
            TFormat::U8 => {
                if let Ok(value) = value.parse::<u8>() {
                    self.set_u8_to_word_address(word_address, value);
                }
            }
            TFormat::I8 => {
                if let Ok(value) = value.parse::<i8>() {
                    self.set_i8_to_word_address(word_address, value);
                }
            }
            TFormat::U16 => {
                if let Ok(value) = value.parse::<u16>() {
                    self.set_u16_to_word_address(word_address, value);
                }
            }
            TFormat::I16 => {
                if let Ok(value) = value.parse::<i16>() {
                    self.set_i16_to_word_address(word_address, value);
                }
            }
            TFormat::U32 => {
                if let Ok(value) = value.parse::<u32>() {
                    self.set_u32_to_word_address(word_address, value);
                }
            }
            TFormat::I32 => {
                if let Ok(value) = value.parse::<i32>() {
                    self.set_i32_to_word_address(word_address, value);
                }
            }
            TFormat::U64 => {
                if let Ok(value) = value.parse::<u64>() {
                    self.set_u64_to_word_address(word_address, value);
                }
            }
            TFormat::I64 => {
                if let Ok(value) = value.parse::<i64>() {
                    self.set_i64_to_word_address(word_address, value);
                }
            }
            TFormat::F32 => {
                if let Ok(value) = value.parse::<f32>() {
                    self.set_f32_to_word_address(word_address, value);
                }
            }
            TFormat::F64 => {
                if let Ok(value) = value.parse::<f64>() {
                    self.set_f64_to_word_address(word_address, value);
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
                self.set_string_to_word_address(word_address, &value);
            }
            TFormat::Unknown => (),
        }
    }

    /// Extrait une valeur [`TValue`] selon le [`Tag`]
    pub fn get_t_value_from_tag(&self, tag: &Tag) -> TValue {
        let word_address = tag.word_address;
        match tag.t_format {
            TFormat::Bool => TValue::Bool(self.get_bool_from_word_address(word_address)),
            TFormat::U8 => TValue::U8(self.get_u8_from_word_address(word_address)),
            TFormat::I8 => TValue::I8(self.get_i8_from_word_address(word_address)),
            TFormat::U16 => TValue::U16(self.get_u16_from_word_address(word_address)),
            TFormat::I16 => TValue::I16(self.get_i16_from_word_address(word_address)),
            TFormat::U32 => TValue::U32(self.get_u32_from_word_address(word_address)),
            TFormat::I32 => TValue::I32(self.get_i32_from_word_address(word_address)),
            TFormat::U64 => TValue::U64(self.get_u64_from_word_address(word_address)),
            TFormat::I64 => TValue::I64(self.get_i64_from_word_address(word_address)),
            TFormat::F32 => TValue::F32(self.get_f32_from_word_address(word_address)),
            TFormat::F64 => TValue::F64(self.get_f64_from_word_address(word_address)),
            TFormat::String(width) => TValue::String(
                width,
                self.get_string_from_word_address(word_address, width),
            ),
            TFormat::Unknown => TValue::String(3, "???".to_string()),
        }
    }

    /// Extrait un `Vec<u8>` de la [`Database`] selon [`WordAddress`]
    pub fn get_vec_u8_from_word_address(&self, word_address: WordAddress, nb_u8: usize) -> Vec<u8> {
        let mut ret = vec![];
        let word_address = word_address as usize;
        for n in 2 * word_address..2 * word_address + nb_u8 {
            ret.push(self.vec_u8[n]);
        }
        ret
    }

    /// Copie un `&[u8]` dans la [`Database`] selon [`WordAddress`]
    pub fn set_vec_u8_to_word_address(&mut self, word_address: WordAddress, vec_u8: &[u8]) {
        let mut word_address = 2 * word_address as usize;
        for value in vec_u8 {
            self.vec_u8[word_address] = *value;
            word_address += 1;
        }
    }
}
