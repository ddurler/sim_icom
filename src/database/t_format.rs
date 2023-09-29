//! Codage et format d'une donnée de la database sur un octet
//!
//! 0x00 : Pas de format connu
//! 0x01 = u8
//! 0x11 = Booléen
//! 0x02 = u16
//! 0x04 = u32
//! 0x08 = u64
//! 0x41 = i8
//! 0x42 = i16
//! 0x44 = i32
//! 0x48 = i64
//! 0x64 = f32
//! 0x68 = f64
//! 0x81 à FF = String(1-127)

/// Énumération des formats reconnus
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TFormat {
    #[default]
    Unknown,
    Bool,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    F32,
    F64,
    String(usize),
}

impl From<u8> for TFormat {
    fn from(value: u8) -> Self {
        match value {
            0x11 => TFormat::Bool,
            0x01 => TFormat::U8,
            0x41 => TFormat::I8,
            0x02 => TFormat::U16,
            0x42 => TFormat::I16,
            0x04 => TFormat::U32,
            0x44 => TFormat::I32,
            0x08 => TFormat::U64,
            0x48 => TFormat::I64,
            0x64 => TFormat::F32,
            0x68 => TFormat::F64,
            n @ 0x81..=0xFF => TFormat::String((n - 0x80) as usize),
            _ => TFormat::Unknown,
        }
    }
}

impl From<TFormat> for u8 {
    fn from(value: TFormat) -> Self {
        match value {
            TFormat::Unknown => 0x00,
            TFormat::Bool => 0x11,
            TFormat::U8 => 0x01,
            TFormat::I8 => 0x41,
            TFormat::U16 => 0x02,
            TFormat::I16 => 0x42,
            TFormat::U32 => 0x04,
            TFormat::I32 => 0x44,
            TFormat::U64 => 0x08,
            TFormat::I64 => 0x48,
            TFormat::F32 => 0x64,
            TFormat::F64 => 0x68,
            TFormat::String(n) => {
                if (1..=127).contains(&n) {
                    0x80 + u8::try_from(n).unwrap()
                } else {
                    0x00
                }
            }
        }
    }
}

impl TFormat {
    /// Retourne le nombre d'octets utilisés par ce format
    #[must_use]
    #[allow(dead_code)]
    pub fn nb_bytes(&self) -> usize {
        match self {
            TFormat::Unknown => 0,
            TFormat::U8 | TFormat::Bool | TFormat::I8 => 1,
            TFormat::U16 | TFormat::I16 => 2,
            TFormat::U32 | TFormat::I32 | TFormat::F32 => 4,
            TFormat::U64 | TFormat::I64 | TFormat::F64 => 8,
            TFormat::String(n) => *n,
        }
    }

    /// Retourne le nombre de mots utilisés par ce format
    #[must_use]
    #[allow(dead_code)]
    pub fn nb_words(&self) -> usize {
        match self {
            TFormat::Unknown => 0,
            TFormat::U8 | TFormat::Bool | TFormat::I8 | TFormat::U16 | TFormat::I16 => 1,
            TFormat::U32 | TFormat::I32 | TFormat::F32 => 2,
            TFormat::U64 | TFormat::I64 | TFormat::F64 => 4,
            TFormat::String(n) => {
                if (1..=127).contains(n) {
                    (*n + 1) / 2
                } else {
                    0
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        for t_format in vec![
            TFormat::Unknown,
            TFormat::U8,
            TFormat::Bool,
            TFormat::U16,
            TFormat::U32,
            TFormat::U64,
            TFormat::I8,
            TFormat::I16,
            TFormat::I32,
            TFormat::I64,
            TFormat::F32,
            TFormat::F64,
            TFormat::String(1),
            TFormat::String(10),
        ] {
            let format_u8 = u8::from(t_format);
            assert_eq!(t_format, TFormat::from(format_u8));
        }
    }

    #[test]
    fn test_nb_bytes_or_words() {
        for w_format in vec![
            TFormat::Unknown,
            TFormat::U8,
            TFormat::Bool,
            TFormat::U16,
            TFormat::U32,
            TFormat::U64,
            TFormat::I8,
            TFormat::I16,
            TFormat::I32,
            TFormat::I64,
            TFormat::F32,
            TFormat::F64,
            TFormat::String(1),
            TFormat::String(10),
        ] {
            let nb_bytes = w_format.nb_bytes();
            let nb_words = w_format.nb_words();
            // Tous les bytes doivent tenir dans tous les words...
            assert!(nb_bytes <= 2 * nb_words);
            // Au pire, il reste un byte de libre dans le dernier word...
            assert!(2 * nb_words - nb_bytes <= 1);
        }
    }
}
