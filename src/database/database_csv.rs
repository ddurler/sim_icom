//! Décodage du contenu d'un fichier database*.csv

use super::IdTag;
use super::TFormat;
use crate::database::Tag;

/// Parse une ligne du fichier database*.csv et retourne
/// `Ok(Some(u16, NumTag, Tag, String))` si la ligne contient la définition d'un [`Tag`]
/// `Ok(None)` si la ligne ne contient pas la définition d'un [`Tag`] (commentaire)
/// `Err(String)` pour signaler une erreur de contenu dans cette ligne
pub fn from_line_csv(line: &str) -> Result<Option<Tag>, String> {
    if line.is_empty() || line.starts_with("//") || line.starts_with("@@") {
        return Ok(None);
    }
    let fields: Vec<&str> = line.split(';').collect();

    let mut tag: Tag = Tag::default();

    // println!("{} fields in '{}'", fields.len(), line);
    // for (n, field) in fields.clone().into_iter().enumerate() {
    //     println!("{n}: '{field}'");
    // }

    // Champ #0 : 00:0000:00:00:00 -> internal + num_tag + indice 0, 1 et 3
    let (is_internal, num_tag_u16, indice_0, indice_1, indice_2) = parse_field0(fields[0].trim())?;
    tag.is_internal = is_internal;

    // Champ #1 : word_address MODBUS (hexa)
    let word_address = parse_str_hexa_to_u16(fields[1].trim())?;
    tag.word_address = word_address;

    // Champ #2 : Format de la donnée hexa
    let format_u8 = parse_str_hexa_to_u8(fields[2].trim())?;
    tag.t_format = TFormat::from(format_u8);
    if tag.t_format == TFormat::Unknown {
        return Err(format!("Format inconnu de donnée : {format_u8:02X}"));
    }

    // Champ #3 : Unité (si définie)
    tag.unity = fields[3].trim().to_string();

    // Champ #4 : Libellé (si défini)
    tag.label = fields[4].trim().to_string();

    // Champs #5 (CanOpen index), #6 (CanOpen), #7 (MQTT topic), #8 (QoS), #9 (Not used)

    // Champ #10 : R/W (0/1)
    let read_write_u8 = match fields[10].trim().parse::<u8>() {
        Ok(rw) => rw,
        Err(e) => {
            return Err(format!("R/W incorrect : {e}"));
        }
    };
    tag.is_write = read_write_u8 == 1;

    // Champ #11 : Zone (décimal)
    let zone = match fields[11].trim().parse::<u8>() {
        Ok(zone) => zone,
        Err(e) => {
            return Err(format!("No de zone incorrect : {e}"));
        }
    };

    // Champ #12 : Valeur par défaut
    tag.default_value = fields[12].trim().to_string();

    // Construction de l'[`IdTag`] trouvé
    tag.id_tag = IdTag::new(zone, num_tag_u16, [indice_0, indice_1, indice_2]);

    // On retourne le [`Tag`] construit
    Ok(Some(tag))
}

/// Parse un champ hexadécimal de 1 caractère
fn parse_char_hexa(car: char) -> Result<u8, String> {
    let value = match car {
        '0' => 0_u8,
        '1' => 1,
        '2' => 2,
        '3' => 3,
        '4' => 4,
        '5' => 5,
        '6' => 6,
        '7' => 7,
        '8' => 8,
        '9' => 9,
        'a' | 'A' => 10,
        'b' | 'B' => 11,
        'c' | 'C' => 12,
        'd' | 'D' => 13,
        'e' | 'E' => 14,
        'f' | 'F' => 15,
        _ => {
            return Err(format!("Caractère hexa incorrect : {car}"));
        }
    };
    Ok(value)
}

/// Parse un champ hexa vers un u8
fn parse_str_hexa_to_u8(field: &str) -> Result<u8, String> {
    let mut value: u8 = 0;
    for car in field.chars() {
        value = 16 * value + parse_char_hexa(car)?;
    }
    Ok(value)
}

/// Parse un champ hexa vers un u16
fn parse_str_hexa_to_u16(field: &str) -> Result<u16, String> {
    let mut value: u16 = 0;
    for car in field.chars() {
        value = 16 * value + u16::from(parse_char_hexa(car)?);
    }
    Ok(value)
}

/// Parse le champ #0 : 00:0000:00:00:00 -> internal + `num_tag` + indices 0, 1 et 2
fn parse_field0(field: &str) -> Result<(bool, u16, u8, u8, u8), String> {
    if field.len() != 16 {
        return Err("Longueur incorrecte du champ#0 (xx:xxxx:xx:xx:xx attendu)".to_string());
    }
    let split: Vec<&str> = field.split(':').collect();
    if split.len() != 5 {
        return Err("Format incorrect du champ#0 (xx:xxxx:xx:xx:xx attendu)".to_string());
    }
    let is_internal = split[0] != "00";
    let num_tag = parse_str_hexa_to_u16(split[1])?;
    let indice_0 = parse_str_hexa_to_u8(split[2])?;
    let indice_1 = parse_str_hexa_to_u8(split[3])?;
    let indice_2 = parse_str_hexa_to_u8(split[4])?;
    Ok((is_internal, num_tag, indice_0, indice_1, indice_2))
}
