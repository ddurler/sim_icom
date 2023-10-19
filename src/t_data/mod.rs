//! Formats et types de données génériques

mod t_format;
pub use t_format::TFormat;

mod t_value;
pub use t_value::TValue;

pub mod be_data;

/// Conversion générique d'un `Vec<u8>` en `String`
/// Retourne une `String` selon codage en UTF-8 si possible, si retourne un 'utf-8-lossy'
pub fn vec_u8_to_string(vec_u8: &[u8]) -> String {
    String::from_utf8_lossy(vec_u8).into()
}

/// Conversion générique d'une 'String' en `Vec<u8>`
/// Retourne un `Vec<u8>` selon le contenu de la `String`
pub fn string_to_vec_u8(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}
