//! Support pour une donnée contenue dans un message d'une trame TLV
//!
//! Dans un une trame TLV qui porte un tag et des données, la structure `DataItem` représente
//! le contenu d'une donnée de ce message.
//!
//! A noter que ce type de donnée n'existe pas pour un message simple tel que ACK ou NACK
//!
//! Un `DataItem` est également un triplet Tag + Length + Value où :
//!
//! * Tag : Caractérise la donnée
//! * Length est le type de la donnée (qui induit sa longueur). C'est un [`TFormat`]
//! * Value de la donnée. C'est un [`TValue`]

use std::fmt;

use crate::t_data::{be_data, TFormat, TValue};

use super::FrameError;

/// Contenu d'une donnée dans un message d'une trame TLV
#[derive(Clone, Debug)]
pub struct DataItem {
    /// tag de la donnée
    pub tag: u8,

    /// Format de la donnée
    pub t_format: TFormat,

    /// Valeur de la donnée
    pub t_value: TValue,
}

impl fmt::Display for DataItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "T={} L={} V={}", self.tag, self.t_format, self.t_value)
    }
}

impl DataItem {
    /// Extraction du premier `DataItem` d'un `Vec<u8>`
    /// Si OK, retourne le `DataItem` extrait et le nombre d'octets qu'il utilise au début du `Vec<u8>`
    #[allow(dead_code)]
    pub fn decode(values: &[u8]) -> Result<(DataItem, usize), FrameError> {
        if values.len() < 2 {
            return Err(FrameError::BadDataLength);
        }
        let tag = values[0];
        let t_format = TFormat::from(values[1]);
        let data_item_len = 2 + t_format.nb_bytes();
        if values.len() < data_item_len {
            return Err(FrameError::BadDataLength);
        }
        match be_data::decode(t_format, &values[2..]) {
            Ok(t_value) => Ok((
                DataItem {
                    tag,
                    t_format,
                    t_value,
                },
                data_item_len,
            )),
            Err(_) => Err(FrameError::BadDataItem),
        }
    }

    /// Extraction des `DataItem` d'un `Vec<u8>`
    #[allow(dead_code)]
    pub fn decode_all(values: &[u8]) -> Result<Vec<DataItem>, FrameError> {
        let mut data_items = vec![];
        let mut start_index = 0;
        while start_index < values.len() {
            match DataItem::decode(&values[start_index..]) {
                Ok((data_item, len)) => {
                    data_items.push(data_item);
                    start_index += len;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(data_items)
    }

    /// Encode un `DataItem` -> `Vec<u8>`
    #[allow(dead_code)]
    pub fn encode(&self) -> Vec<u8> {
        let tag = self.tag;
        let format = u8::from(self.t_format);
        let value_vec_u8 = be_data::encode(&self.t_value);
        let mut vec_u8 = vec![tag, format];
        vec_u8.extend(value_vec_u8);
        vec_u8
    }
}

#[cfg(test)]
mod tests {
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
            TValue::String(3, "ABC".to_string()),
            TValue::String(0, String::new()),
        ] {
            let tag = 12;
            let t_format = TFormat::from(&t_value);
            let data_item_in = DataItem {
                tag,
                t_format,
                t_value: t_value.clone(),
            };
            let vec_u8 = DataItem::encode(&data_item_in);
            let result_data_item_out = DataItem::decode(&vec_u8);
            assert!(result_data_item_out.is_ok());
            let (data_item_out, _) = result_data_item_out.unwrap();

            assert_eq!(data_item_out.tag, tag);
            assert_eq!(data_item_out.t_format, t_format);
            assert_eq!(String::from(&data_item_out.t_value), String::from(&t_value));
        }
    }

    #[test]
    fn test_multiple_decode() {
        // Liste des DataItem dans un même Vec<u8>
        let test_data_items = vec![
            DataItem {
                tag: 1,
                t_format: TFormat::Bool,
                t_value: TValue::Bool(true),
            },
            DataItem {
                tag: 2,
                t_format: TFormat::U16,
                t_value: TValue::U16(123),
            },
            DataItem {
                tag: 3,
                t_format: TFormat::String(3),
                t_value: TValue::String(3, "ABC".to_string()),
            },
            DataItem {
                tag: 4,
                t_format: TFormat::F32,
                t_value: TValue::F32(1.23),
            },
            DataItem {
                tag: 5,
                t_format: TFormat::I16,
                t_value: TValue::I16(-123),
            },
            DataItem {
                tag: 6,
                t_format: TFormat::String(0),
                t_value: TValue::String(0, String::new()),
            },
            DataItem {
                tag: 7,
                t_format: TFormat::I64,
                t_value: TValue::I64(-1_000_000_000),
            },
        ];

        // Création d'un Vec<u8> contenant tous les test_data_items
        let mut vec_u8 = vec![];
        for data_item in &test_data_items {
            let vec_data_item_u8 = DataItem::encode(data_item);
            vec_u8.extend(vec_data_item_u8);
        }

        // Decode le Vec<u8> ainsi créé
        let res_data_items = DataItem::decode_all(&vec_u8);
        assert!(res_data_items.is_ok());
        let data_items = res_data_items.unwrap();

        // Vérifie le contenu décodé
        assert_eq!(data_items.len(), test_data_items.len());
        for (i, data_item_test) in test_data_items.iter().enumerate() {
            let data_item = &data_items[i];

            assert_eq!(data_item_test.tag, data_item.tag);
            assert_eq!(data_item_test.t_format, data_item.t_format);
            assert_eq!(
                String::from(&data_item_test.t_value),
                String::from(&data_item.t_value)
            );
        }
    }
}
