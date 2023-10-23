//! Helpers pour les `middlewares`

use super::{Context, DatabaseAfsecComm, IdTag, RecordData, TValue};

/// Helper pour découper un `u32` au format 10000 * version + 100 * revision + edition
pub fn u32_to_version_revision_edition(version_revision_edition: u32) -> (u16, u16, u16) {
    let edition = version_revision_edition % 100;
    let version_revision_edition = version_revision_edition / 100;
    let revision = version_revision_edition % 100;
    let version_revision_edition = version_revision_edition / 100;
    let version = version_revision_edition % 100;

    (version as u16, revision as u16, edition as u16)
}

/// Helper pour convertir une `zone` + `tag_str5` en `IdTag`
pub fn zone_vec_u8_tag_to_id_tag(zone: u8, vec_u8_tag: &[u8]) -> IdTag {
    // Converti le vec_u8_tag en un Vec<u8> d'au moins 5 éléments
    let mut vec_u8 = vec_u8_tag.to_vec();
    while vec_u8.len() < 5 {
        vec_u8.push(0);
    }
    // Création de l'IdTag correspondant
    #[allow(clippy::cast_lossless)]
    let tag = vec_u8[0] as u16 * 256 + vec_u8[1] as u16;
    IdTag::new(zone, tag, [vec_u8[2], vec_u8[3], vec_u8[4]])
}

/// Helper pour convertir un `tag_num` + `indices` en un `Vec<u8>` de 5 `u8`
pub fn tag_num_indices_to_vec_u8(
    num_tag: u16,
    indice_0: u8,
    indice_1: u8,
    indice_2: u8,
) -> Vec<u8> {
    let mut vec_u8 = vec![];
    vec_u8.extend(num_tag.to_be_bytes());
    vec_u8.extend([indice_0, indice_1, indice_2]);
    vec_u8
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
        TValue::VecU8(len, value) => {
            let mut vec_u8 = value.clone();
            while vec_u8.len() < len {
                vec_u8.push(0);
            }
            if vec_u8.len() > len {
                vec_u8 = vec_u8[0..len].to_vec();
            }
            db.set_vec_u8_to_id_tag(afsec_service.id_user, id_tag, &vec_u8);
        }
    }
}

/// Helper pour l'ajout d'une donnée d'un enregistrement d'une table
pub fn add_record(context: &mut Context, record: RecordData) {
    if RecordData::is_id_tag_end_of_record(record.id_tag) {
        println!("AFSEC Comm: Got END_OF_RECORD");
        RecordData::collect_record_datas(context);
    } else {
        context.record_datas.push(record);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u32_to_version_revision_edition() {
        let version = 1_u16;
        let revision = 2_u16;
        let edition = 3_u16;

        #[allow(clippy::cast_lossless)]
        let version_revision_edition =
            version as u32 * 10_000 + revision as u32 * 100 + edition as u32;

        assert_eq!(
            u32_to_version_revision_edition(version_revision_edition),
            (version, revision, edition)
        );
    }

    #[test]
    fn test_zone_vec_u8_tag_to_id_tag() {
        let zone = 1_u8;
        let vec_u8_tag = vec![0x12, 0x23, 0x34, 0x45, 0x56];

        let id_tag = zone_vec_u8_tag_to_id_tag(zone, &vec_u8_tag);

        assert_eq!(id_tag, IdTag::new(zone, 0x1223, [0x34, 0x45, 0x56]));
    }

    #[test]
    fn test_tag_num_indices_to_vec_u8() {
        assert_eq!(
            tag_num_indices_to_vec_u8(0x0123, 0x45, 0x67, 0x89),
            vec![0x01, 0x23, 0x45, 0x67, 0x89]
        );
    }
}
