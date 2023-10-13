//! Format et conteneur des différentes valeurs pour les tags de la database

use std::fmt;

use super::TFormat;

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
        write!(f, "{}", String::from(self))
    }
}

impl From<&TValue> for TFormat {
    fn from(value: &TValue) -> Self {
        match value {
            TValue::Bool(_) => TFormat::Bool,
            TValue::U8(_) => TFormat::U8,
            TValue::I8(_) => TFormat::I8,
            TValue::U16(_) => TFormat::U16,
            TValue::I16(_) => TFormat::I16,
            TValue::U32(_) => TFormat::U32,
            TValue::I32(_) => TFormat::I32,
            TValue::U64(_) => TFormat::U64,
            TValue::I64(_) => TFormat::I64,
            TValue::F32(_) => TFormat::F32,
            TValue::F64(_) => TFormat::F64,
            TValue::String(width, _) => TFormat::String(*width),
        }
    }
}

impl From<&TValue> for bool {
    fn from(value: &TValue) -> Self {
        i64::from(value) != 0
    }
}

impl From<&TValue> for u8 {
    fn from(value: &TValue) -> Self {
        u8::try_from(u64::from(value)).unwrap_or(0)
    }
}

impl From<&TValue> for u16 {
    fn from(value: &TValue) -> Self {
        u16::try_from(u64::from(value)).unwrap_or(0)
    }
}

impl From<&TValue> for u32 {
    fn from(value: &TValue) -> Self {
        u32::try_from(u64::from(value)).unwrap_or(0)
    }
}

impl From<&TValue> for u64 {
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    fn from(value: &TValue) -> Self {
        match value {
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
}

impl From<&TValue> for i8 {
    fn from(value: &TValue) -> Self {
        i8::try_from(i64::from(value)).unwrap_or(0)
    }
}

impl From<&TValue> for i16 {
    fn from(value: &TValue) -> Self {
        i16::try_from(i64::from(value)).unwrap_or(0)
    }
}

impl From<&TValue> for i32 {
    fn from(value: &TValue) -> Self {
        i32::try_from(i64::from(value)).unwrap_or(0)
    }
}

impl From<&TValue> for i64 {
    #[allow(clippy::cast_possible_truncation)]
    fn from(value: &TValue) -> Self {
        match value {
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
}

impl From<&TValue> for f32 {
    #[allow(clippy::cast_possible_truncation)]
    fn from(value: &TValue) -> Self {
        f64::from(value) as f32
    }
}

impl From<&TValue> for f64 {
    #[allow(clippy::cast_precision_loss)]
    fn from(value: &TValue) -> Self {
        match value {
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
}

impl From<&TValue> for String {
    fn from(value: &TValue) -> Self {
        match value {
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
}

impl TValue {
    #[allow(dead_code)]
    pub fn to_t_value_bool(&self) -> Self {
        TValue::Bool(bool::from(self))
    }

    #[allow(dead_code)]
    pub fn to_t_value_u8(&self) -> Self {
        TValue::U8(u8::from(self))
    }

    #[allow(dead_code)]
    pub fn to_t_value_i8(&self) -> Self {
        TValue::I8(i8::from(self))
    }

    #[allow(dead_code)]
    pub fn to_t_value_u16(&self) -> Self {
        TValue::U16(u16::from(self))
    }

    #[allow(dead_code)]
    pub fn to_t_value_i16(&self) -> Self {
        TValue::I16(i16::from(self))
    }

    #[allow(dead_code)]
    pub fn to_t_value_u32(&self) -> Self {
        TValue::U32(u32::from(self))
    }

    #[allow(dead_code)]
    pub fn to_t_value_i32(&self) -> Self {
        TValue::I32(i32::from(self))
    }

    #[allow(dead_code)]
    pub fn to_t_value_u64(&self) -> Self {
        TValue::U64(u64::from(self))
    }

    #[allow(dead_code)]
    pub fn to_t_value_i64(&self) -> Self {
        TValue::I64(i64::from(self))
    }

    #[allow(dead_code)]
    pub fn to_t_value_f32(&self) -> Self {
        TValue::F32(f32::from(self))
    }

    #[allow(dead_code)]
    pub fn to_t_value_f64(&self) -> Self {
        TValue::F64(f64::from(self))
    }

    #[allow(dead_code)]
    pub fn to_t_value_string(&self, width: usize) -> Self {
        let value = String::from(self);
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
            _ => String::from(self).len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_float_eq::*;

    #[test]
    fn test_extract_t_format() {
        for (t_value, t_format) in [
            (TValue::Bool(true), TFormat::Bool),
            (TValue::U8(100), TFormat::U8),
            (TValue::I8(-100), TFormat::I8),
            (TValue::U16(10000), TFormat::U16),
            (TValue::I16(-10000), TFormat::I16),
            (TValue::U32(1_000_000), TFormat::U32),
            (TValue::I32(-1_000_000), TFormat::I32),
            (TValue::U64(1_000_000), TFormat::U64),
            (TValue::I64(-1_000_000), TFormat::I64),
            (TValue::F32(-1.23), TFormat::F32),
            (TValue::F64(-1.23), TFormat::F64),
            (TValue::String(3, "ABC".to_string()), TFormat::String(3)),
        ] {
            assert_eq!(TFormat::from(&t_value), t_format);
        }
    }

    #[test]
    fn test_bool_true_from() {
        let value = TValue::Bool(true);

        assert!(bool::from(&value));
        assert!(u8::from(&value) != 0);
        assert!(i8::from(&value) != 0);
        assert!(u16::from(&value) != 0);
        assert!(i16::from(&value) != 0);
        assert!(u32::from(&value) != 0);
        assert!(i32::from(&value) != 0);
        assert!(u64::from(&value) != 0);
        assert!(i64::from(&value) != 0);
        assert!(f32::from(&value) != 0.0);
        assert!(f64::from(&value) != 0.0);
        assert_eq!(String::from(&value), "true");
    }

    #[test]
    fn test_bool_false_from() {
        let value = TValue::Bool(false);

        assert!(!bool::from(&value));
        assert_eq!(u8::from(&value), 0);
        assert_eq!(i8::from(&value), 0);
        assert_eq!(u16::from(&value), 0);
        assert_eq!(i16::from(&value), 0);
        assert_eq!(u32::from(&value), 0);
        assert_eq!(i32::from(&value), 0);
        assert_eq!(u64::from(&value), 0);
        assert_eq!(i64::from(&value), 0);
        assert_f32_near!(f32::from(&value), 0.0);
        assert_f64_near!(f64::from(&value), 0.0);
        assert_eq!(String::from(&value), "false");
    }

    #[test]
    fn test_extract_unsigned() {
        for value in vec![
            TValue::U8(123),
            TValue::U16(123),
            TValue::U32(123),
            TValue::U64(123),
            TValue::F32(123.0),
            TValue::F64(123.0),
            TValue::String(3, "123".to_string()),
        ] {
            assert!(bool::from(&value));
            assert_eq!(u8::from(&value), 123);
            assert_eq!(i8::from(&value), 123);
            assert_eq!(u16::from(&value), 123);
            assert_eq!(i16::from(&value), 123);
            assert_eq!(u32::from(&value), 123);
            assert_eq!(i32::from(&value), 123);
            assert_eq!(u64::from(&value), 123);
            assert_eq!(i64::from(&value), 123);
            assert_f32_near!(f32::from(&value), 123.0);
            assert_f64_near!(f64::from(&value), 123.0);
            assert_eq!(String::from(&value), "123");
        }
    }

    #[test]
    fn test_extract_signed() {
        for value in vec![
            TValue::I8(-123),
            TValue::I16(-123),
            TValue::I32(-123),
            TValue::I64(-123),
            TValue::F32(-123.0),
            TValue::F64(-123.0),
            TValue::String(4, "-123".to_string()),
        ] {
            assert!(bool::from(&value));
            assert_eq!(u8::from(&value), 0);
            assert_eq!(i8::from(&value), -123);
            assert_eq!(u16::from(&value), 0);
            assert_eq!(i16::from(&value), -123);
            assert_eq!(u32::from(&value), 0);
            assert_eq!(i32::from(&value), -123);
            assert_eq!(u64::from(&value), 0);
            assert_eq!(i64::from(&value), -123);
            assert_f32_near!(f32::from(&value), -123.0);
            assert_f64_near!(f64::from(&value), -123.0);
            assert_eq!(String::from(&value), "-123");
        }
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
