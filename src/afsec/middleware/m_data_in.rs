//! `middleware` pour le traitement `AF_DATA_IN`
//!
//! Prend en charge une conversation pour transmettre des données à l'AFSEC+.
//! La conversation est engagée par l'ICOM sur un `AF_ALIVE` ou sur invitation à poursuivre par
//! un `AF_DATA_IN`
//!
//! Les données transmises sont les `notification_changes` reçues des autres utilisateurs.

use super::{
    id_message, utils, CommonMiddlewareTrait, Context, DataFrame, DataItem, DatabaseAfsecComm,
    IdTag, IdUser, RawFrame, TValue, TAG_DATA_PACK,
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
        println!("AFSEC Comm: AF_DATA_IN #{}...", context.nb_data_in);

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
                let data_item = DataItem::new(id_message::D_DATA_ZONE, TValue::U8(cur_zone));
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
            let data_item = DataItem::new(id_message::D_DATA_TAG, TValue::VecU8(5, vec_u8));
            if new_raw_frame.try_extend_data_item(&data_item).is_err() {
                // Ne passe pas, on arrête de gaver la trame
                break;
            }

            // Value
            let data_item = DataItem::new(id_message::D_DATA_VALUE, t_value);
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
        if id_user != afsec_service.id_user && id_tag.num_tag != TAG_DATA_PACK {
            // On ne retient que les changements d'autres utilisateurs et qui ne
            // concernent pas les changements gérés par le 'pack-in'
            context.notification_changes.push((id_tag, t_value.clone()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::{Arc, Mutex};

    use crate::database::ID_ANONYMOUS_USER;
    use crate::t_data::TFormat;
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
            t_format: TFormat::U16,
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

        // Inscription pour être notifié des changements dans la database
        afsec_service.id_user = {
            let mut db: std::sync::MutexGuard<'_, crate::database::Database> =
            // Verrouiller la database partagée
            afsec_service.thread_db.lock().unwrap();

            db.get_id_user("TEST", true)
        };

        // Par défaut, la valeur 0 dans la database
        {
            // Verrouiller la database partagée
            let db: std::sync::MutexGuard<'_, crate::database::Database> =
                afsec_service.thread_db.lock().unwrap();

            assert_eq!(db.get_u16_from_id_tag(0, id_tag), 0);
        }

        // Création d'un middleware DATA_IN pour le test
        let middleware: MDataIn = MDataIn::default();

        // Création d'une requête AFSEC+ pour invitation à parler
        let request = RawFrame::new_message(id_message::AF_ALIVE);
        let request = DataFrame::try_from(request).unwrap();

        // Envoi du message 'invitation à parler' au middleware
        let option_response =
            middleware.get_conversation(&mut context, &mut afsec_service, &request);

        // Le middleware doit avoir répondu None (rien à dire)
        assert!(option_response.is_none());

        // On modifie le contenu de l'id_tag dans la database (par un autre utilisateur)
        {
            // Verrouiller la database partagée
            let mut db: std::sync::MutexGuard<'_, crate::database::Database> =
                afsec_service.thread_db.lock().unwrap();

            db.set_u16_to_id_tag(ID_ANONYMOUS_USER, id_tag, 123);
        }

        // Active le système de notification pour notifier les middlewares
        let mut vec_changes = vec![];
        loop {
            // Verrouiller la database partagée
            let mut db = afsec_service.thread_db.lock().unwrap();

            // Voir s'il y a une notification d'un autre utilisateur
            if let Some(notification_change) = db.get_change(afsec_service.id_user, false, true) {
                if let Some(tag) = db.get_tag_from_id_tag(notification_change.id_tag) {
                    let id_user = notification_change.id_user;
                    let id_tag = notification_change.id_tag;
                    let t_value = db.get_t_value_from_tag(id_user, tag);

                    vec_changes.push((id_user, id_tag, t_value));
                }
            } else {
                break;
            }
        }
        assert!(!vec_changes.is_empty());

        // Informe le middleware des modification_changes
        for (id_user, id_tag, t_value) in vec_changes {
            middleware.notification_change(
                &mut context,
                &mut afsec_service,
                id_user,
                id_tag,
                &t_value,
            );
        }

        // Envoi du 'invitation à parler' message au middleware
        let option_response =
            middleware.get_conversation(&mut context, &mut afsec_service, &request);

        // Le middleware doit avoir répondu DATA_IN
        assert!(option_response.is_some());
        let response = option_response.unwrap();
        let response = DataFrame::try_from(response).unwrap();
        assert_eq!(response.get_tag(), id_message::IC_DATA_IN);

        // On doit y retrouve la zone = 0, le num_tag et les indices et la valeur
        let mut zone_ok = false;
        let mut tag_ok = false;
        let mut value_ok = false;
        for data_item in response.get_data_items() {
            match data_item.tag {
                id_message::D_DATA_ZONE => {
                    assert_eq!(u8::from(&data_item.t_value), 0);
                    zone_ok = true;
                }
                id_message::D_DATA_TAG => {
                    assert_eq!(
                        data_item.t_value.to_vec_u8()[0..5],
                        vec![0x01, 0x02, 0, 0, 0]
                    );
                    tag_ok = true;
                }
                id_message::D_DATA_VALUE => {
                    assert_eq!(u16::from(&data_item.t_value), 123);
                    value_ok = true;
                }
                _ => (),
            }
        }
        assert!(zone_ok);
        assert!(tag_ok);
        assert!(value_ok);
    }
}
