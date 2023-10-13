//! Encodage et décodage des trames 'brutes' telles qu'échangées via la liaison série
//! entre l'AFSEC+ et l'ICOM (support `Vec<u8>` sous-jacent)
//!
//! Ce module est prévu pour construire une trame TLV au fur et à mesure que des octets sont
//! reçus :
//! ```
//! let frame = RawFrame::default();
//! frame.push(octet);
//! ```
//!
//! On peut alors obtenir l'état en cours de la construction pour statuer ce qu'il convient de faire pour
//! poursuivre la construction :
//!
//! * `FrameState::Empty` Rien reçu : Abandoner si timeout
//! * `FrameState::Building` Des octets reçus. La trame semble correcte mais on n'a pas tout reçu : Continuer
//!    la construction en cours ou abandonner si timeout
//! * `FrameState::Ok` Réception d'une trame complète et correcte :  On peut traiter son contenu
//! * `FrameState::Junk` Réception d'octets qu'on ne sait pas interpréter : Abandonner
//!
//! Si des octets surnuméraires sont reçus après avoir identifié une trame correcte, la trame
//! devient `FrameState::Junk`. On peut alors tenter `frame.remove_junk` pour retrouver la trame correcte
//! du début.
//!
//! Ce module est également prévu pour interagir avec une vue 'logique' de la trame via [`DataFrame`]
//! pour décoder le contenu et encoder une réponse

use std::fmt;

/// Début de message
pub const STX: u8 = 0x02;

/// Fin de message
pub const ETX: u8 = 0x03;

/// Acquit de message
pub const ACK: u8 = 0x06;

/// Non-acquit de message
pub const NACK: u8 = 0x15;

/// Énumération de l''état' courant de la construction de la trame
#[derive(Debug, PartialEq)]
pub enum FrameState {
    /// Rien reçu
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
        write!(f, "frame {}: {:?}", self.get_state(), self.to_vec_u8())
    }
}

impl RawFrame {
    /// Constructeur
    pub fn new(octets: &[u8]) -> Self {
        let mut ret = RawFrame::default();
        ret.extend(octets);
        ret
    }

    /// Calcul du checksum (xor qui ignore le 1er caractère (STX) et les 2 derniers (XOR + ETX))
    pub fn calcul_xor(&self) -> u8 {
        match self {
            RawFrame::Empty
            | RawFrame::Ack
            | RawFrame::AckAndJunk(_)
            | RawFrame::Nack
            | RawFrame::NackAndJunk(_)
            | RawFrame::Junk(_)
            | RawFrame::Stx => 0,
            RawFrame::Tag(tag) => *tag,
            RawFrame::TagLen(tag, len) => *tag ^ *len,
            RawFrame::TagLenValue(tag, len, values) => {
                let mut xor: u8 = 0;
                xor ^= *tag;
                xor ^= *len;
                for v in values {
                    xor ^= *v;
                }
                xor
            }
            RawFrame::Xor(_, _, _, xor)
            | RawFrame::Ok(_, _, _, xor)
            | RawFrame::OkAndJunk(_, _, _, xor, _) => *xor,
        }
    }

    /// Construction de la `RawFrame` en ajoutant un octet
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
                    // Octet est le XOR de la trame
                    if octet == RawFrame::TagLenValue(*tag, *len, vec![]).calcul_xor() {
                        RawFrame::Xor(*tag, *len, vec![], octet)
                    } else {
                        let junk = vec![STX, *tag, *len, octet];
                        RawFrame::Junk(junk)
                    }
                } else {
                    RawFrame::TagLenValue(*tag, *len, vec![octet])
                }
            }
            RawFrame::TagLenValue(tag, len, values) => {
                if *len as usize == values.len() {
                    // Octet est le XOR de la trame
                    let f = RawFrame::TagLenValue(*tag, *len, values.clone());
                    if octet == f.calcul_xor() {
                        RawFrame::Xor(*tag, *len, values.clone(), octet)
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
    pub fn extend(&mut self, octets: &[u8]) {
        for octet in octets {
            self.push(*octet);
        }
    }

    /// Etat de la trame en cours
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

    /// Extraction des octets reçus sous forme d'un `Vec<u8>`
    pub fn to_vec_u8(&self) -> Vec<u8> {
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

    /// Tente de nettoyer une trame en retirant la partie 'junk' si c'est possible
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
                frame.to_vec_u8(),
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
