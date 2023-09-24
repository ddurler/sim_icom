//! Format et conteneur des différentes valeurs pour les tags de la database

use std::fmt;

/// Format et conteneur d'une valeur atomique
#[derive(Clone, Debug)]
pub enum TValue {
    Bool(bool),
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    F32(f32),
    F64(f64),
    /// Longueur max. de la chaîne
    String(usize, String),
}

impl fmt::Display for TValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.extract_string())
    }
}

impl TValue {
    #[allow(dead_code)]
    pub fn extract_bool(&self) -> bool {
        self.extract_f64() != 0.0
    }

    #[allow(dead_code)]
    pub fn extract_u8(&self) -> u8 {
        let value = self.extract_u64();
        u8::try_from(value).unwrap_or(0)
    }

    #[allow(dead_code)]
    pub fn extract_i8(&self) -> i8 {
        let value = self.extract_i64();
        i8::try_from(value).unwrap_or(0)
    }

    #[allow(dead_code)]
    pub fn extract_u16(&self) -> u16 {
        let value = self.extract_u64();
        u16::try_from(value).unwrap_or(0)
    }

    #[allow(dead_code)]
    pub fn extract_i16(&self) -> i16 {
        let value = self.extract_i64();
        i16::try_from(value).unwrap_or(0)
    }

    #[allow(dead_code)]
    pub fn extract_u32(&self) -> u32 {
        let value = self.extract_u64();
        u32::try_from(value).unwrap_or(0)
    }

    #[allow(dead_code)]
    pub fn extract_i32(&self) -> i32 {
        let value = self.extract_i64();
        i32::try_from(value).unwrap_or(0)
    }

    #[allow(dead_code)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    pub fn extract_u64(&self) -> u64 {
        match self {
            TValue::Bool(value) => u64::from(*value),
            TValue::U8(value) => u64::try_from(*value).unwrap_or(0),
            TValue::I8(value) => u64::try_from(*value).unwrap_or(0),
            TValue::U16(value) => u64::try_from(*value).unwrap_or(0),
            TValue::I16(value) => u64::try_from(*value).unwrap_or(0),
            TValue::U32(value) => u64::try_from(*value).unwrap_or(0),
            TValue::I32(value) => u64::try_from(*value).unwrap_or(0),
            TValue::U64(value) => *value,
            TValue::I64(value) => u64::try_from(*value).unwrap_or(0),
            TValue::F32(value) => *value as u64,
            TValue::F64(value) => *value as u64,
            TValue::String(_, value) => value.parse::<u64>().unwrap_or(0),
        }
    }

    #[allow(dead_code)]
    #[allow(clippy::cast_possible_truncation)]
    pub fn extract_i64(&self) -> i64 {
        match self {
            TValue::Bool(value) => i64::from(*value),
            TValue::U8(value) => i64::try_from(*value).unwrap_or(0),
            TValue::I8(value) => i64::try_from(*value).unwrap_or(0),
            TValue::U16(value) => i64::try_from(*value).unwrap_or(0),
            TValue::I16(value) => i64::try_from(*value).unwrap_or(0),
            TValue::U32(value) => i64::try_from(*value).unwrap_or(0),
            TValue::I32(value) => i64::try_from(*value).unwrap_or(0),
            TValue::U64(value) => i64::try_from(*value).unwrap_or(0),
            TValue::I64(value) => *value,
            TValue::F32(value) => *value as i64,
            TValue::F64(value) => *value as i64,
            TValue::String(_, value) => value.parse::<i64>().unwrap_or(0),
        }
    }

    #[allow(dead_code)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    pub fn extract_f32(&self) -> f32 {
        self.extract_f64() as f32
    }

    #[allow(dead_code)]
    #[allow(clippy::cast_precision_loss)]
    pub fn extract_f64(&self) -> f64 {
        match self {
            TValue::Bool(value) => {
                if *value {
                    1.0
                } else {
                    0.0
                }
            }
            TValue::U8(value) => f64::try_from(*value).unwrap_or(0.0),
            TValue::I8(value) => f64::try_from(*value).unwrap_or(0.0),
            TValue::U16(value) => f64::try_from(*value).unwrap_or(0.0),
            TValue::I16(value) => f64::try_from(*value).unwrap_or(0.0),
            TValue::U32(value) => f64::try_from(*value).unwrap_or(0.0),
            TValue::I32(value) => f64::try_from(*value).unwrap_or(0.0),
            TValue::U64(value) => *value as f64,
            TValue::I64(value) => *value as f64,
            TValue::F32(value) => f64::try_from(*value).unwrap_or(0.0),
            TValue::F64(value) => *value,
            TValue::String(_, value) => value.parse::<f64>().unwrap_or(0.0),
        }
    }

    #[allow(dead_code)]
    pub fn extract_string(&self) -> String {
        match self {
            TValue::Bool(value) => {
                if *value {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            TValue::U8(value) => format!("{value}"),
            TValue::I8(value) => format!("{value}"),
            TValue::U16(value) => format!("{value}"),
            TValue::I16(value) => format!("{value}"),
            TValue::U32(value) => format!("{value}"),
            TValue::I32(value) => format!("{value}"),
            TValue::U64(value) => format!("{value}"),
            TValue::I64(value) => format!("{value}"),
            TValue::F32(value) => format!("{value}"),
            TValue::F64(value) => format!("{value}"),
            TValue::String(width, value) => {
                let value = value.trim();
                let value = if value.len() > *width {
                    // Tronque si trop long
                    // /!\ format! ne le fait pas...
                    value[..*width].to_string()
                } else {
                    value.to_string()
                };
                format!("{value:width$}")
            }
        }
    }

    #[allow(dead_code)]
    pub fn to_t_value_bool(&self) -> Self {
        TValue::Bool(self.extract_bool())
    }

    #[allow(dead_code)]
    pub fn to_t_value_u8(&self) -> Self {
        TValue::U8(self.extract_u8())
    }

    #[allow(dead_code)]
    pub fn to_t_value_i8(&self) -> Self {
        TValue::I8(self.extract_i8())
    }

    #[allow(dead_code)]
    pub fn to_t_value_u16(&self) -> Self {
        TValue::U16(self.extract_u16())
    }

    #[allow(dead_code)]
    pub fn to_t_value_i16(&self) -> Self {
        TValue::I16(self.extract_i16())
    }

    #[allow(dead_code)]
    pub fn to_t_value_u32(&self) -> Self {
        TValue::U32(self.extract_u32())
    }

    #[allow(dead_code)]
    pub fn to_t_value_i32(&self) -> Self {
        TValue::I32(self.extract_i32())
    }

    #[allow(dead_code)]
    pub fn to_t_value_u64(&self) -> Self {
        TValue::U64(self.extract_u64())
    }

    #[allow(dead_code)]

    pub fn to_t_value_i64(&self) -> Self {
        TValue::I64(self.extract_i64())
    }

    #[allow(dead_code)]
    pub fn to_t_value_f32(&self) -> Self {
        TValue::F32(self.extract_f32())
    }

    #[allow(dead_code)]
    pub fn to_t_value_f64(&self) -> Self {
        TValue::F64(self.extract_f64())
    }

    #[allow(dead_code)]

    pub fn to_t_value_string(&self, width: usize) -> Self {
        let value = self.extract_string();
        let value = value.trim();
        let value = if value.len() > width {
            // Tronque si trop long
            // /!\ format! ne le fait pas...
            value[..width].to_string()
        } else {
            value.to_string()
        };
        TValue::String(width, format!("{value:width$}"))
    }

    #[allow(dead_code)]
    pub fn string_width(&self) -> usize {
        match self {
            TValue::String(width, _) => *width,
            _ => self.extract_string().len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_float_eq::*;

    #[test]
    fn test_extract_bool_true() {
        let value = TValue::Bool(true);

        assert!(value.extract_bool());
        assert!(value.extract_u8() != 0);
        assert!(value.extract_i8() != 0);
        assert!(value.extract_u16() != 0);
        assert!(value.extract_i16() != 0);
        assert!(value.extract_u32() != 0);
        assert!(value.extract_i32() != 0);
        assert!(value.extract_u64() != 0);
        assert!(value.extract_i64() != 0);
        assert!(value.extract_f32() != 0.0);
        assert!(value.extract_f64() != 0.0);
        assert_eq!(value.extract_string(), "true");
    }

    #[test]
    fn test_extract_bool_false() {
        let value = TValue::Bool(false);

        assert!(!value.extract_bool());
        assert_eq!(value.extract_u8(), 0);
        assert_eq!(value.extract_i8(), 0);
        assert_eq!(value.extract_u16(), 0);
        assert_eq!(value.extract_i16(), 0);
        assert_eq!(value.extract_u32(), 0);
        assert_eq!(value.extract_i32(), 0);
        assert_eq!(value.extract_i64(), 0);
        assert_f32_near!(value.extract_f32(), 0.0);
        assert_f64_near!(value.extract_f64(), 0.0);
        assert_eq!(value.extract_string(), "false");
    }

    #[test]
    fn test_extract_u8() {
        let value = TValue::U8(123);

        assert!(value.extract_bool());
        assert_eq!(value.extract_u8(), 123);
        assert_eq!(value.extract_i8(), 123);
        assert_eq!(value.extract_u16(), 123);
        assert_eq!(value.extract_i16(), 123);
        assert_eq!(value.extract_u32(), 123);
        assert_eq!(value.extract_i32(), 123);
        assert_eq!(value.extract_u64(), 123);
        assert_eq!(value.extract_i64(), 123);
        assert_f32_near!(value.extract_f32(), 123.0);
        assert_f64_near!(value.extract_f64(), 123.0);
        assert_eq!(value.extract_string(), "123");
    }

    #[test]
    fn test_extract_i8() {
        let value = TValue::I8(-123);

        assert!(value.extract_bool());
        assert_eq!(value.extract_u8(), 0);
        assert_eq!(value.extract_i8(), -123);
        assert_eq!(value.extract_u16(), 0);
        assert_eq!(value.extract_i16(), -123);
        assert_eq!(value.extract_u32(), 0);
        assert_eq!(value.extract_i32(), -123);
        assert_eq!(value.extract_u64(), 0);
        assert_eq!(value.extract_i64(), -123);
        assert_f32_near!(value.extract_f32(), -123.0);
        assert_f64_near!(value.extract_f64(), -123.0);
        assert_eq!(value.extract_string(), "-123");
    }

    #[test]
    fn test_extract_u16() {
        let value = TValue::U16(123);

        assert!(value.extract_bool());
        assert_eq!(value.extract_u8(), 123);
        assert_eq!(value.extract_i8(), 123);
        assert_eq!(value.extract_u16(), 123);
        assert_eq!(value.extract_i16(), 123);
        assert_eq!(value.extract_u32(), 123);
        assert_eq!(value.extract_i32(), 123);
        assert_eq!(value.extract_u64(), 123);
        assert_eq!(value.extract_i64(), 123);
        assert_f32_near!(value.extract_f32(), 123.0);
        assert_f64_near!(value.extract_f64(), 123.0);
        assert_eq!(value.extract_string(), "123");
    }

    #[test]
    fn test_extract_i16() {
        let value = TValue::I16(-123);

        assert!(value.extract_bool());
        assert_eq!(value.extract_u8(), 0);
        assert_eq!(value.extract_i8(), -123);
        assert_eq!(value.extract_u16(), 0);
        assert_eq!(value.extract_i16(), -123);
        assert_eq!(value.extract_u32(), 0);
        assert_eq!(value.extract_i32(), -123);
        assert_eq!(value.extract_u64(), 0);
        assert_eq!(value.extract_i64(), -123);
        assert_f32_near!(value.extract_f32(), -123.0);
        assert_f64_near!(value.extract_f64(), -123.0);
        assert_eq!(value.extract_string(), "-123");
    }

    #[test]
    fn test_extract_u32() {
        let value = TValue::U32(123);

        assert!(value.extract_bool());
        assert_eq!(value.extract_u8(), 123);
        assert_eq!(value.extract_i8(), 123);
        assert_eq!(value.extract_u16(), 123);
        assert_eq!(value.extract_i16(), 123);
        assert_eq!(value.extract_u32(), 123);
        assert_eq!(value.extract_i32(), 123);
        assert_eq!(value.extract_u64(), 123);
        assert_eq!(value.extract_i64(), 123);
        assert_f32_near!(value.extract_f32(), 123.0);
        assert_f64_near!(value.extract_f64(), 123.0);
        assert_eq!(value.extract_string(), "123");
    }

    #[test]
    fn test_extract_i32() {
        let value = TValue::I32(-123);

        assert!(value.extract_bool());
        assert_eq!(value.extract_u8(), 0);
        assert_eq!(value.extract_i8(), -123);
        assert_eq!(value.extract_u16(), 0);
        assert_eq!(value.extract_i16(), -123);
        assert_eq!(value.extract_u32(), 0);
        assert_eq!(value.extract_i32(), -123);
        assert_eq!(value.extract_u64(), 0);
        assert_eq!(value.extract_i64(), -123);
        assert_f32_near!(value.extract_f32(), -123.0);
        assert_f64_near!(value.extract_f64(), -123.0);
        assert_eq!(value.extract_string(), "-123");
    }

    #[test]
    fn test_extract_u64() {
        let value = TValue::U64(123);

        assert!(value.extract_bool());
        assert_eq!(value.extract_u8(), 123);
        assert_eq!(value.extract_i8(), 123);
        assert_eq!(value.extract_u16(), 123);
        assert_eq!(value.extract_i16(), 123);
        assert_eq!(value.extract_u32(), 123);
        assert_eq!(value.extract_i32(), 123);
        assert_eq!(value.extract_u64(), 123);
        assert_eq!(value.extract_i64(), 123);
        assert_f32_near!(value.extract_f32(), 123.0);
        assert_f64_near!(value.extract_f64(), 123.0);
        assert_eq!(value.extract_string(), "123");
    }

    #[test]
    fn test_extract_i64() {
        let value = TValue::I64(-123);

        assert!(value.extract_bool());
        assert_eq!(value.extract_u8(), 0);
        assert_eq!(value.extract_i8(), -123);
        assert_eq!(value.extract_u16(), 0);
        assert_eq!(value.extract_i16(), -123);
        assert_eq!(value.extract_u32(), 0);
        assert_eq!(value.extract_i32(), -123);
        assert_eq!(value.extract_u64(), 0);
        assert_eq!(value.extract_i64(), -123);
        assert_f32_near!(value.extract_f32(), -123.0);
        assert_f64_near!(value.extract_f64(), -123.0);
        assert_eq!(value.extract_string(), "-123");
    }

    #[test]
    fn test_extract_f32() {
        let value = TValue::F32(123.0);

        assert!(value.extract_bool());
        assert_eq!(value.extract_u8(), 123);
        assert_eq!(value.extract_i8(), 123);
        assert_eq!(value.extract_u16(), 123);
        assert_eq!(value.extract_i16(), 123);
        assert_eq!(value.extract_u32(), 123);
        assert_eq!(value.extract_i32(), 123);
        assert_eq!(value.extract_u64(), 123);
        assert_eq!(value.extract_i64(), 123);
        assert_f32_near!(value.extract_f32(), 123.0);
        assert_f64_near!(value.extract_f64(), 123.0);
        assert_eq!(value.extract_string(), "123");
    }

    #[test]
    fn test_extract_f64() {
        let value = TValue::F64(123.0);

        assert!(value.extract_bool());
        assert_eq!(value.extract_u8(), 123);
        assert_eq!(value.extract_i8(), 123);
        assert_eq!(value.extract_u16(), 123);
        assert_eq!(value.extract_i16(), 123);
        assert_eq!(value.extract_u32(), 123);
        assert_eq!(value.extract_i32(), 123);
        assert_eq!(value.extract_u64(), 123);
        assert_eq!(value.extract_i64(), 123);
        assert_f32_near!(value.extract_f32(), 123.0);
        assert_f64_near!(value.extract_f64(), 123.0);
        assert_eq!(value.extract_string(), "123");
    }

    #[test]
    fn test_extract_string() {
        let value = TValue::String(3, "123".to_string());

        assert!(value.extract_bool());
        assert_eq!(value.extract_u8(), 123);
        assert_eq!(value.extract_i8(), 123);
        assert_eq!(value.extract_u16(), 123);
        assert_eq!(value.extract_i16(), 123);
        assert_eq!(value.extract_u32(), 123);
        assert_eq!(value.extract_i32(), 123);
        assert_eq!(value.extract_u64(), 123);
        assert_eq!(value.extract_i64(), 123);
        assert_f32_near!(value.extract_f32(), 123.0);
        assert_f64_near!(value.extract_f64(), 123.0);
        assert_eq!(value.extract_string(), "123");
    }

    #[test]
    fn test_to_t_value() {
        let value = TValue::U16(1);
        let value = value.to_t_value_bool();

        match value {
            TValue::Bool(value) => assert!(value),
            _ => panic!("Conversion incorrecte en bool"),
        };
    }

    #[test]
    fn test_to_t_u8() {
        let value = TValue::U32(1);
        let value = value.to_t_value_u8();

        match value {
            TValue::U8(value) => assert_eq!(value, 1),
            _ => panic!("Conversion incorrecte en u8"),
        };
    }

    #[test]
    fn test_to_t_i8() {
        let value = TValue::I32(-1);
        let value = value.to_t_value_i8();

        match value {
            TValue::I8(value) => assert_eq!(value, -1),
            _ => panic!("Conversion incorrecte en i8"),
        };
    }

    #[test]
    fn test_to_t_u16() {
        let value = TValue::U64(1);
        let value = value.to_t_value_u16();

        match value {
            TValue::U16(value) => assert_eq!(value, 1),
            _ => panic!("Conversion incorrecte en u16"),
        };
    }

    #[test]
    fn test_to_t_i16() {
        let value = TValue::I64(-1);
        let value = value.to_t_value_i16();

        match value {
            TValue::I16(value) => assert_eq!(value, -1),
            _ => panic!("Conversion incorrecte en i16"),
        };
    }

    #[test]
    fn test_to_t_u32() {
        let value = TValue::F32(1.0);
        let value = value.to_t_value_u32();

        match value {
            TValue::U32(value) => assert_eq!(value, 1),
            _ => panic!("Conversion incorrecte en u32"),
        };
    }

    #[test]
    fn test_to_t_i32() {
        let value = TValue::F64(-1.0);
        let value = value.to_t_value_i32();

        match value {
            TValue::I32(value) => assert_eq!(value, -1),
            _ => panic!("Conversion incorrecte en i32"),
        };
    }

    #[test]
    fn test_to_t_u64() {
        let value = TValue::U8(1);
        let value = value.to_t_value_u64();

        match value {
            TValue::U64(value) => assert_eq!(value, 1),
            _ => panic!("Conversion incorrecte en u64"),
        };
    }

    #[test]
    fn test_to_t_i64() {
        let value = TValue::I8(-1);
        let value = value.to_t_value_i64();

        match value {
            TValue::I64(value) => assert_eq!(value, -1),
            _ => panic!("Conversion incorrecte en i32"),
        };
    }

    #[test]
    fn test_to_t_f32() {
        let value = TValue::U16(1);
        let value = value.to_t_value_f32();

        match value {
            TValue::F32(value) => assert_f32_near!(value, 1.0),
            _ => panic!("Conversion incorrecte en f32"),
        };
    }

    #[test]
    fn test_to_t_f64() {
        let value = TValue::I16(-1);
        let value = value.to_t_value_f64();

        match value {
            TValue::F64(value) => assert_f64_near!(value, -1.0),
            _ => panic!("Conversion incorrecte en f64"),
        };
    }

    #[test]
    fn test_to_t_string() {
        let value = TValue::I32(-1);
        let value = value.to_t_value_string(10);

        match value {
            TValue::String(width, value) => {
                assert_eq!(width, 10);
                assert_eq!(value.len(), 10);
                assert!(value.starts_with("-1"));
            }
            _ => panic!("Conversion incorrecte en string"),
        };
    }

    #[test]
    fn test_string_width() {
        for width in 1..10 {
            let value = TValue::String(width, "TOTO".to_string());
            assert_eq!(value.string_width(), width);
            let display = format!("{value}");
            assert_eq!(display.len(), width);
        }
    }
}
