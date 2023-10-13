//! Encodage et décodage des trames au format TLV (Tag + Length + Value)
//! échangées via la liaison série entre l'AFSEC+ et l'ICOM
//!
//! Ce module s'appuie sur la structure [`RawFrame`] qui gère le transport des messages octet par octet.
//!
//! Le message est ensuite décoder/encoder par une structure logique de son contenu avec :
//!
//! * tag : Sujet principal du message
//! * Vec<DataItem> : Liste des données dans le message
//!
//! Un `DataItem` est également un triplet Tag + Length + Value où :
//!
//! * Tag : Caractérise la donnée
//! * Length est le type de la donnée (qui induit sa longueur). C'est un [`TFormat`]
//! * Value est la donnée. C'est un [`TValue`]

use std::convert;
use std::fmt;

use crate::t_data::{be_data, TFormat, TValue};

use super::RawFrame;

/// Contenu d'une donnée dans une trame TLV
#[derive(Clone, Debug)]
pub struct DataItem {
    /// tag de la donnée
    pub tag: u8,

    /// format de la donnée
    pub t_format: TFormat,

    // Valeur
    pub t_value: TValue,
}

impl fmt::Display for DataItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "T={} L={} V={}", self.tag, self.t_format, self.t_value)
    }
}

impl DataItem {
    // Encode un `DataItem` -> `Vec<u8>`
    pub fn encode(&self) -> Vec<u8> {
        let tag = self.tag;
        let format = u8::from(self.t_format);
        let value_vec_u8 = be_data::encode(&self.t_value);
        let mut vec_u8 = vec![tag, format];
        vec_u8.extend(value_vec_u8);
        vec_u8
    }
}

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

/// Extraction du premier `DataItem` d'un `Vec<u8>`
/// Si OK, retourne le `DataItem` extrait et le nombre d'octets qu'il utilise au début du `Vec<u8>`
fn extract_first_data_item(values: &[u8]) -> Result<(DataItem, usize), &'static str> {
    if values.len() < 2 {
        return Err("Bad data length");
    }
    let tag = values[0];
    let t_format = TFormat::from(values[1]);
    let data_item_len = 2 + t_format.nb_bytes();
    if values.len() < data_item_len {
        return Err("Bad data length");
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
        Err(s) => Err(s),
    }
}

/// Extraction des `DataItem` d'un `Vec<u8>`
fn extract_data_items(values: &[u8]) -> Result<Vec<DataItem>, &'static str> {
    let mut data_items = vec![];
    let mut start_index = 0;
    while start_index < values.len() {
        match extract_first_data_item(&values[start_index..]) {
            Ok((data_item, index)) => {
                data_items.push(data_item);
                start_index += index;
            }
            Err(s) => return Err(s),
        }
    }
    Ok(data_items)
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
            RawFrame::Ok(tag, _, values, _) => match extract_data_items(&values) {
                Ok(data_items) => Ok(DataFrame::Message(tag, data_items)),
                Err(s) => Err(s),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::afsec::tlv_frame::raw_frame;

    #[test]
    fn test_decode_ack() {
        // TODO : Utiliser ici l'encodeur de RawFrame...
        let raw_frame = RawFrame::new(&[raw_frame::ACK]);
        assert_eq!(raw_frame, RawFrame::Ack);
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
        // TODO : Utiliser ici l'encodeur de RawFrame...
        let raw_frame = RawFrame::new(&[raw_frame::NACK]);
        assert_eq!(raw_frame, RawFrame::Nack);
        let result_data_frame = DataFrame::try_from(raw_frame);
        assert!(result_data_frame.is_ok());
        let data_frame = result_data_frame.unwrap();
        if let DataFrame::SimpleNACK = data_frame {
        } else {
            panic!("Mauvais décodage d'un ACK")
        }
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn test_decode_message() {
        // TODO : Utiliser ici l'encodeur de RawFrame...

        // Contenu du message
        let message_tag = 1;
        let data_item = DataItem {
            tag: 2,
            t_format: TFormat::U16,
            t_value: TValue::U16(123),
        };
        let data_item_vec_u8 = data_item.encode();
        let data_item_vec_u8_len = data_item_vec_u8.len() as u8;
        let xor = data_item_vec_u8
            .clone()
            .iter()
            .fold(message_tag ^ data_item_vec_u8_len, |a, b| a ^ *b);

        // Création de la raw_frame
        let mut message_vec_u8 = vec![raw_frame::STX, message_tag, data_item_vec_u8_len];
        message_vec_u8.extend(data_item_vec_u8.clone());
        message_vec_u8.extend(&[xor, raw_frame::ETX]);

        let raw_frame = RawFrame::new(&message_vec_u8);
        // On s'assure que cette raw_frame est bien ce qu'on a voulu créer
        assert_eq!(
            raw_frame,
            RawFrame::Ok(
                message_tag,
                data_item_vec_u8_len,
                data_item_vec_u8.clone(),
                xor
            )
        );

        // Décodage de la raw_frame en data_frame
        let result_data_frame = DataFrame::try_from(raw_frame);

        // On vérifie que la data_frame à bien identifier le contenu du message créé
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
}
