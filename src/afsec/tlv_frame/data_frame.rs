//! Encodage et décodage des trames au format TLV (Tag + Length + Value)
//! échangées via la liaison série entre l'AFSEC+ et l'ICOM
//!
//! Ce module s'appuie sur la structure [`RawFrame`] qui gère le transport des messages octet par octet.
//!
//! Le message est ensuite décoder/encoder par une structure logique de son contenu avec :
//!
//! * tag : Sujet principal du message
//! * `Vec<DataItem>` : Liste des données dans le message
//!
//! Les données du message sont portées par `DataItem`

use std::convert;
use std::fmt;

use super::{DataItem, RawFrame};

/// Abstraction logique du contenu d'une trame TLV
#[derive(Debug)]
pub enum DataFrame {
    /// Message simple ACK
    SimpleACK,

    //. Message simple NACK
    SimpleNACK,

    /// tag  + données
    Message(u8, Vec<DataItem>),
}

impl fmt::Display for DataFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ret;
        match self {
            DataFrame::SimpleACK => ret = "ACK".to_string(),
            DataFrame::SimpleNACK => ret = "NACK".to_string(),
            DataFrame::Message(tag, datas) => {
                ret = format!("T={tag} datas=[");
                for data in datas {
                    ret += &format!("{data}, ");
                }
                ret += "]";
            }
        }
        write!(f, "{ret}")
    }
}

impl convert::TryFrom<RawFrame> for DataFrame {
    type Error = &'static str;

    fn try_from(value: RawFrame) -> Result<Self, Self::Error> {
        match value {
            RawFrame::Empty => Err("Empty frame"),
            RawFrame::Ack => Ok(DataFrame::SimpleACK),
            RawFrame::AckAndJunk(_)
            | RawFrame::NackAndJunk(_)
            | RawFrame::OkAndJunk(_, _, _, _, _)
            | RawFrame::Junk(_) => Err("Junk frame"),
            RawFrame::Nack => Ok(DataFrame::SimpleNACK),
            RawFrame::Stx
            | RawFrame::Tag(_)
            | RawFrame::TagLen(_, _)
            | RawFrame::TagLenValue(_, _, _)
            | RawFrame::Xor(_, _, _, _) => Err("Building frame"),
            RawFrame::Ok(tag, _, values, _) => match DataItem::decode_all(&values) {
                Ok(data_items) => Ok(DataFrame::Message(tag, data_items)),
                Err(s) => Err(s),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::t_data::{TFormat, TValue};

    #[test]
    fn test_decode_ack() {
        let raw_frame = RawFrame::new_ack();
        let result_data_frame = DataFrame::try_from(raw_frame);
        assert!(result_data_frame.is_ok());
        let data_frame = result_data_frame.unwrap();
        if let DataFrame::SimpleACK = data_frame {
        } else {
            panic!("Mauvais décodage d'un ACK")
        }
    }

    #[test]
    fn test_decode_nack() {
        let raw_frame = RawFrame::new_nack();
        let result_data_frame = DataFrame::try_from(raw_frame);
        assert!(result_data_frame.is_ok());
        let data_frame = result_data_frame.unwrap();
        if let DataFrame::SimpleNACK = data_frame {
        } else {
            panic!("Mauvais décodage d'un NACK")
        }
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn test_encode_message() {
        // Contenu du message
        let message_tag = 1;
        let data_item = DataItem {
            tag: 2,
            t_format: TFormat::U16,
            t_value: TValue::U16(123),
        };

        // Création de la raw_frame
        let mut raw_frame = RawFrame::new_message(message_tag);
        raw_frame.try_extend_data_item(&data_item).unwrap();

        // Décodage de la raw_frame en data_frame
        let result_data_frame = DataFrame::try_from(raw_frame);

        // On vérifie que la data_frame à bien identifié le contenu du message créé
        assert!(result_data_frame.is_ok());
        let data_frame = result_data_frame.unwrap();
        if let DataFrame::Message(tag, data_items) = data_frame {
            assert_eq!(message_tag, tag);
            assert_eq!(data_items.len(), 1);
            let data_item = data_items[0].clone();
            assert_eq!(data_item.tag, 2);
            assert_eq!(data_item.t_format, TFormat::U16);
            if let TValue::U16(value) = data_item.t_value {
                assert_eq!(value, 123);
            } else {
                panic!("Mauvais décodage de la valeur d'une donnée dans un message")
            }
        } else {
            panic!("Mauvais décodage d'un message")
        }
    }

    #[test]
    fn test_overflow_message() {
        // Contenu du message
        let message_tag = 1;
        let data_item = DataItem {
            tag: 2,
            t_format: TFormat::U16,
            t_value: TValue::U16(123),
        };

        // Création de la raw_frame
        let mut raw_frame = RawFrame::new_message(message_tag);

        // On vérifie qu'on ne peut pas ajouter indéfiniment des `data_item`
        for _ in 0..256 {
            if raw_frame.try_extend_data_item(&data_item).is_err() {
                return;
            }
        }

        panic!("Overflow construction du message non détectée");
    }
}
