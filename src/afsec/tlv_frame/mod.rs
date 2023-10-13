//! Encodage et décodage des trames TLV (Tag + Length + Value) utilisées pour
//! communiquer entre l'AFSEC+ et l'ICOM.
//!
//! Ce module propose de gérer la construction et l'analyse de ces trames sous 2 aspects :
//!
//! * `RawFrame` : Trame 'brute' telle qu'échangée via la liaison série sous forme d'un `Vec<u8>`
//! * `DataFrame` : Trame contenant un tag et une liste de données (elles-mêmes au format TLV)
//!

mod data_frame;
pub use data_frame::DataFrame;

mod raw_frame;
pub use raw_frame::RawFrame;
