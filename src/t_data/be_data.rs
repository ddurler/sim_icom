//! Conversion de donnée encodée en big endian (BE)
//!

use std::vec;

use super::{TFormat, TValue};

/// Extraction d'une donnée : `TFormat` + `Vec<u8>` -> `TValue`
#[allow(clippy::cast_possible_wrap)]
pub fn decode(t_format: TFormat, vec_u8: &[u8]) -> Result<TValue, &'static str> {
    if vec_u8.len() < t_format.nb_bytes() {
        Err("Missing u8 in data")
    } else {
        let vec_u8 = vec_u8.to_vec();
        Ok(match t_format {
            TFormat::Unknown => TValue::VecU8(0, vec![]),
            TFormat::Bool => TValue::Bool(vec_u8[0] != 0),
            TFormat::U8 => TValue::U8(vec_u8[0]),
            TFormat::I8 => TValue::I8(vec_u8[0] as i8),
            TFormat::U16 => {
                let vec_u8: Vec<u8> = vec_u8.iter().take(2).copied().collect();
                let vec_u8: [u8; 2] = vec_u8.try_into().unwrap();
                TValue::U16(u16::from_be_bytes(vec_u8))
            }
            TFormat::I16 => {
                let vec_u8: Vec<u8> = vec_u8.iter().take(2).copied().collect();
                let vec_u8: [u8; 2] = vec_u8.try_into().unwrap();
                TValue::I16(i16::from_be_bytes(vec_u8))
            }
            TFormat::U32 => {
                let vec_u8: Vec<u8> = vec_u8.iter().take(4).copied().collect();
                let vec_u8: [u8; 4] = vec_u8.try_into().unwrap();
                TValue::U32(u32::from_be_bytes(vec_u8))
            }
            TFormat::I32 => {
                let vec_u8: Vec<u8> = vec_u8.iter().take(4).copied().collect();
                let vec_u8: [u8; 4] = vec_u8.try_into().unwrap();
                TValue::I32(i32::from_be_bytes(vec_u8))
            }
            TFormat::U64 => {
                let vec_u8: Vec<u8> = vec_u8.iter().take(8).copied().collect();
                let vec_u8: [u8; 8] = vec_u8.try_into().unwrap();
                TValue::U64(u64::from_be_bytes(vec_u8))
            }
            TFormat::I64 => {
                let vec_u8: Vec<u8> = vec_u8.iter().take(8).copied().collect();
                let vec_u8: [u8; 8] = vec_u8.try_into().unwrap();
                TValue::I64(i64::from_be_bytes(vec_u8))
            }
            TFormat::F32 => {
                let vec_u8: Vec<u8> = vec_u8.iter().take(4).copied().collect();
                let vec_u8: [u8; 4] = vec_u8.try_into().unwrap();
                TValue::F32(f32::from_be_bytes(vec_u8))
            }
            TFormat::F64 => {
                let vec_u8: Vec<u8> = vec_u8.iter().take(8).copied().collect();
                let vec_u8: [u8; 8] = vec_u8.try_into().unwrap();
                TValue::F64(f64::from_be_bytes(vec_u8))
            }
            TFormat::VecU8(n) => TValue::VecU8(n, vec_u8.clone()),
        })
    }
}

/// Construction d'une donnée : `TValue` -> `Vec<u8>`
#[allow(clippy::cast_sign_loss)]
pub fn encode(t_value: &TValue) -> Vec<u8> {
    match t_value {
        TValue::Bool(value) => {
            if *value {
                vec![1]
            } else {
                vec![0]
            }
        }
        TValue::U8(value) => vec![*value],
        TValue::I8(value) => vec![*value as u8],
        TValue::U16(value) => value.to_be_bytes().to_vec(),
        TValue::I16(value) => value.to_be_bytes().to_vec(),
        TValue::U32(value) => value.to_be_bytes().to_vec(),
        TValue::I32(value) => value.to_be_bytes().to_vec(),
        TValue::U64(value) => value.to_be_bytes().to_vec(),
        TValue::I64(value) => value.to_be_bytes().to_vec(),
        TValue::F32(value) => value.to_be_bytes().to_vec(),
        TValue::F64(value) => value.to_be_bytes().to_vec(),
        TValue::VecU8(_, value) => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use crate::t_data::string_to_vec_u8;

    use super::*;

    #[test]
    fn test_encode_decode() {
        for t_value in [
            TValue::Bool(true),
            TValue::U8(100),
            TValue::I8(-100),
            TValue::U16(10_000),
            TValue::I16(-10_000),
            TValue::U32(1_000_000),
            TValue::I32(-1_000_000),
            TValue::U64(1_000_000),
            TValue::I64(-1_000_000),
            TValue::F32(-1.23),
            TValue::F64(-1.23),
            TValue::VecU8(3, string_to_vec_u8("ABC")),
            TValue::VecU8(3, vec![0xFF, 0xFF, 0xFF]),
        ] {
            let t_format = TFormat::from(&t_value);
            let vec_u8 = encode(&t_value);
            let t_value_decode_vec_u8 = decode(t_format, &vec_u8).unwrap();
            let encode_decode_vec_u8 = encode(&t_value_decode_vec_u8);
            assert_eq!(vec_u8, encode_decode_vec_u8);
        }
    }
}
