//! `middleware` pour le traitement `AF_DATA_IN`
//!
//! Prend en charge une conversation pour transmettre des données à l'AFSEC+.
//! La conversation est engagée par l'ICOM sur un `AF_ALIVE` ou sur invitation à poursuivre par
//! un `AF_DATA_IN`
//!
//! Les données transmises sont les `notification_changes` reçues des autres utilisateurs.

use super::{
    id_message, utils, CommonMiddlewareTrait, Context, DataFrame, DataItem, DatabaseAfsecComm,
    IdTag, IdUser, RawFrame, TFormat, TValue,
};

#[derive(Default)]
pub struct MDataIn {}

impl CommonMiddlewareTrait for MDataIn {
    fn reset_conversation(&self, _context: &mut Context) {}

    fn get_conversation(
        &self,
        context: &mut Context,
        _afsec_service: &mut DatabaseAfsecComm,
        request_data_frame: &DataFrame,
        _is_already_conversing: bool,
    ) -> Option<RawFrame> {
        if ![id_message::AF_ALIVE, id_message::AF_DATA_IN].contains(&request_data_frame.get_tag()) {
            // Non concerné par cette conversation
            return None;
        }

        if context.notification_changes.is_empty() {
            // Rien à transmettre à l'AFSEC+
            return None;
        }

        // Décompte des AF_DATA_IN traités
        context.nb_data_in += 1;
        println!("AFSEC Comm.: AF_DATA_IN #{}...", context.nb_data_in);

        // Préparation d'un message `IC_DATA_IN` pour transmettre des datas à l'AFSEC+
        let mut raw_frame = RawFrame::new_message(id_message::IC_DATA_IN);

        // On gave la trame de réponse avec des données à transmettre à l'AFSEC+
        let mut cur_zone = 0xFF_u8;
        loop {
            if context.notification_changes.is_empty() {
                // Plus rien à transmettre
                break;
            }

            // Tente de transmettre l'item #0 des notification_changes dans la trame
            // On préserve la construction actuelle
            let mut new_raw_frame = raw_frame.clone();

            // On laisse l'item dans la liste tant que pas sûr de pouvoir l'intégrer dans le message
            let (id_tag, t_value) = context.notification_changes[0].clone();

            // Dans le message, on doit mettre 3 choses : `D_DATA_ZONE`, `D_DATA_TAG` et `D_DATA_VALUE`

            // La zone peut être omise si elle est idem à la donnée précédente du message
            if id_tag.zone != cur_zone {
                cur_zone = id_tag.zone;
                let data_item =
                    DataItem::new(id_message::D_DATA_ZONE, TFormat::U8, TValue::U8(cur_zone));
                if new_raw_frame.try_extend_data_item(&data_item).is_err() {
                    // Ne passe pas, on arrête de gaver la trame
                    break;
                }
            }

            // Tag
            let vec_u8 = utils::tag_num_indices_to_vec_u8(
                id_tag.num_tag,
                id_tag.indice_0,
                id_tag.indice_1,
                id_tag.indice_2,
            );
            let data_item = DataItem::new(
                id_message::D_DATA_TAG,
                TFormat::VecU8(5),
                TValue::VecU8(5, vec_u8),
            );
            if new_raw_frame.try_extend_data_item(&data_item).is_err() {
                // Ne passe pas, on arrête de gaver la trame
                break;
            }

            // Value
            let data_item = DataItem::new(id_message::D_DATA_TAG, TFormat::from(&t_value), t_value);
            if new_raw_frame.try_extend_data_item(&data_item).is_err() {
                // Ne passe pas, on arrête de gaver la trame
                break;
            }

            // Tout est passé
            raw_frame = new_raw_frame.clone();
            context.notification_changes.remove(0);
        }

        // Réponse
        Some(raw_frame)
    }

    fn notification_change(
        &self,
        context: &mut Context,
        afsec_service: &mut DatabaseAfsecComm,
        id_user: IdUser,
        id_tag: IdTag,
        t_value: &TValue,
    ) {
        if id_user != afsec_service.id_user {
            // On ne retient que les changements d'autres utilisateurs
            context.notification_changes.push((id_tag, t_value.clone()));
        }
    }
}
