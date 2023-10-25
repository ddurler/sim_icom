//! Gestion des tables d'enregistrements

use super::{Context, IdTag, TValue};

/// Tag pour un `END_OF_RECORD` d'un `DATA_OUT` lors d'un enregistrement d'un journal
/// Voir SR DEV 004
const TAG_NUM_END_OF_RECORD: u16 = 0x7210;

/// Structure pour une donnée d'un enregistrement
#[derive(Debug)]
pub struct RecordData {
    /// Index de l'enregistrement dans la table
    pub table_index: u64,

    /// `IdTag` de la donnée de cet enregistrement
    pub id_tag: IdTag,

    /// `TValue` de la donnée de cet enregistrement
    pub t_value: TValue,
}

impl Default for RecordData {
    fn default() -> Self {
        Self {
            table_index: 0,
            id_tag: IdTag::default(),
            t_value: TValue::Bool(false),
        }
    }
}

impl RecordData {
    /// Constructeur d'une donnée d'un enregistrement
    pub fn new(table_index: u64, id_tag: IdTag, t_value: &TValue) -> Self {
        Self {
            table_index,
            id_tag,
            t_value: t_value.clone(),
        }
    }

    /// Indique s'il s'agit d'un tag `END_OF_RECORD` pour les enregistrements
    pub fn is_id_tag_end_of_record(id_tag: IdTag) -> bool {
        id_tag.num_tag == TAG_NUM_END_OF_RECORD
    }

    /// Annonce la fin de la collecte des données d'un enregistrement
    /// Toutes les données sont dans le contexte
    pub fn collect_record_datas(context: &mut Context) {
        if !context.record_datas.is_empty() {
            println!("AFSEC Comm: Constitution d'un RECORD avec:");
            for record in &context.record_datas {
                println!(
                    "    table_index={}, id_tag={}, t_value={}",
                    record.table_index, record.id_tag, record.t_value
                );
                // Informe le contexte
                context
                    .records
                    .set_index(record.id_tag.zone, record.table_index);
            }
            // RAZ des données
            context.record_datas = vec![];
        }
    }
}
