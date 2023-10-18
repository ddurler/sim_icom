//! Helpers pour les `middlewares`

use super::{DatabaseAfsecComm, IdTag, TValue};

/// Helper pour découper un u32 en 10000 * version + 100 * revision + edition
pub fn get_version_revision_edition_from_u32(version_revision_edition: u32) -> (u16, u16, u16) {
    let edition = version_revision_edition % 100;
    let version_revision_edition = version_revision_edition / 100;
    let revision = version_revision_edition % 100;
    let version_revision_edition = version_revision_edition / 100;
    let version = version_revision_edition % 100;

    (version as u16, revision as u16, edition as u16)
}

/// Helper pour convertir une `zone` + `tag_str5` en `IdTag`
#[allow(clippy::cast_lossless)]
pub fn get_id_tag_from_zone_tag_str5(zone: u8, tag_str5: &str) -> IdTag {
    // Converti le tag_str5 en un Vec<u8> d'au moins 5 éléments
    let mut vec_u8 = tag_str5.as_bytes().to_vec();
    while vec_u8.len() < 5 {
        vec_u8.push(0);
    }
    // Création de l'IdTag correspondant
    let tag = vec_u8[0] as u16 * 256 + vec_u8[1] as u16;
    IdTag::new(zone, tag, [vec_u8[2], vec_u8[3], vec_u8[4]])
}

/// Helper pour mettre à jour la `Database`
pub fn update_database(afsec_service: &mut DatabaseAfsecComm, id_tag: IdTag, t_value: TValue) {
    println!("AFSEC Comm: Database update {id_tag} = {t_value}");

    // Verrouiller la database partagée
    let mut db: std::sync::MutexGuard<'_, crate::database::Database> =
        afsec_service.thread_db.lock().unwrap();

    /* Mise à jour database */
    match t_value {
        TValue::Bool(value) => db.set_bool_to_id_tag(afsec_service.id_user, id_tag, value),
        TValue::U8(value) => db.set_u8_to_id_tag(afsec_service.id_user, id_tag, value),
        TValue::I8(value) => db.set_i8_to_id_tag(afsec_service.id_user, id_tag, value),
        TValue::U16(value) => db.set_u16_to_id_tag(afsec_service.id_user, id_tag, value),
        TValue::I16(value) => db.set_i16_to_id_tag(afsec_service.id_user, id_tag, value),
        TValue::U32(value) => db.set_u32_to_id_tag(afsec_service.id_user, id_tag, value),
        TValue::I32(value) => db.set_i32_to_id_tag(afsec_service.id_user, id_tag, value),
        TValue::U64(value) => db.set_u64_to_id_tag(afsec_service.id_user, id_tag, value),
        TValue::I64(value) => db.set_i64_to_id_tag(afsec_service.id_user, id_tag, value),
        TValue::F32(value) => db.set_f32_to_id_tag(afsec_service.id_user, id_tag, value),
        TValue::F64(value) => db.set_f64_to_id_tag(afsec_service.id_user, id_tag, value),
        TValue::String(_, value) => db.set_string_to_id_tag(afsec_service.id_user, id_tag, &value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::cast_lossless)]
    fn test_get_version_revision_edition_from_u32() {
        let version = 1_u16;
        let revision = 2_u16;
        let edition = 3_u16;

        let version_revision_edition =
            version as u32 * 10_000 + revision as u32 * 100 + edition as u32;

        assert_eq!(
            get_version_revision_edition_from_u32(version_revision_edition),
            (version, revision, edition)
        );
    }

    #[test]
    fn test_get_id_tag_from_zone_tag_str5() {
        let zone = 1_u8;
        let tag_str5 = String::from_utf8(vec![0x12, 0x23, 0x34, 0x45, 0x56]).unwrap();

        let id_tag = get_id_tag_from_zone_tag_str5(zone, &tag_str5);

        assert_eq!(id_tag, IdTag::new(zone, 0x1223, [0x34, 0x45, 0x56]));
    }
}
