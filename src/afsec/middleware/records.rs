//! Gestion des tables d'enregistrements

use super::{Context, IdTag, TValue};

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
            }
            // RAZ des données
            context.record_datas = vec![];
        }
    }
}
