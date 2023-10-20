//! Encodage et décodage des trames TLV (Tag + Length + Value) utilisées pour
//! communiquer entre l'AFSEC+ et l'ICOM.
//!
//! Ce module propose de gérer la construction et l'analyse de ces trames sous 2 aspects:
//!
//! * `RawFrame`: Trame 'brute' telle qu'échangée via la liaison série sous forme d'un `Vec<u8>`
//! * `DataFrame`: Trame contenant un tag et une liste de données `DataItem`
//!
//! Les structures ou énumérations suivantes sont présentes:
//!
//! * `FrameState`: Identifie l'avance lors de la construction d'une `RawFrame`
//! * `DataItem`: Donnée d'une trame avec un tag et une liste de données (elles-mêmes au format TLV)
//! * `FrameErreur`: Situation d'erreur lors de l'encodage ou décodage des trames
//!

mod data_frame;
pub use data_frame::DataFrame;

mod data_item;
pub use data_item::DataItem;

mod raw_frame;
pub use raw_frame::{FrameError, FrameState, RawFrame};
pub use raw_frame::{ACK, ETX, NACK, STX};

#[cfg(test)]
mod tests {
    use super::*;
    use assert_float_eq::*;

    use crate::t_data::{string_to_vec_u8, TFormat, TValue};

    // Les tests suivants sont ceux du fichier `TLVFrame.c` du résident #4000 de l'AFSEC+

    #[test]
    fn test_construction_af_alive() {
        /* Construction AF_ALIVE (type = 0 et pas de données) */
        let raw_frame = RawFrame::new_message(0);
        assert_eq!(raw_frame.encode(), vec![2, 0, 0, 0, 3]);
        assert_eq!(raw_frame.get_state(), FrameState::Ok);
        let data_frame = DataFrame::try_from(raw_frame).unwrap();
        assert_eq!(data_frame.get_tag(), 0);
        assert_eq!(data_frame.get_data_items().len(), 0);
    }

    #[test]
    fn test_construction_avec_une_string() {
        /* Construction avec une string */
        let mut raw_frame = RawFrame::new_message(0x23);
        raw_frame
            .try_extend_data_item(&DataItem::new(
                0x45,
                TFormat::VecU8(5),
                TValue::VecU8(5, string_to_vec_u8("ABCDE")),
            ))
            .unwrap();
        assert_eq!(
            raw_frame.encode(),
            vec![0x02, 0x23, 0x07, 0x45, 0x85, b'A', b'B', b'C', b'D', b'E', 0xA5, 0x03]
        );
        assert_eq!(raw_frame.get_state(), FrameState::Ok);
        let data_frame = DataFrame::try_from(raw_frame).unwrap();
        assert_eq!(data_frame.get_tag(), 0x23);
        assert_eq!(data_frame.get_data_items().len(), 1);
        let data_item = data_frame.get_data_items()[0].clone();
        assert_eq!(data_item.tag, 0x45);
        assert_eq!(data_item.t_format, TFormat::VecU8(5));
        assert_eq!(String::from(&data_item.t_value), "ABCDE");
    }

    #[test]
    fn test_construction_avec_un_vec_u8() {
        /* Test en + */
        /* Construction avec un vec_u8 */
        let mut raw_frame = RawFrame::new_message(0x23);
        raw_frame
            .try_extend_data_item(&DataItem::new(
                0x45,
                TFormat::VecU8(5),
                TValue::VecU8(5, vec![0x80, 0x90, 0xA0, 0xC0, 0xF0]),
            ))
            .unwrap();
        assert_eq!(
            raw_frame.encode(),
            vec![0x02, 0x23, 0x07, 0x45, 0x85, 0x80, 0x90, 0xA0, 0xC0, 0xF0, 100, 0x03]
        );
        assert_eq!(raw_frame.get_state(), FrameState::Ok);
        let data_frame = DataFrame::try_from(raw_frame).unwrap();
        assert_eq!(data_frame.get_tag(), 0x23);
        assert_eq!(data_frame.get_data_items().len(), 1);
        let data_item = data_frame.get_data_items()[0].clone();
        assert_eq!(data_item.tag, 0x45);
        assert_eq!(data_item.t_format, TFormat::VecU8(5));
        assert_eq!(
            data_item.t_value.to_vec_u8(),
            vec![0x80, 0x90, 0xA0, 0xC0, 0xF0]
        );
    }

    #[test]
    fn test_construction_avec_3_strings() {
        /* Construction avec 3 strings */
        let mut raw_frame = RawFrame::new_message(0x23);
        raw_frame
            .try_extend_data_item(&DataItem::new(
                0x45,
                TFormat::VecU8(1),
                TValue::VecU8(1, string_to_vec_u8("X")),
            ))
            .unwrap();
        raw_frame
            .try_extend_data_item(&DataItem::new(
                0x67,
                TFormat::VecU8(1),
                TValue::VecU8(1, string_to_vec_u8("Y")),
            ))
            .unwrap();
        raw_frame
            .try_extend_data_item(&DataItem::new(
                0x89,
                TFormat::VecU8(1),
                TValue::VecU8(1, string_to_vec_u8("Z")),
            ))
            .unwrap();
        assert_eq!(
            raw_frame.encode(),
            vec![
                0x02, 0x23, 0x09, 0x45, 0x81, b'X', 0x67, 0x81, b'Y', 0x89, 0x81, b'Z', 0x5B, 0x03
            ]
        );
        assert_eq!(raw_frame.get_state(), FrameState::Ok);
        let data_frame = DataFrame::try_from(raw_frame).unwrap();
        assert_eq!(data_frame.get_tag(), 0x23);
        assert_eq!(data_frame.get_data_items().len(), 3);
        let data_item = data_frame.get_data_items()[0].clone();
        assert_eq!(data_item.tag, 0x45);
        assert_eq!(data_item.t_format, TFormat::VecU8(1));
        assert_eq!(String::from(&data_item.t_value), "X");
        let data_item = data_frame.get_data_items()[1].clone();
        assert_eq!(data_item.tag, 0x67);
        assert_eq!(data_item.t_format, TFormat::VecU8(1));
        assert_eq!(String::from(&data_item.t_value), "Y");
        let data_item = data_frame.get_data_items()[2].clone();
        assert_eq!(data_item.tag, 0x89);
        assert_eq!(data_item.t_format, TFormat::VecU8(1));
        assert_eq!(String::from(&data_item.t_value), "Z");
    }

    #[test]
    fn test_construction_avec_un_boolean() {
        /* Construction avec un booléen */
        let mut raw_frame = RawFrame::new_message(0x23);
        raw_frame
            .try_extend_data_item(&DataItem::new(0x45, TFormat::Bool, TValue::Bool(true)))
            .unwrap();
        assert_eq!(
            raw_frame.encode(),
            vec![0x02, 0x23, 0x03, 0x45, 0x11, 0x01, 0x75, 0x03]
        );
        assert_eq!(raw_frame.get_state(), FrameState::Ok);
        let data_frame = DataFrame::try_from(raw_frame).unwrap();
        assert_eq!(data_frame.get_tag(), 0x23);
        assert_eq!(data_frame.get_data_items().len(), 1);
        let data_item = data_frame.get_data_items()[0].clone();
        assert_eq!(data_item.tag, 0x45);
        assert_eq!(data_item.t_format, TFormat::Bool);
        assert!(bool::from(&data_item.t_value));
    }

    #[test]
    fn test_construction_avec_4_booleans() {
        /* Construction avec 4 booléens */
        let mut raw_frame = RawFrame::new_message(0x01);
        raw_frame
            .try_extend_data_item(&DataItem::new(0x23, TFormat::Bool, TValue::Bool(true)))
            .unwrap();
        raw_frame
            .try_extend_data_item(&DataItem::new(0x45, TFormat::Bool, TValue::Bool(false)))
            .unwrap();
        raw_frame
            .try_extend_data_item(&DataItem::new(0x67, TFormat::Bool, TValue::Bool(false)))
            .unwrap();
        raw_frame
            .try_extend_data_item(&DataItem::new(0x89, TFormat::Bool, TValue::Bool(true)))
            .unwrap();
        assert_eq!(
            raw_frame.encode(),
            vec![
                0x02, 0x01, 0x0C, 0x23, 0x11, 0x01, 0x45, 0x11, 0x00, 0x67, 0x11, 0x00, 0x89, 0x11,
                0x01, 0x85, 0x03
            ]
        );
        assert_eq!(raw_frame.get_state(), FrameState::Ok);
        let data_frame = DataFrame::try_from(raw_frame).unwrap();
        assert_eq!(data_frame.get_tag(), 0x01);
        assert_eq!(data_frame.get_data_items().len(), 4);
        let data_item = data_frame.get_data_items()[0].clone();
        assert_eq!(data_item.tag, 0x23);
        assert_eq!(data_item.t_format, TFormat::Bool);
        assert!(bool::from(&data_item.t_value));
        let data_item = data_frame.get_data_items()[1].clone();
        assert_eq!(data_item.tag, 0x45);
        assert_eq!(data_item.t_format, TFormat::Bool);
        assert!(!bool::from(&data_item.t_value));
        let data_item = data_frame.get_data_items()[2].clone();
        assert_eq!(data_item.tag, 0x67);
        assert_eq!(data_item.t_format, TFormat::Bool);
        assert!(!bool::from(&data_item.t_value));
        let data_item = data_frame.get_data_items()[3].clone();
        assert_eq!(data_item.tag, 0x89);
        assert_eq!(data_item.t_format, TFormat::Bool);
        assert!(bool::from(&data_item.t_value));
    }

    #[test]
    fn test_construction_avec_un_unsigned_char() {
        /* Construction avec un unsigned char */
        let mut raw_frame = RawFrame::new_message(0x23);
        raw_frame
            .try_extend_data_item(&DataItem::new(0x45, TFormat::U8, TValue::U8(123)))
            .unwrap();
        assert_eq!(
            raw_frame.encode(),
            vec![0x02, 0x23, 0x03, 0x45, 0x01, 0x7B, 0x1F, 0x03]
        );
        assert_eq!(raw_frame.get_state(), FrameState::Ok);
        let data_frame = DataFrame::try_from(raw_frame).unwrap();
        assert_eq!(data_frame.get_tag(), 0x23);
        assert_eq!(data_frame.get_data_items().len(), 1);
        let data_item = data_frame.get_data_items()[0].clone();
        assert_eq!(data_item.tag, 0x45);
        assert_eq!(data_item.t_format, TFormat::U8);
        assert_eq!(u8::from(&data_item.t_value), 123);
    }

    #[test]
    fn test_construction_avec_un_signed_char() {
        /* Construction avec un signed char */
        let mut raw_frame = RawFrame::new_message(0x23);
        raw_frame
            .try_extend_data_item(&DataItem::new(0x45, TFormat::I8, TValue::I8(-123)))
            .unwrap();
        assert_eq!(
            raw_frame.encode(),
            vec![0x02, 0x23, 0x03, 0x45, 0x41, 0x85, 0xA1, 0x03]
        );
        assert_eq!(raw_frame.get_state(), FrameState::Ok);
        let data_frame = DataFrame::try_from(raw_frame).unwrap();
        assert_eq!(data_frame.get_tag(), 0x23);
        assert_eq!(data_frame.get_data_items().len(), 1);
        let data_item = data_frame.get_data_items()[0].clone();
        assert_eq!(data_item.tag, 0x45);
        assert_eq!(data_item.t_format, TFormat::I8);
        assert_eq!(i8::from(&data_item.t_value), -123);
    }

    #[test]
    fn test_construction_avec_un_unsigned_int() {
        /* Construction avec un unsigned int */
        let mut raw_frame = RawFrame::new_message(0x23);
        raw_frame
            .try_extend_data_item(&DataItem::new(0x45, TFormat::U16, TValue::U16(123)))
            .unwrap();
        assert_eq!(
            raw_frame.encode(),
            vec![0x02, 0x23, 0x04, 0x45, 0x02, 0x00, 0x7B, 0x1B, 0x03]
        );
        assert_eq!(raw_frame.get_state(), FrameState::Ok);
        let data_frame = DataFrame::try_from(raw_frame).unwrap();
        assert_eq!(data_frame.get_tag(), 0x23);
        assert_eq!(data_frame.get_data_items().len(), 1);
        let data_item = data_frame.get_data_items()[0].clone();
        assert_eq!(data_item.tag, 0x45);
        assert_eq!(data_item.t_format, TFormat::U16);
        assert_eq!(u16::from(&data_item.t_value), 123);
    }

    #[test]
    fn test_construction_avec_un_signed_int() {
        /* Construction avec un signed int */
        let mut raw_frame = RawFrame::new_message(0x23);
        raw_frame
            .try_extend_data_item(&DataItem::new(0x45, TFormat::I16, TValue::I16(-123)))
            .unwrap();
        assert_eq!(
            raw_frame.encode(),
            vec![0x02, 0x23, 0x04, 0x45, 0x42, 0xFF, 0x85, 0x5A, 0x03]
        );
        assert_eq!(raw_frame.get_state(), FrameState::Ok);
        let data_frame = DataFrame::try_from(raw_frame).unwrap();
        assert_eq!(data_frame.get_tag(), 0x23);
        assert_eq!(data_frame.get_data_items().len(), 1);
        let data_item = data_frame.get_data_items()[0].clone();
        assert_eq!(data_item.tag, 0x45);
        assert_eq!(data_item.t_format, TFormat::I16);
        assert_eq!(i16::from(&data_item.t_value), -123);
    }

    #[test]
    fn test_construction_avec_un_unsigned_long_int() {
        /* Construction avec un unsigned long int */
        let mut raw_frame = RawFrame::new_message(0x23);
        raw_frame
            .try_extend_data_item(&DataItem::new(0x45, TFormat::U32, TValue::U32(123)))
            .unwrap();
        assert_eq!(
            raw_frame.encode(),
            vec![0x02, 0x23, 0x06, 0x45, 0x04, 0x00, 0x00, 0x00, 0x7B, 0x1F, 0x03]
        );
        assert_eq!(raw_frame.get_state(), FrameState::Ok);
        let data_frame = DataFrame::try_from(raw_frame).unwrap();
        assert_eq!(data_frame.get_tag(), 0x23);
        assert_eq!(data_frame.get_data_items().len(), 1);
        let data_item = data_frame.get_data_items()[0].clone();
        assert_eq!(data_item.tag, 0x45);
        assert_eq!(data_item.t_format, TFormat::U32);
        assert_eq!(u32::from(&data_item.t_value), 123);
    }

    #[test]
    fn test_construction_avec_un_signed_long_int() {
        /* Construction avec un signed long int */
        let mut raw_frame = RawFrame::new_message(0x23);
        raw_frame
            .try_extend_data_item(&DataItem::new(0x45, TFormat::I32, TValue::I32(-123)))
            .unwrap();
        assert_eq!(
            raw_frame.encode(),
            vec![0x02, 0x23, 0x06, 0x45, 0x44, 0xFF, 0xFF, 0xFF, 0x85, 0x5E, 0x03]
        );
        assert_eq!(raw_frame.get_state(), FrameState::Ok);
        let data_frame = DataFrame::try_from(raw_frame).unwrap();
        assert_eq!(data_frame.get_tag(), 0x23);
        assert_eq!(data_frame.get_data_items().len(), 1);
        let data_item = data_frame.get_data_items()[0].clone();
        assert_eq!(data_item.tag, 0x45);
        assert_eq!(data_item.t_format, TFormat::I32);
        assert_eq!(i32::from(&data_item.t_value), -123);
    }

    #[test]
    fn test_construction_avec_un_float() {
        /* Construction avec un float */
        let mut raw_frame = RawFrame::new_message(0x23);
        raw_frame
            .try_extend_data_item(&DataItem::new(0x45, TFormat::F32, TValue::F32(-123.0)))
            .unwrap();
        assert_eq!(
            raw_frame.encode(),
            vec![0x02, 0x23, 0x06, 0x45, 0x64, 0xC2, 0xF6, 0x00, 0x00, 0x30, 0x03]
        );
        assert_eq!(raw_frame.get_state(), FrameState::Ok);
        let data_frame = DataFrame::try_from(raw_frame).unwrap();
        assert_eq!(data_frame.get_tag(), 0x23);
        assert_eq!(data_frame.get_data_items().len(), 1);
        let data_item = data_frame.get_data_items()[0].clone();
        assert_eq!(data_item.tag, 0x45);
        assert_eq!(data_item.t_format, TFormat::F32);
        assert_f32_near!(f32::from(&data_item.t_value), -123.0);
    }

    #[test]
    fn test_construction_avec_un_unsigned_long_long() {
        /* Construction avec un unsigned long long int */
        let mut raw_frame = RawFrame::new_message(0x23);
        raw_frame
            .try_extend_data_item(&DataItem::new(0x45, TFormat::U64, TValue::U64(123)))
            .unwrap();
        assert_eq!(
            raw_frame.encode(),
            vec![
                0x02, 0x23, 0x0A, 0x45, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7B, 0x1F,
                0x03
            ]
        );
        assert_eq!(raw_frame.get_state(), FrameState::Ok);
        let data_frame = DataFrame::try_from(raw_frame).unwrap();
        assert_eq!(data_frame.get_tag(), 0x23);
        assert_eq!(data_frame.get_data_items().len(), 1);
        let data_item = data_frame.get_data_items()[0].clone();
        assert_eq!(data_item.tag, 0x45);
        assert_eq!(data_item.t_format, TFormat::U64);
        assert_eq!(u64::from(&data_item.t_value), 123);
    }

    #[test]
    fn test_construction_avec_un_double_float() {
        /* Construction avec un double float */
        let mut raw_frame = RawFrame::new_message(0x23);
        raw_frame
            .try_extend_data_item(&DataItem::new(0x45, TFormat::F64, TValue::F64(-123.0)))
            .unwrap();
        assert_eq!(
            raw_frame.encode(),
            vec![
                0x02, 0x23, 0x0A, 0x45, 0x68, 0xC0, 0x5E, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5A,
                0x03
            ]
        );
        assert_eq!(raw_frame.get_state(), FrameState::Ok);
        let data_frame = DataFrame::try_from(raw_frame).unwrap();
        assert_eq!(data_frame.get_tag(), 0x23);
        assert_eq!(data_frame.get_data_items().len(), 1);
        let data_item = data_frame.get_data_items()[0].clone();
        assert_eq!(data_item.tag, 0x45);
        assert_eq!(data_item.t_format, TFormat::F64);
        assert_f64_near!(f64::from(&data_item.t_value), -123.0);
    }

    #[test]
    fn test_construction_avec_un_tlv() {
        /* Construction avec un TLV */
        /* Idem construction avec un booléen */
    }

    #[test]
    fn test_construction_trop_longue() {
        /* Construction trop longue */
        let mut raw_frame = RawFrame::new_message(0x23);
        let mut b_overflow = false;
        for _ in 0..10 {
            /* Data max len=250 */
            if let Ok(()) = raw_frame.try_extend_data_item(&DataItem::new(
                0x01,
                TFormat::VecU8(26),
                TValue::VecU8(26, string_to_vec_u8("ABCDEFGHIJKLMNOPQRSTUVWXYZ")),
            )) {
            } else {
                b_overflow = true;
                break;
            };
        }
        assert!(b_overflow, "Overflow non détecté");
        assert_eq!(raw_frame.get_state(), FrameState::Ok);
    }

    #[test]
    fn test_decodage_simple_ack() {
        /* Décodage d'une trame simple ACK */
        let raw_frame = RawFrame::new_ack();
        assert_eq!(raw_frame.encode().len(), 1);
        assert_eq!(raw_frame.get_state(), FrameState::Ok);
        let data_frame = DataFrame::try_from(raw_frame).unwrap();
        assert!(data_frame.is_simple_ack());
        assert!(!data_frame.is_simple_nack());
        assert!(!data_frame.is_message());
        assert_eq!(data_frame.get_tag(), ACK);
        assert_eq!(data_frame.get_data_items().len(), 0);
    }

    #[test]
    fn test_decodage_simple_nack() {
        /* Décodage d'une trame simple NACK */
        let raw_frame = RawFrame::new_nack();
        assert_eq!(raw_frame.encode().len(), 1);
        assert_eq!(raw_frame.get_state(), FrameState::Ok);
        let data_frame = DataFrame::try_from(raw_frame).unwrap();
        assert!(!data_frame.is_simple_ack());
        assert!(data_frame.is_simple_nack());
        assert!(!data_frame.is_message());
        assert_eq!(data_frame.get_tag(), NACK);
        assert_eq!(data_frame.get_data_items().len(), 0);
    }

    #[test]
    fn test_conversion_boolean_true() {
        /* Conversion d'un BOOL à TRUE */
        let t_value = TValue::Bool(true);
        assert!(bool::from(&t_value));
        assert!(u8::from(&t_value) != 0);
        assert!(i8::from(&t_value) != 0);
        assert!(u16::from(&t_value) != 0);
        assert!(i16::from(&t_value) != 0);
        assert!(u32::from(&t_value) != 0);
        assert!(i32::from(&t_value) != 0);
        assert!(f32::from(&t_value) != 0.0);
        assert!(f64::from(&t_value) != 0.0);
        assert_eq!(String::from(&t_value).to_uppercase(), "TRUE");
    }

    #[test]
    fn test_conversion_boolean_false() {
        /* Conversion d'un BOOL à FALSE */
        let t_value = TValue::Bool(false);
        assert!(!bool::from(&t_value));
        assert!(u8::from(&t_value) == 0);
        assert!(i8::from(&t_value) == 0);
        assert!(u16::from(&t_value) == 0);
        assert!(i16::from(&t_value) == 0);
        assert!(u32::from(&t_value) == 0);
        assert!(i32::from(&t_value) == 0);
        assert!(f32::from(&t_value) == 0.0);
        assert!(f64::from(&t_value) == 0.0);
        assert_eq!(String::from(&t_value).to_uppercase(), "FALSE");
    }

    #[test]
    fn test_conversion_u8() {
        /* Conversion d'un U8 */
        let t_value = TValue::U8(123);
        assert!(bool::from(&t_value));
        assert_eq!(u8::from(&t_value), 123);
        assert_eq!(i8::from(&t_value), 123);
        assert_eq!(u16::from(&t_value), 123);
        assert_eq!(i16::from(&t_value), 123);
        assert_eq!(u32::from(&t_value), 123);
        assert_eq!(i32::from(&t_value), 123);
        assert_f32_near!(f32::from(&t_value), 123.0);
        assert_f64_near!(f64::from(&t_value), 123.0);
        assert_eq!(String::from(&t_value), "123");
    }

    #[test]
    fn test_conversion_i8() {
        /* Conversion d'un I8 */
        let t_value = TValue::I8(-123);
        assert!(bool::from(&t_value));
        assert_eq!(u8::from(&t_value), 0); // 0x85 dans la version AFSEC+
        assert_eq!(i8::from(&t_value), -123);
        assert_eq!(u16::from(&t_value), 0); // 0x0085 dans la version AFSEC+
        assert_eq!(i16::from(&t_value), -123);
        assert_eq!(u32::from(&t_value), 0); // 0x0000_0085 dans la version AFSEC+
        assert_eq!(i32::from(&t_value), -123);
        assert_f32_near!(f32::from(&t_value), -123.0);
        assert_f64_near!(f64::from(&t_value), -123.0);
        assert_eq!(String::from(&t_value), "-123");
    }

    #[test]
    fn test_conversion_u16() {
        /* Conversion d'un U16 */
        let t_value = TValue::U16(123);
        assert!(bool::from(&t_value));
        assert_eq!(u8::from(&t_value), 123);
        assert_eq!(i8::from(&t_value), 123);
        assert_eq!(u16::from(&t_value), 123);
        assert_eq!(i16::from(&t_value), 123);
        assert_eq!(u32::from(&t_value), 123);
        assert_eq!(i32::from(&t_value), 123);
        assert_f32_near!(f32::from(&t_value), 123.0);
        assert_f64_near!(f64::from(&t_value), 123.0);
        assert_eq!(String::from(&t_value), "123");
    }

    #[test]
    fn test_conversion_i16() {
        /* Conversion d'un I16 */
        let t_value = TValue::I16(-123);
        assert!(bool::from(&t_value));
        assert_eq!(u8::from(&t_value), 0); // 0x85 dans la version AFSEC+
        assert_eq!(i8::from(&t_value), -123);
        assert_eq!(u16::from(&t_value), 0); // 0xFF85 dans la version AFSEC+
        assert_eq!(i16::from(&t_value), -123);
        assert_eq!(u32::from(&t_value), 0); // 0x0000_FF85 dans la version AFSEC+
        assert_eq!(i32::from(&t_value), -123);
        assert_f32_near!(f32::from(&t_value), -123.0);
        assert_f64_near!(f64::from(&t_value), -123.0);
        assert_eq!(String::from(&t_value), "-123");
    }

    #[test]
    fn test_conversion_u32() {
        /* Conversion d'un U32 */
        let t_value = TValue::U32(123);
        assert!(bool::from(&t_value));
        assert_eq!(u8::from(&t_value), 123);
        assert_eq!(i8::from(&t_value), 123);
        assert_eq!(u16::from(&t_value), 123);
        assert_eq!(i16::from(&t_value), 123);
        assert_eq!(u32::from(&t_value), 123);
        assert_eq!(i32::from(&t_value), 123);
        assert_f32_near!(f32::from(&t_value), 123.0);
        assert_f64_near!(f64::from(&t_value), 123.0);
        assert_eq!(String::from(&t_value), "123");
    }

    #[test]
    fn test_conversion_i32() {
        /* Conversion d'un I32 */
        let t_value = TValue::I32(-123);
        assert!(bool::from(&t_value));
        assert_eq!(u8::from(&t_value), 0); // 0x85 dans la version AFSEC+
        assert_eq!(i8::from(&t_value), -123);
        assert_eq!(u16::from(&t_value), 0); // 0xFF85 dans la version AFSEC+
        assert_eq!(i16::from(&t_value), -123);
        assert_eq!(u32::from(&t_value), 0); // 0xFFFF_FF85 dans la version AFSEC+
        assert_eq!(i32::from(&t_value), -123);
        assert_f32_near!(f32::from(&t_value), -123.0);
        assert_f64_near!(f64::from(&t_value), -123.0);
        assert_eq!(String::from(&t_value), "-123");
    }

    #[test]
    fn test_conversion_u64() {
        /* Conversion d'un U64 */
        let t_value = TValue::U64(123);
        assert!(bool::from(&t_value));
        assert_eq!(u8::from(&t_value), 123);
        assert_eq!(i8::from(&t_value), 123);
        assert_eq!(u16::from(&t_value), 123);
        assert_eq!(i16::from(&t_value), 123);
        assert_eq!(u32::from(&t_value), 123);
        assert_eq!(i32::from(&t_value), 123);
        assert_f32_near!(f32::from(&t_value), 123.0);
        assert_f64_near!(f64::from(&t_value), 123.0);
        assert_eq!(String::from(&t_value), "123");
    }

    #[test]
    fn test_conversion_f32() {
        /* Conversion d'un F32 */
        let t_value = TValue::F32(-123.0);
        assert!(bool::from(&t_value));
        assert_eq!(u8::from(&t_value), 0); // 0x85 dans la version AFSEC+
        assert_eq!(i8::from(&t_value), -123);
        assert_eq!(u16::from(&t_value), 0); // 0xFF85 dans la version AFSEC+
        assert_eq!(i16::from(&t_value), -123);
        assert_eq!(u32::from(&t_value), 0); // 0xFFFF_FF85 dans la version AFSEC+
        assert_eq!(i32::from(&t_value), -123);
        assert_f32_near!(f32::from(&t_value), -123.0);
        assert_f64_near!(f64::from(&t_value), -123.0);
        assert_eq!(String::from(&t_value), "-123"); // -123.000000 dans la version AFSEC+
    }

    #[test]
    fn test_conversion_f64() {
        /* Conversion d'un F64 */
        let t_value = TValue::F64(-123.0);
        assert!(bool::from(&t_value));
        assert_eq!(u8::from(&t_value), 0); // 0x85 dans la version AFSEC+
        assert_eq!(i8::from(&t_value), -123);
        assert_eq!(u16::from(&t_value), 0); // 0xFF85 dans la version AFSEC+
        assert_eq!(i16::from(&t_value), -123);
        assert_eq!(u32::from(&t_value), 0); // 0xFFFF_FF85 dans la version AFSEC+
        assert_eq!(i32::from(&t_value), -123);
        assert_f32_near!(f32::from(&t_value), -123.0);
        assert_f64_near!(f64::from(&t_value), -123.0);
        assert_eq!(String::from(&t_value), "-123"); // -123.000000 dans la version AFSEC+
    }

    #[test]
    fn test_conversion_string_positif() {
        /* Conversion d'une string (entier positif) */
        let t_value = TValue::VecU8(3, string_to_vec_u8("123"));
        assert!(bool::from(&t_value));
        assert_eq!(u8::from(&t_value), 123);
        assert_eq!(i8::from(&t_value), 123);
        assert_eq!(u16::from(&t_value), 123);
        assert_eq!(i16::from(&t_value), 123);
        assert_eq!(u32::from(&t_value), 123);
        assert_eq!(i32::from(&t_value), 123);
        assert_f32_near!(f32::from(&t_value), 123.0);
        assert_f64_near!(f64::from(&t_value), 123.0);
        assert_eq!(String::from(&t_value), "123");
    }

    #[test]
    fn test_conversion_string_negative() {
        /* Conversion d'une string (entier negatif) */
        let t_value = TValue::VecU8(4, string_to_vec_u8("-123"));
        assert!(bool::from(&t_value));
        assert_eq!(u8::from(&t_value), 0); // 0x85 dans la version AFSEC+
        assert_eq!(i8::from(&t_value), -123);
        assert_eq!(u16::from(&t_value), 0); // 0xFF85 dans la version AFSEC+
        assert_eq!(i16::from(&t_value), -123);
        assert_eq!(u32::from(&t_value), 0); // 0xFFFF_FF85 dans la version AFSEC+
        assert_eq!(i32::from(&t_value), -123);
        assert_f32_near!(f32::from(&t_value), -123.0);
        assert_f64_near!(f64::from(&t_value), -123.0);
        assert_eq!(String::from(&t_value), "-123");
    }
}
