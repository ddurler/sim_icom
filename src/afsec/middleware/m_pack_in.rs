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
    IdUser, RawFrame, TFormat, TValue, TAG_DATA_PACK,
};

#[derive(Default)]
pub struct MPackIn {}

impl CommonMiddlewareTrait for MPackIn {
    fn reset_conversation(&self, _context: &mut Context) {}

    #[allow(clippy::cast_possible_truncation)]
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

        // Décompte des AF_DATA_IN traités
        context.nb_data_in += 1;
        println!("AFSEC Comm: AF_PACK_IN #{}...", context.nb_pack_in);

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

            // Adresse `mot` de ce bloc[0-255]
            let cur_word_address = bloc * 32;

            // Numéro du bloc [1-total_nb_blocs]
            // On calcule 1 pour le 1er bloc transmis et `total_nb_blocs` pour le dernier bloc
            let num_bloc = total_nb_blocs - context.pack_in.private_datas.len() + 1;

            // Payload de ce bloc
            let mut vec_u8 = vec![];

            // Octet #0: numéro de bloc+nombre total de blocs (0x12 pour dire bloc #1 pour un total de 2)
            vec_u8.push(16 * num_bloc as u8 + total_nb_blocs as u8);

            // Octet #1: adresse mot du bloc
            vec_u8.push(cur_word_address);

            // Le reste est le contenu du bloc
            vec_u8.extend(&context.pack_in.private_datas[0].1);

            // Taille du payload (normalement 2 + 64 = 66)
            let width = vec_u8.len();

            // Tente d'ajouter ce payload dans le message
            let data_item = DataItem::new(
                id_message::D_PACK_PAYLOAD,
                TFormat::VecU8(width),
                TValue::VecU8(width, vec_u8),
            );
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
        println!("AFSEC Comm: AF_PACK_IN replies with packets #{vec_blocs:?}/{total_nb_blocs}");

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
        println!(
            "AFSEC Comm: AF_PACK_IN starts new transaction with #{} packets",
            context.pack_in.set_blocs.len()
        );

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
        println!("AFSEC Comm: AF_PACK_IN ends transaction");
    }
}
