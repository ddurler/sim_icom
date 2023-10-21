//! Encodage et décodage des trames au format TLV (Tag + Length + Value)
//! échangées via la liaison série entre l'AFSEC+ et l'ICOM
//!
//! Ce module s'appuie sur la structure [`RawFrame`] qui gère le transport des messages octet par octet.
//!
//! Le message est ensuite décodé/encodé dans une structure logique de son contenu avec:
//!
//! * tag: Sujet principal du message
//! * `Vec<DataItem>`: Liste des données dans le message
//!
//! Les données du message sont portées par `DataItem`

use std::convert;
use std::fmt;

use super::{DataItem, FrameError, RawFrame, ACK, NACK};

/// Abstraction logique du contenu d'une trame TLV
#[derive(Debug)]
pub enum DataFrame {
    /// Message simple ACK
    SimpleACK,

    /// Message simple NACK
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
    type Error = FrameError;

    fn try_from(value: RawFrame) -> Result<Self, Self::Error> {
        match value {
            RawFrame::Empty => Err(FrameError::IsEmpty),

            RawFrame::Ack => Ok(DataFrame::SimpleACK),

            RawFrame::AckAndJunk(_)
            | RawFrame::NackAndJunk(_)
            | RawFrame::OkAndJunk(_, _, _, _, _)
            | RawFrame::Junk(_) => Err(FrameError::IsJunk),

            RawFrame::Nack => Ok(DataFrame::SimpleNACK),

            RawFrame::Stx
            | RawFrame::Tag(_)
            | RawFrame::TagLen(_, _)
            | RawFrame::TagLenValue(_, _, _)
            | RawFrame::Xor(_, _, _, _) => Err(FrameError::IsBuilding),

            RawFrame::Ok(tag, _, data_items, _) => match DataItem::decode_all(&data_items) {
                Ok(data_items) => Ok(DataFrame::Message(tag, data_items)),
                Err(_) => Err(FrameError::BadDataItem),
            },
        }
    }
}

impl DataFrame {
    /// Retourne true s'il s'agit d'une trame simple ACK
    #[allow(dead_code)]
    pub fn is_simple_ack(&self) -> bool {
        matches!(self, DataFrame::SimpleACK)
    }

    /// Retourne true s'il s'agit d'une trame simple NACK
    #[allow(dead_code)]
    pub fn is_simple_nack(&self) -> bool {
        matches!(self, DataFrame::SimpleNACK)
    }

    /// Retourne true s'il s'agit d'une trame message tag + `Vec<DataItem>`
    #[allow(dead_code)]
    pub fn is_message(&self) -> bool {
        matches!(self, DataFrame::Message(_, _))
    }

    /// Retourne le type de trame (tag) ou ACK si simple ACK ou NACK si simple NACK
    #[allow(dead_code)]
    pub fn get_tag(&self) -> u8 {
        match self {
            DataFrame::SimpleACK => ACK,
            DataFrame::SimpleNACK => NACK,
            DataFrame::Message(tag, _) => *tag,
        }
    }

    /// Retourne un `Vec<DataItem>` du message
    #[allow(dead_code)]
    pub fn get_data_items(&self) -> Vec<DataItem> {
        if let DataFrame::Message(_, data_items) = self {
            data_items.clone()
        } else {
            vec![]
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
        assert!(data_frame.is_simple_ack());
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
        assert!(data_frame.is_simple_nack());
    }

    #[test]
    fn test_encode_message() {
        // Contenu du message
        let message_tag = 1;
        let data_item = DataItem::new(2, TValue::U16(123));

        // Création de la raw_frame
        let mut raw_frame = RawFrame::new_message(message_tag);
        raw_frame.try_extend_data_item(&data_item).unwrap();

        // Décodage de la raw_frame en data_frame
        let result_data_frame = DataFrame::try_from(raw_frame);
        assert!(result_data_frame.is_ok());
        let data_frame = result_data_frame.unwrap();

        // On vérifie que la data_frame a bien identifié le contenu du message créé
        assert!(data_frame.is_message());
        assert_eq!(data_frame.get_tag(), message_tag);
        let data_items = data_frame.get_data_items();
        assert_eq!(data_items.len(), 1);
        let data_item = data_items[0].clone();
        assert_eq!(data_item.tag, 2);
        assert_eq!(data_item.t_format, TFormat::U16);
        if let TValue::U16(value) = data_item.t_value {
            assert_eq!(value, 123);
        } else {
            panic!("Mauvais décodage de la valeur d'une donnée dans un message")
        }
    }

    #[test]
    #[allow(clippy::cast_lossless)]
    fn test_message_data_items() {
        let nb_data_items = 10;
        // Création d'une table de DataItem's
        let mut test_data_items = vec![];
        for i in 0..nb_data_items {
            // Change de type de valeur selon l'indice
            #[allow(clippy::cast_possible_truncation)]
            let t_value = match i % 4 {
                0 => TValue::U8(i),
                1 => TValue::U16(i as u16),
                2 => TValue::U32(i as u32),
                _ => TValue::U64(i as u64),
            };
            let test_data_item = DataItem::new(i, t_value);
            test_data_items.push(test_data_item);
        }

        // Création de la raw_frame
        let mut raw_frame = RawFrame::new_message(1);
        for i in 0..nb_data_items {
            #[allow(clippy::cast_possible_truncation)]
            raw_frame
                .try_extend_data_item(&test_data_items[i as usize])
                .unwrap();
        }

        // Décodage de la raw_frame en data_frame
        let result_data_frame = DataFrame::try_from(raw_frame);
        assert!(result_data_frame.is_ok());
        let data_frame = result_data_frame.unwrap();
        assert!(data_frame.is_message());
        assert_eq!(data_frame.get_data_items().len(), nb_data_items as usize);

        // Parcourt des data_items
        for (i, data_item) in data_frame.get_data_items().iter().enumerate() {
            assert_eq!(data_item.tag, test_data_items[i].tag);
            assert_eq!(
                data_item.t_format,
                TFormat::from(&test_data_items[i].t_value)
            );
            assert_eq!(
                u16::from(&data_item.t_value),
                u16::from(&test_data_items[i].t_value)
            );
        }
    }

    #[test]
    fn test_overflow_message() {
        // Contenu du message
        let message_tag = 1;
        let data_item = DataItem::new(2, TValue::U16(123));

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
