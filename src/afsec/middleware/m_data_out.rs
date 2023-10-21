//! `middleware` pour le traitement `AF_DATA_OUT`
//!
//! Prend en charge les conversations `AF_DATA_OUT` du résident qui transmet des données.
//! Il peut s'agir de données pour renseigner la `Database` (`ZONE` + `IdTag` + `TValue`)
//! ou de donnée pour un enregistrement dans un journal (`TABLE_INDEX` en sus)

use super::{
    id_message, records::RecordData, utils, CommonMiddlewareTrait, Context, DataFrame,
    DatabaseAfsecComm, IdTag, IdUser, RawFrame, TValue,
};

#[derive(Default)]
pub struct MDataOut {}

impl CommonMiddlewareTrait for MDataOut {
    fn reset_conversation(&self, context: &mut Context) {
        // Table index et le numéro de zone sont contextuels et peuvent être valides pour plusieurs trames
        context.option_vec_u8_tag = None;
        context.option_t_value = None;
        // Sauvegarde des données des enregistrements (si existent)
        RecordData::collect_record_datas(context);
    }

    fn get_conversation(
        &self,
        context: &mut Context,
        afsec_service: &mut DatabaseAfsecComm,
        request_data_frame: &DataFrame,
    ) -> Option<RawFrame> {
        if request_data_frame.get_tag() != id_message::AF_DATA_OUT {
            return None;
        }
        // Décompte des AF_DATA_OUT traités
        context.nb_data_out += 1;
        println!("AFSEC Comm: AF_DATA_OUT #{}...", context.nb_data_out);

        // Init avant traitement
        context.option_vec_u8_tag = None;
        context.option_t_value = None;

        // Exploitation des informations reçues et mise à jour de la database
        for data_item in request_data_frame.get_data_items() {
            match data_item.tag {
                id_message::D_DATA_ZONE => context.option_zone = Some(u8::from(&data_item.t_value)),
                id_message::D_DATA_TABLE_INDEX => {
                    context.option_table_index = Some(u64::from(&data_item.t_value));
                }
                id_message::D_DATA_TAG => {
                    let tag_as_string = data_item.t_value.to_t_value_vec_u8(5);
                    if let TValue::VecU8(_, vec_u8) = tag_as_string {
                        context.option_vec_u8_tag = Some(vec_u8);
                    }
                }
                id_message::D_DATA_VALUE => context.option_t_value = Some(data_item.t_value),
                _ => (),
            }

            // Si on a reçu au moins zone + vec_u8_tag + t_value
            if let Some(zone) = context.option_zone {
                if let Some(vec_u8_tag) = &context.option_vec_u8_tag {
                    let id_tag = utils::zone_vec_u8_tag_to_id_tag(zone, vec_u8_tag);
                    if let Some(t_value) = &context.option_t_value {
                        if let Some(table_index) = context.option_table_index {
                            // Avec un `table index`, on est dans la mise à jour d'un enregistrement
                            let record = RecordData::new(table_index, id_tag, t_value);
                            utils::add_record(context, record);
                        } else {
                            // Mise à jour de la database
                            utils::update_database(afsec_service, id_tag, t_value.clone());
                        }
                        // RAZ après traitement
                        context.option_vec_u8_tag = None;
                        context.option_t_value = None;
                    }
                }
            }
        }

        // Réponse
        Some(RawFrame::new_ack())
    }

    fn notification_change(
        &self,
        _context: &mut Context,
        _afsec_service: &mut DatabaseAfsecComm,
        _id_user: IdUser,
        _id_tag: IdTag,
        _t_value: &TValue,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::{Arc, Mutex};

    use crate::afsec::tlv_frame::DataItem;
    use crate::database::ID_ANONYMOUS_USER;
    use crate::{database::Tag, Database};

    #[test]
    fn test_conversation() {
        // Création d'une database
        let mut db = Database::default();

        // Création d'un tag
        let id_tag = IdTag::new(0, 0x0102, [0, 0, 0]);
        let tag = Tag {
            word_address: 0x0000,
            id_tag,
            ..Default::default()
        };
        db.add_tag(&tag);

        // Créer la database partagée mutable
        let shared_db = Arc::new(Mutex::new(db));
        // Cloner la référence à la database partagée pour la communication avec l'AFSEC+
        let db_afsec = Arc::clone(&shared_db);

        // Création contexte pour les middlewares
        let mut context = Context::default();
        let mut afsec_service = DatabaseAfsecComm::new(db_afsec, "fake".to_string());

        // Par défaut, la valeur 0 dans la database
        {
            // Verrouiller la database partagée
            let db: std::sync::MutexGuard<'_, crate::database::Database> =
                afsec_service.thread_db.lock().unwrap();

            assert_eq!(db.get_u16_from_id_tag(ID_ANONYMOUS_USER, id_tag), 0);
        }

        // Création d'une requête AFSEC+ AF_DATA_OUT pour changer la valeur de l'id_tag
        let mut request = RawFrame::new_message(id_message::AF_DATA_OUT);
        let data_item_zone = DataItem::new(id_message::D_DATA_ZONE, TValue::U8(0));
        request.try_extend_data_item(&data_item_zone).unwrap();
        let data_item_tag = DataItem::new(
            id_message::D_DATA_TAG,
            TValue::VecU8(5, vec![0x01, 0x02, 0, 0, 0]),
        );
        request.try_extend_data_item(&data_item_tag).unwrap();
        let data_item_value = DataItem::new(id_message::D_DATA_VALUE, TValue::U16(123));
        request.try_extend_data_item(&data_item_value).unwrap();
        let request = DataFrame::try_from(request).unwrap();

        // Envoi du message au middleware
        let middleware = MDataOut::default();
        let response = middleware
            .get_conversation(&mut context, &mut afsec_service, &request)
            .unwrap();

        // Le middleware doit avoir réponde ACK
        assert_eq!(response, RawFrame::new_ack());

        // Et on doit maintenant lire la valeur 123 dans la database
        {
            // Verrouiller la database partagée
            let db: std::sync::MutexGuard<'_, crate::database::Database> =
                afsec_service.thread_db.lock().unwrap();

            assert_eq!(db.get_u16_from_id_tag(ID_ANONYMOUS_USER, id_tag), 123);
        }
    }
}
