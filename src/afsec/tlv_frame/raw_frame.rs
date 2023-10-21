//! Encodage et décodage des trames 'brutes' telles qu'échangées via la liaison série
//! entre l'AFSEC+ et l'ICOM (support `Vec<u8>` sous-jacent)
//!
//! Ce module est prévu pour construire une trame TLV au fur et à mesure que des octets sont reçus:
//!
//! ```
//! let frame = RawFrame::default();
//! frame.push(octet);
//! ```
//!
//! On peut alors obtenir l'état en cours de la construction pour statuer ce qu'il convient de faire pour
//! poursuivre la construction:
//!
//! * `FrameState::Empty` Rien reçu: Abandoner si timeout
//! * `FrameState::Building` Des octets reçus. La trame semble correcte mais on n'a pas tout reçu: Continuer
//!    la construction en cours ou abandonner si timeout
//! * `FrameState::Ok` Réception d'une trame complète et correcte:  On peut traiter son contenu
//! * `FrameState::Junk` Réception d'octets qu'on ne sait pas interpréter: Abandonner
//!
//! Si des octets surnuméraires sont reçus après avoir identifié une trame correcte, la trame
//! devient `FrameState::Junk`. On peut alors tenter `frame.remove_junk` pour retrouver la trame correcte
//! du début.
//!
//! Ce module est également prévu pour interagir avec une vue 'logique' de la trame via `DataFrame`
//! pour décoder le contenu et encoder la réponse
//!
//! Enfin, ce module propose les primitives nécessaires pour encoder la réponse élaborée en créant
//! un message simple ACK ou NACK ou en créant un message avec un tag de message des des `DataItem`
//!
//! ```
//! // Simple ACK
//! let frame_ack = RawFrame::new_ack();
//!
//! // Simple NAK
//! let frame_nack = RawFrame::new_nack();
//!
//! // Message tag=10 et une donnée TLV = {tag=1, L=booléen, V=true}
//! let frame_message = RawFrame::new_message(tag: 10);
//! frame_message.try_append_data_item(DataItem::new(1, TFormat::Bool, TValue::Bool(true)));
//! ```
//!
//! Un `Vec<u8>` des octets correspondant à la la `RawFrame` est obtenu par `RawFrame::encode`

use std::fmt;

use super::DataItem;

/// Longueur max des données d'un message TLV
const RAW_FRAME_MAX_LEN: usize = 250;

/// Début de message
pub const STX: u8 = 0x02;

/// Fin de message
pub const ETX: u8 = 0x03;

/// Acquit de message
pub const ACK: u8 = 0x06;

/// Non-acquit de message
pub const NACK: u8 = 0x15;

/// Erreur lors de l'encodage ou du décodage d'une trame
#[derive(Debug)]
pub enum FrameError {
    /// La trame est vide
    IsEmpty,

    /// La trame n'est pas correcte
    IsJunk,

    /// La trame n'est pas complètement construite
    IsBuilding,

    /// La trame n'est pas un message correct
    IsNotOk,

    /// Inconsistance longueur des `DataItem`
    BadDataLength,

    /// Inconsistance décodage des `DataItem`
    BadDataItem,

    /// Overflow de la longueur max. d'une trame
    MaxLengthOverflow,
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            FrameError::IsEmpty => "La trame est vide",
            FrameError::IsJunk => "La trame n'est pas correcte",
            FrameError::IsBuilding => "La trame n'est pas complètement construite",
            FrameError::IsNotOk => "La trame n'est pas un message correct",
            FrameError::BadDataLength => "Inconsistance longueur des `DataItem` de la trame",
            FrameError::BadDataItem => "Inconsistance décodage des `DataItem` de la trame",
            FrameError::MaxLengthOverflow => "Overflow de la longueur max. d'une trame",
        };
        write!(f, "{s}")
    }
}

/// Énumération de l''état' courant de la construction de la trame
#[derive(Debug, PartialEq)]
pub enum FrameState {
    /// Rien reçu (la trame est vide)
    Empty,

    /// Construction en cours d'une trame qui semble correcte jusqu'à présent
    Building,

    /// Réception d'une trame complète, correcte et valide
    Ok,

    /// Réception d'une trame non exploitable
    Junk,
}

impl fmt::Display for FrameState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FrameState::Empty => write!(f, "Empty"),
            FrameState::Building => write!(f, "Building"),
            FrameState::Ok => write!(f, "OK"),
            FrameState::Junk => write!(f, "Junk"),
        }
    }
}

/// Structure pour encoder et décoder une trame brute au format `Vec<u8>`
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub enum RawFrame {
    // Frame vide
    #[default]
    Empty,

    // Contient ACK
    Ack,

    // Contient ACK et d'autres octets
    AckAndJunk(Vec<u8>),

    // Contient NACK
    Nack,

    // Contient NACK et d'autres octets
    NackAndJunk(Vec<u8>),

    // Contient STX
    Stx,

    // Contient STX + Tag
    Tag(u8),

    // Contient STX + Tag + Len
    TagLen(u8, u8),

    // Contient STX + Tag + Len + Data
    TagLenValue(u8, u8, Vec<u8>),

    // Contient STX + Tag + Len + Data + XOR correct
    Xor(u8, u8, Vec<u8>, u8),

    // Message complet avec STX + Tag + Len + Values + XorOK + ETX
    Ok(u8, u8, Vec<u8>, u8),

    // Message complet suivi d'autres octets
    OkAndJunk(u8, u8, Vec<u8>, u8, Vec<u8>),

    // Rien de ci-dessus
    Junk(Vec<u8>),
}

impl fmt::Display for RawFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "frame {}: {:?}", self.get_state(), self.encode())
    }
}

impl RawFrame {
    /// Constructeur (`RawFrame` empty)
    #[allow(dead_code)]
    pub fn new(octets: &[u8]) -> Self {
        let mut ret = RawFrame::default();
        ret.extend(octets);
        ret
    }

    /// Constructeur `RawFrame` ACK
    #[allow(dead_code)]
    pub fn new_ack() -> Self {
        Self::Ack
    }

    /// Constructeur `RawFrame` NACK
    #[allow(dead_code)]
    pub fn new_nack() -> Self {
        Self::Nack
    }

    /// Constructeur `RawFrame` message (tag sans donnée)
    /// Les données `DataItem` peuvent être ajoutées ensuite par `try_extend_data_item`
    #[allow(dead_code)]
    pub fn new_message(tag: u8) -> Self {
        Self::Ok(tag, 0, vec![], tag)
    }

    /// Calcul du checksum (xor qui ignore le 1er caractère (STX) et les 2 derniers (XOR + ETX))
    /// Porte donc sur le tag, la longueur des données et le contenu des données
    #[allow(dead_code)]
    fn calcul_xor(tag: u8, len: u8, values: &[u8]) -> u8 {
        values.iter().fold(tag ^ len, |a, b| a ^ *b)
    }

    /// Construction de la `RawFrame` en ajoutant un octet
    #[allow(dead_code)]
    pub fn push(&mut self, octet: u8) {
        *self = match self {
            RawFrame::Empty => match octet {
                ACK => RawFrame::Ack,
                NACK => RawFrame::Nack,
                STX => RawFrame::Stx,
                _ => RawFrame::Junk(vec![octet]),
            },
            RawFrame::Ack => RawFrame::AckAndJunk(vec![octet]),
            RawFrame::AckAndJunk(junk) => {
                junk.push(octet);
                RawFrame::AckAndJunk(junk.clone())
            }
            RawFrame::Nack => RawFrame::NackAndJunk(vec![octet]),
            RawFrame::NackAndJunk(junk) => {
                junk.push(octet);
                RawFrame::NackAndJunk(junk.clone())
            }
            RawFrame::Stx => RawFrame::Tag(octet),
            RawFrame::Tag(tag) => RawFrame::TagLen(*tag, octet),
            RawFrame::TagLen(tag, len) => {
                if *len == 0 {
                    // Octet est le XOR d'un trame vide, dont tag ^ 0 ^ [] -> tag
                    if octet == *tag {
                        RawFrame::Xor(*tag, 0, vec![], octet)
                    } else {
                        let junk = vec![STX, *tag, 0, octet];
                        RawFrame::Junk(junk)
                    }
                } else {
                    RawFrame::TagLenValue(*tag, *len, vec![octet])
                }
            }
            RawFrame::TagLenValue(tag, len, values) => {
                if *len as usize == values.len() {
                    // Octet est le XOR de la trame
                    let xor = RawFrame::calcul_xor(*tag, *len, values);
                    if octet == xor {
                        RawFrame::Xor(*tag, *len, values.clone(), xor)
                    } else {
                        let mut junk = vec![STX, *tag, *len];
                        junk.extend(values.clone());
                        junk.push(octet);
                        RawFrame::Junk(junk)
                    }
                } else {
                    values.push(octet);
                    RawFrame::TagLenValue(*tag, *len, values.clone())
                }
            }
            RawFrame::Xor(tag, len, values, xor) => {
                if octet == ETX {
                    RawFrame::Ok(*tag, *len, values.clone(), *xor)
                } else {
                    let mut junk = vec![STX, *tag, *len];
                    junk.extend(values.clone());
                    junk.push(*xor);
                    junk.push(octet);
                    RawFrame::Junk(junk)
                }
            }
            RawFrame::Ok(tag, len, values, xor) => {
                RawFrame::OkAndJunk(*tag, *len, values.clone(), *xor, vec![octet])
            }

            RawFrame::OkAndJunk(tag, len, values, xor, junk) => {
                junk.push(octet);
                RawFrame::OkAndJunk(*tag, *len, values.clone(), *xor, junk.clone())
            }
            RawFrame::Junk(junk) => {
                junk.push(octet);
                RawFrame::Junk(junk.clone())
            }
        }
    }

    /// Construction de la `RawFrame` en ajoutant des octets
    #[allow(dead_code)]
    pub fn extend(&mut self, octets: &[u8]) {
        for octet in octets {
            self.push(*octet);
        }
    }

    /// Construction de la `RawFrame` en tentant d'ajouter un `DataItem`
    /// Retourne une erreur si la `RawFrame` n'est pas un message OK
    /// Retourne une erreur si l'ajout du `DataItem` produit une trame trop longue (`RAW_FRAME_MAX_LEN`)
    #[allow(dead_code)]
    #[allow(clippy::cast_possible_truncation)]
    pub fn try_extend_data_item(&mut self, data_item: &DataItem) -> Result<(), FrameError> {
        if let Self::Ok(tag, len, values, _) = self {
            let vec_u8 = data_item.encode();
            let new_len = vec_u8.len() + *len as usize;
            if new_len > RAW_FRAME_MAX_LEN {
                Err(FrameError::MaxLengthOverflow)
            } else {
                let mut new_values = values.clone();
                new_values.extend(vec_u8);
                let new_xor = RawFrame::calcul_xor(*tag, new_len as u8, &new_values);
                *self = Self::Ok(*tag, new_len as u8, new_values, new_xor);
                Ok(())
            }
        } else {
            Err(FrameError::IsNotOk)
        }
    }

    /// État de la `RawFrame`
    #[allow(dead_code)]
    pub fn get_state(&self) -> FrameState {
        match self {
            RawFrame::Empty => FrameState::Empty,

            RawFrame::Ack | RawFrame::Nack | RawFrame::Ok(_, _, _, _) => FrameState::Ok,

            RawFrame::AckAndJunk(_)
            | RawFrame::NackAndJunk(_)
            | RawFrame::OkAndJunk(_, _, _, _, _)
            | RawFrame::Junk(_) => FrameState::Junk,

            RawFrame::Stx
            | RawFrame::Tag(_)
            | RawFrame::TagLen(_, _)
            | RawFrame::TagLenValue(_, _, _)
            | RawFrame::Xor(_, _, _, _) => FrameState::Building,
        }
    }

    /// Encodage de la `RawFrame` sous forme d'un `Vec<u8>`
    #[allow(dead_code)]
    pub fn encode(&self) -> Vec<u8> {
        match self {
            RawFrame::Empty => vec![],
            RawFrame::Ack => vec![ACK],
            RawFrame::AckAndJunk(junk) => {
                let mut ret = vec![ACK];
                ret.extend(junk.clone());
                ret
            }
            RawFrame::Nack => vec![NACK],
            RawFrame::NackAndJunk(junk) => {
                let mut ret = vec![NACK];
                ret.extend(junk.clone());
                ret
            }
            RawFrame::Stx => vec![STX],
            RawFrame::Tag(tag) => vec![STX, *tag],
            RawFrame::TagLen(tag, len) => vec![STX, *tag, *len],
            RawFrame::TagLenValue(tag, len, values) => {
                let mut ret = vec![STX, *tag, *len];
                ret.extend(values.clone());
                ret
            }
            RawFrame::Xor(tag, len, values, xor) => {
                let mut ret = vec![STX, *tag, *len];
                ret.extend(values.clone());
                ret.push(*xor);
                ret
            }
            RawFrame::Ok(tag, len, values, xor) => {
                let mut ret = vec![STX, *tag, *len];
                ret.extend(values.clone());
                ret.push(*xor);
                ret.push(ETX);
                ret
            }
            RawFrame::OkAndJunk(tag, len, values, xor, junk) => {
                let mut ret = vec![STX, *tag, *len];
                ret.extend(values.clone());
                ret.push(*xor);
                ret.push(ETX);
                ret.extend(junk.clone());
                ret
            }
            RawFrame::Junk(junk) => junk.clone(),
        }
    }

    /// Tente de nettoyer une trame en retirant la partie 'junk' si possible
    #[allow(dead_code)]
    pub fn remove_junk(&mut self) {
        match self {
            RawFrame::AckAndJunk(_) => *self = RawFrame::Ack,
            RawFrame::NackAndJunk(_) => *self = RawFrame::Nack,
            RawFrame::OkAndJunk(tag, len, values, xor, _) => {
                *self = RawFrame::Ok(*tag, *len, values.clone(), *xor);
            }
            RawFrame::Junk(_) => *self = RawFrame::Empty,
            _ => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::t_data::TValue;

    #[test]
    fn test_constructor_ack() {
        let raw_frame = RawFrame::new_ack();
        assert_eq!(raw_frame, RawFrame::Ack);
        assert_eq!(raw_frame.encode(), vec![ACK]);
    }

    #[test]
    fn test_constructor_nack() {
        let raw_frame = RawFrame::new_nack();
        assert_eq!(raw_frame, RawFrame::Nack);
        assert_eq!(raw_frame.encode(), vec![NACK]);
    }

    #[test]
    fn test_constructor_message() {
        let raw_frame = RawFrame::new_message(1);
        assert_eq!(raw_frame, RawFrame::Ok(1, 0, vec![], 1));
        assert_eq!(raw_frame.encode(), vec![STX, 1, 0, 1, ETX]);
    }

    #[test]
    fn test_decode_message() {
        // Contenu du message pour le test
        let message_tag = 1;
        let data_item = DataItem::new(2, TValue::U16(123));

        // Éléments théorique du contenu du message
        let data_item_vec_u8 = data_item.encode();
        #[allow(clippy::cast_possible_truncation)]
        let data_item_vec_u8_len = data_item_vec_u8.len() as u8;
        let xor = RawFrame::calcul_xor(message_tag, data_item_vec_u8_len, &data_item_vec_u8);
        // Octets de cette trame
        let mut raw_frame_as_vec_u8 = vec![STX, message_tag, data_item_vec_u8_len];
        raw_frame_as_vec_u8.extend(&data_item_vec_u8);
        raw_frame_as_vec_u8.extend([xor, ETX]);

        // Création de la raw_frame
        let mut raw_frame = RawFrame::new_message(message_tag);
        raw_frame.try_extend_data_item(&data_item).unwrap();

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
        assert_eq!(raw_frame.encode(), raw_frame_as_vec_u8);
    }

    #[test]
    fn test_construction() {
        let tests: Vec<(&[u8], RawFrame, FrameState)> = vec![
            (&[ACK], RawFrame::Ack, FrameState::Ok),
            (&[ACK, 0], RawFrame::AckAndJunk(vec![0]), FrameState::Junk),
            (
                &[ACK, 0, 1],
                RawFrame::AckAndJunk(vec![0, 1]),
                FrameState::Junk,
            ),
            (&[NACK], RawFrame::Nack, FrameState::Ok),
            (&[NACK, 1], RawFrame::NackAndJunk(vec![1]), FrameState::Junk),
            (
                &[NACK, 1, 0],
                RawFrame::NackAndJunk(vec![1, 0]),
                FrameState::Junk,
            ),
            (&[STX], RawFrame::Stx, FrameState::Building),
            (&[STX, 1], RawFrame::Tag(1), FrameState::Building),
            (&[STX, 1, 2], RawFrame::TagLen(1, 2), FrameState::Building),
            (
                &[STX, 1, 2, 0],
                RawFrame::TagLenValue(1, 2, vec![0]),
                FrameState::Building,
            ),
            (
                &[STX, 1, 2, 0, 1],
                RawFrame::TagLenValue(1, 2, vec![0, 1]),
                FrameState::Building,
            ),
            (
                &[STX, 1, 2, 0, 1, 0],
                RawFrame::Junk(vec![STX, 1, 2, 0, 1, 0]),
                FrameState::Junk,
            ),
            (
                &[STX, 1, 2, 0, 1, 2],
                RawFrame::Xor(1, 2, vec![0, 1], 2),
                FrameState::Building,
            ),
            (
                &[STX, 1, 2, 0, 1, 2, ETX],
                RawFrame::Ok(1, 2, vec![0, 1], 2),
                FrameState::Ok,
            ),
            (
                &[STX, 1, 2, 0, 1, 2, ETX, 0],
                RawFrame::OkAndJunk(1, 2, vec![0, 1], 2, vec![0]),
                FrameState::Junk,
            ),
            (&[STX, 1, 0], RawFrame::TagLen(1, 0), FrameState::Building),
            (
                &[STX, 1, 0, 1],
                RawFrame::Xor(1, 0, vec![], 1),
                FrameState::Building,
            ),
            (
                &[STX, 1, 0, 1, ETX],
                RawFrame::Ok(1, 0, vec![], 1),
                FrameState::Ok,
            ),
            (&[1, 2, 3], RawFrame::Junk(vec![1, 2, 3]), FrameState::Junk),
        ];

        for (octets, frame, state) in tests {
            assert_eq!(
                RawFrame::new(octets),
                frame,
                "Construction incorrecte de la trame {octets:?}"
            );
            assert_eq!(
                frame.get_state(),
                state,
                "État incorrect de la trame construite {octets:?}"
            );
            assert_eq!(
                frame.encode(),
                octets.to_vec(),
                "Restitution incorrecte des octets de la trame {octets:?}"
            );
        }
    }

    #[test]
    fn test_remove_junk() {
        let tests: Vec<(&[u8], RawFrame)> = vec![
            (&[ACK], RawFrame::Ack),
            (&[ACK, 0, 1], RawFrame::Ack),
            (&[NACK], RawFrame::Nack),
            (&[NACK, 0, 1], RawFrame::Nack),
            (&[STX], RawFrame::Stx),
            (&[STX, 1], RawFrame::Tag(1)),
            (&[STX, 1, 2], RawFrame::TagLen(1, 2)),
            (&[STX, 1, 2, 0, 1, 2], RawFrame::Xor(1, 2, vec![0, 1], 2)),
            (
                &[STX, 1, 2, 0, 1, 2, ETX],
                RawFrame::Ok(1, 2, vec![0, 1], 2),
            ),
            (
                &[STX, 1, 2, 0, 1, 2, ETX, 0],
                RawFrame::Ok(1, 2, vec![0, 1], 2),
            ),
            (&[1, 2, 3], RawFrame::Empty),
        ];

        for (octets, frame) in tests {
            let mut f = RawFrame::new(octets);
            f.remove_junk();
            assert_eq!(f, frame, "Récupération NOK trame avec junk {octets:?}");
        }
    }
}
