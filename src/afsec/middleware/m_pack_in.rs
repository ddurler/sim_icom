//! `middleware` pour le traitement `AF_PACK_IN`
//!
//! Prend en charge une conversation pour transmettre des données à l'AFSEC+.
//! La conversation est engagée par l'ICOM sur un `AF_ALIVE` ou sur invitation à poursuivre par
//! un `AF_PACK_IN`
//!
//! Les données transmises sont les `notification_changes` reçues des autres utilisateurs.
//!
//! Ce `middleware` est prioritaire sur le `middleware` qui prend en charge les `DATA_IN` car il ne
//! s'occupe que des données `DATA_PACK` qui représentent une table de 256 mots découpée en 8 blocs
//! de données de 64 octets (32 mots)
//!
//! Ce `middleware` utilise plusieurs infos dans le contexte:
//!
//! * `is_transaction`: `bool`: Ce flag est à true lorsqu'une transaction de données `pack_in` est en cours.
//!     Dans ce cas, les données à transmettre sont dans `set_blocs` et dans `private_datas`
//! * `set_blocs`: `HashSet<u8>`: Hors transaction, contient la liste des u8 (de 0 à 7) des blocs qui seront
//!     à transmettre lors de la prochaine transaction. Pendant une transaction, cette liste est exploitée
//!     conjointement avec `private_datas`
//! * `private_datas`: `Vec<(u8, Vec<u8>)>`: Cette liste est initialisée lorsqu'une transaction débute avec une
//!     copie privée des blocs à transmettre pendant la transaction. Le premier `u8` est le numéro de bloc de 0 à 7
//!     identique au contenu de `set_blocs`. Au fur et à mesure que des blocs sont transmis, les items de
//!     `private_datas` sont supprimés mais `set_blocs` reste intact.
//!     Le nombre total de `blocs` à transmettre pendant la transaction est `set_blocs.len()`.
//!     Les `blocs` restant à transmettre sont dans `private_datas.len()`
//! * `set_pending_blocs: HashSet<u8>`: Idem à `set_blocs` pour enregistrer les blocs à transmettre lorsque
//!     la transaction en cours sera terminée (`notification_changes` reçues pendant une transaction `pack_in`)

use std::vec;

use super::{
    id_message, CommonMiddlewareTrait, Context, DataFrame, DataItem, DatabaseAfsecComm, IdTag,
    IdUser, RawFrame, TValue, DEBUG_LEVEL_ALL, DEBUG_LEVEL_SOME, TAG_DATA_PACK,
};

#[derive(Default)]
pub struct MPackIn {}

impl CommonMiddlewareTrait for MPackIn {
    fn reset_conversation(&self, _context: &mut Context) {}

    fn get_conversation(
        &self,
        context: &mut Context,
        afsec_service: &mut DatabaseAfsecComm,
        request_data_frame: &DataFrame,
    ) -> Option<RawFrame> {
        if ![id_message::AF_ALIVE, id_message::AF_PACK_IN].contains(&request_data_frame.get_tag()) {
            // Non concerné par cette conversation
            return None;
        }

        // Vérifie si transaction en cours ou s'il faut démarrer une nouvelle transaction
        if !context.pack_in.is_transaction {
            if context.pack_in.set_blocs.is_empty() {
                // Pas de transaction en cours et rien à transmettre
                return None;
            }
            // Début d'une transaction `pack_in`
            MPackIn::start_transaction(context, afsec_service);
        }

        // Décompte des AF_PACK_IN traités
        context.nb_pack_in += 1;
        if context.debug_level >= DEBUG_LEVEL_SOME {
            println!("AFSEC Comm: AF_PACK_IN #{}...", context.nb_pack_in);
        }

        // Préparation d'un message `IC_PACK_IN` pour transmettre des datas à l'AFSEC+
        let mut raw_frame = RawFrame::new_message(id_message::IC_PACK_IN);

        // Nombre de `blocs` à transmettre
        let total_nb_blocs = context.pack_in.set_blocs.len();

        // Liste des blocs de cette transmission (pour la trace)
        let mut vec_blocs = vec![];

        // On gave la trame avec des données à transmettre à l'AFSEC+
        loop {
            if context.pack_in.private_datas.is_empty() {
                // Plus rien à transmettre
                break;
            }

            // Tente de transmettre l'item #0 des private_datas dans la trame
            // Rappel les items sont (u8, Vec<u8>) donc
            //   .0 est le numéro de bloc entre 0 et 7
            //   .1 est le contenu du bloc (64 octets)

            // On préserve la construction actuelle
            let mut new_raw_frame = raw_frame.clone();

            // Indice du bloc à transmettre [0-7]
            let bloc = context.pack_in.private_datas[0].0;

            // Adresse `mot` de ce bloc[0-255] (On a 8 blocs de 32 mots)
            let cur_word_address = bloc * 32;

            // Numéro du bloc [1-total_nb_blocs]
            // On calcule 1 pour le 1er bloc transmis et `total_nb_blocs` pour le dernier bloc
            let num_bloc = total_nb_blocs - context.pack_in.private_datas.len() + 1;

            // Payload de ce bloc
            let mut vec_u8 = vec![];

            // Octet #0: numéro de bloc+nombre total de blocs (0x12 pour dire bloc #1 pour un total de 2)
            #[allow(clippy::cast_possible_truncation)]
            vec_u8.push(16 * num_bloc as u8 + total_nb_blocs as u8);

            // Octet #1: adresse mot du bloc
            vec_u8.push(cur_word_address);

            // Le reste est le contenu du bloc
            vec_u8.extend(&context.pack_in.private_datas[0].1);

            // Taille du payload (normalement 2 + 64 = 66)
            let width = vec_u8.len();

            // Tente d'ajouter ce payload dans le message
            let data_item = DataItem::new(id_message::D_PACK_PAYLOAD, TValue::VecU8(width, vec_u8));
            if new_raw_frame.try_extend_data_item(&data_item).is_err() {
                // Ne passe pas, on arrête de gaver la trame
                break;
            }

            // Ca passe...
            raw_frame = new_raw_frame.clone();
            context.pack_in.private_datas.remove(0);
            vec_blocs.push(num_bloc);
        }

        // Trace
        if context.debug_level >= DEBUG_LEVEL_ALL {
            println!("AFSEC Comm: AF_PACK_IN replies with packets #{vec_blocs:?}/{total_nb_blocs}");
        }

        if context.pack_in.private_datas.is_empty() {
            // Tous les blocs de la transaction sont dans un message
            MPackIn::end_transaction(context);
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
        _t_value: &TValue,
    ) {
        if id_user != afsec_service.id_user && id_tag.zone == 5 && id_tag.num_tag == TAG_DATA_PACK {
            // On ne retient que les changements d'autres utilisateurs d'un tag `DATA_PACK`
            // dans la zone de commande (zone = 5)
            // On identifie le 'bloc' de 64 octets concerné par le dernier indice du tag
            if context.pack_in.is_transaction {
                // Une transaction est en cours, on mémorise le changement pour la transaction à suivre
                context.pack_in.set_pending_blocs.insert(id_tag.indice_2);
            } else {
                context.pack_in.set_blocs.insert(id_tag.indice_2);
            }
        }
    }
}

impl MPackIn {
    /// Nouvelle transaction `pack-in`
    fn start_transaction(context: &mut Context, afsec_service: &mut DatabaseAfsecComm) {
        if context.pack_in.is_transaction {
            // Transaction déjà en cours...
            return;
        }

        // Démarre la transaction
        context.pack_in.is_transaction = true;
        if context.debug_level >= DEBUG_LEVEL_SOME {
            println!(
                "AFSEC Comm: AF_PACK_IN starts new transaction with #{} packets",
                context.pack_in.set_blocs.len()
            );
        }

        // Mise à jour de la copie privée des `blocs` à transmettre à l'AFSEC+
        context.pack_in.private_datas = vec![];

        for bloc in &context.pack_in.set_blocs {
            // On va chercher les 64 octets correspondant dans la database
            let id_tag = IdTag::new(5, TAG_DATA_PACK, [0, 0, *bloc]);
            let vec_u8 = {
                // Verrouiller la database partagée
                let db: std::sync::MutexGuard<'_, crate::database::Database> =
                    afsec_service.thread_db.lock().unwrap();

                db.get_vec_u8_from_id_tag(afsec_service.id_user, id_tag, 64)
            };
            context.pack_in.private_datas.push((*bloc, vec_u8));
        }
    }

    /// Termine la transaction `pack-in` en cours
    fn end_transaction(context: &mut Context) {
        if !context.pack_in.is_transaction {
            // Pas de transaction en cours...
            return;
        }

        // On récupère les éléments éventuellement pending pour une nouvelle transaction à suivre
        context.pack_in.set_blocs = context.pack_in.set_pending_blocs.clone();
        context.pack_in.set_pending_blocs.clear();

        // Hors transaction maintenant
        context.pack_in.is_transaction = false;
        if context.debug_level >= DEBUG_LEVEL_ALL {
            println!("AFSEC Comm: AF_PACK_IN ends transaction");
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
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_lossless)]
    fn test_conversation() {
        // Création d'une database
        let mut db = Database::default();

        // Adresse (arbitraire) de la zone 'pack-out' dans la database
        // (1ere zone de 32 mots)
        let word_address_pack_out = 0x0010;

        // id_tag correspondant à la 1ere zone 'pack-out (en zone 5) dans la database
        let id_tag = IdTag::new(5, TAG_DATA_PACK, [0, 0, 0]);
        let tag = Tag {
            word_address: word_address_pack_out,
            id_tag,
            t_format: TFormat::VecU8(64),
            ..Default::default()
        };
        db.add_tag(&tag);

        // Choix d'une adresse mot (0-31 car une seule zone de 32 mots pour ce test)
        // et des valeurs (u8) dans la zone 'pack-out
        let test_address = 10_u16;
        let test_values = vec![1_u8, 2_u8, 3_u8, 4_u8];

        // Créer la database partagée mutable
        let shared_db = Arc::new(Mutex::new(db));
        // Cloner la référence à la database partagée pour la communication avec l'AFSEC+
        let db_afsec = Arc::clone(&shared_db);

        // Création contexte pour les middlewares
        let mut context = Context::new(DEBUG_LEVEL_ALL);
        let mut afsec_service =
            DatabaseAfsecComm::new(db_afsec, "fake".to_string(), DEBUG_LEVEL_ALL);

        // Inscription pour être notifié des changements dans la database
        afsec_service.id_user = {
            let mut db: std::sync::MutexGuard<'_, crate::database::Database> =
            // Verrouiller la database partagée
            afsec_service.thread_db.lock().unwrap();

            db.get_id_user("TEST", true)
        };

        // Création d'un middleware DATA_IN pour le test
        let middleware = MPackIn::default();

        // Création d'une requête AFSEC+ pour invitation à parler
        let request = RawFrame::new_message(id_message::AF_ALIVE);
        let request = DataFrame::try_from(request).unwrap();

        // Envoi du message 'invitation à parler' au middleware
        let option_response =
            middleware.get_conversation(&mut context, &mut afsec_service, &request);

        // Le middleware doit avoir répondu None (rien à dire)
        assert!(option_response.is_none());

        // On modifie le contenu de la zone 'pack-in dans la database (par un autre utilisateur)
        let word_address = word_address_pack_out + test_address;
        {
            // Verrouiller la database partagée
            let mut db: std::sync::MutexGuard<'_, crate::database::Database> =
                afsec_service.thread_db.lock().unwrap();

            db.set_vec_u8_to_word_address(ID_ANONYMOUS_USER, word_address, &test_values);
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

        // Le middleware doit avoir répondu PACK_IN
        assert!(option_response.is_some());
        let response = option_response.unwrap();
        let response = DataFrame::try_from(response).unwrap();
        assert_eq!(response.get_tag(), id_message::IC_PACK_IN);

        // On doit y retrouve un payload avec l'adresse et les valeurs modifiées
        // Le middleware renvoie des paquets de 64 bytes (la zone de 256 mots est
        // découpée en 8 zones de 64 octets)
        let mut payload_ok = false;
        for data_item in response.get_data_items() {
            match data_item.tag {
                id_message::D_PACK_PAYLOAD => {
                    assert!(!payload_ok);
                    let payload_nb_octets = data_item.t_format.nb_bytes();
                    let vec_u8 = data_item.t_value.to_vec_u8();

                    // On doit avoir au moins 2 octets + les données dans ce paquet
                    assert!(payload_nb_octets >= 2 + test_values.len());

                    // Nombre d'octets de données
                    let data_nb_bytes = payload_nb_octets - 2;

                    // Octet #0 : numéro de paquet/total paquets
                    // On doit avoir Message 1/1
                    assert_eq!(vec_u8[0], 0x11);
                    // Octet #1 : adresse dans la pack_in
                    // Les octets du payload à partir de cette adresse doivent recouvrir la partie notifiée
                    let payload_start_word_address = vec_u8[1] as u16;
                    let payload_end_word_address =
                        payload_start_word_address + data_nb_bytes as u16 / 2;
                    let data_end_word_address = test_address + test_values.len() as u16 / 2;
                    assert!(payload_start_word_address <= test_address);
                    assert!(payload_end_word_address >= data_end_word_address);

                    // Les autres octets doivent contenir les valeurs modifiées
                    let offset_modified = (test_address - payload_start_word_address) as usize * 2;
                    assert_eq!(
                        vec_u8[2 + offset_modified..2 + offset_modified + test_values.len()],
                        test_values
                    );

                    payload_ok = true;
                }
                _ => {
                    panic!("Unexpected data");
                }
            }
        }
    }
}
