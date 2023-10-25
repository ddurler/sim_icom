//! `middleware` pour le traitement `AF_PACK_OUT`
//!
//! Prend en charge les conversations `AF_PACK_OUT` du résident qui transmet des données.
//! Il s'agit ici de `blocs` de données pour renseigner une partie de données brutes' de la `Database`
//! contenant 256 mots.
//!
//! Une trame `AF_PACK_OUT` ne pouvant pas contenir l'ensemble de la zone de 256 mots, la mise à
//! jour ce fait par petits `blocs` numérotés. Une transaction commence lorsqu'un premier bloc est reçu.
//! Les données des blocs sont mémorisées dans une zone privée. Lorsque le dernier bloc est reçu, la
//! `database` est mise à jour de tous les blocs reçus et la transaction se termine.
//!
//! Ce `middleware` utilise plusieurs infos dans le contexte:
//!
//! * `is_transaction`: `bool`: Ce flag est à true lorsqu'une transaction de données `pack_out` est en cours.
//!     Dans ce cas, les données reçues sont `private_datas`
//! * `option_nb_total_packets: Option<u8>` : Contient le nombre de paquets annoncés dans la transaction
//! * `option_last_num_packet: Option<u8>` : Contient le numéro du dernier paquets reçus
//! * `private_datas: Vec<(u8, Vec<u8>)>` : Contient la liste des paquets reçus pendant la transaction avec
//!   * .0 : est l'adresse mot (0-255) du début des données dans la zone dédiée de la `database`
//!   * .1 : est le contenu des octets à partir de cette adresse
//!   Lorsque la transaction se termine à la réception du dernier paquet, les données dans `private_datas`
//!   sont mises à jour dans la `database`

use std::vec;

use super::{
    id_message, CommonMiddlewareTrait, Context, DataFrame, DatabaseAfsecComm, IdTag, IdUser,
    RawFrame, TValue, TAG_DATA_PACK,
};

#[derive(Default)]
pub struct MPackOut {}

impl CommonMiddlewareTrait for MPackOut {
    fn reset_conversation(&self, _context: &mut Context) {}

    fn get_conversation(
        &self,
        context: &mut Context,
        afsec_service: &mut DatabaseAfsecComm,
        request_data_frame: &DataFrame,
    ) -> Option<RawFrame> {
        if request_data_frame.get_tag() != id_message::AF_PACK_OUT {
            return None;
        }

        // Décompte des AF_PACK_OUT traités
        context.nb_pack_out += 1;
        println!("AFSEC Comm: AF_PACK_OUT #{}...", context.nb_pack_out);

        // Vérifie si transaction en cours ou s'il faut démarrer une nouvelle transaction
        if !context.pack_out.is_transaction {
            // Début d'une transaction `pack_out`
            MPackOut::start_transaction(context);
        }

        // Indicateur de dernier paquet reçu
        let mut last_packet_received = false;

        // Exploitation des packets reçus
        for data_item in request_data_frame.get_data_items() {
            if data_item.tag == id_message::D_PACK_PAYLOAD {
                if last_packet_received {
                    println!("AFSEC Comm: AF_PACK_OUT got packet after receiving last packet ???");
                }
                let vec_u8 = data_item.t_value.to_vec_u8();
                if vec_u8.len() >= 2 {
                    // Octet #0: Numéro de packet/total packet (exemple 0x12 pour paquet 1/2)
                    let total_nb_packets = vec_u8[0] % 16;
                    let num_packet = vec_u8[0] / 16;
                    // Vérifie consistance du nombre total de paquets
                    if let Some(nb) = context.pack_out.option_nb_total_packets {
                        if nb != total_nb_packets {
                            println!("AFSEC Comm: AF_PACK_OUT change in total #packets {nb} to {total_nb_packets} ???");
                        }
                    } else {
                        context.pack_out.option_nb_total_packets = Some(total_nb_packets);
                    }
                    // Vérifie consistance numérotation des paquets
                    if let Some(last_num_packet) = context.pack_out.option_last_num_packet {
                        if num_packet != last_num_packet + 1 {
                            println!("AFSEC Comm: AF_PACK_OUT missing packet between #{last_num_packet} and #{num_packet} ???",);
                        }
                    } else if num_packet != 1 {
                        println!("AFSEC Comm: AF_PACK_OUT got first packet with number #{num_packet} ???",);
                    }
                    context.pack_out.option_last_num_packet = Some(num_packet);

                    // Octet #1: Adresse mot des données
                    let word_address = vec_u8[1];

                    // Tous les autres octets sont les données du paquet
                    let data = vec_u8[2..].to_vec();

                    // Mémorisation des données du paquet reçu
                    context.pack_out.private_datas.push((word_address, data));

                    // Dernier paquet ?
                    last_packet_received = num_packet == total_nb_packets;
                } else {
                    println!(
                        "AFSEC Comm: AF_PACK_OUT got too short data (len={}) ???",
                        vec_u8.len()
                    );
                }
            } else {
                println!(
                    "AFSEC Comm: AF_PACK_OUT got unexpected id_tag {} ???",
                    data_item.tag
                );
            }
        }

        // Si le dernier paquet a été reçu, on termine la transaction avec la mise à jour de la database
        if last_packet_received {
            MPackOut::end_transaction(context, afsec_service);
        }

        // Réponse (toujours ACK)
        // TODO faut-il répondre NACK lorsque des erreurs sont détectées (voir ci-dessus) ?
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

impl MPackOut {
    /// Nouvelle transaction `pack-in`
    fn start_transaction(context: &mut Context) {
        if context.pack_out.is_transaction {
            // Transaction déjà en cours...
            return;
        }

        // Démarre la transaction
        context.pack_out.is_transaction = true;
        println!("AFSEC Comm: AF_PACK_OUT starts new transaction");

        // Préparation des données pour la transaction
        context.pack_out.option_nb_total_packets = None;
        context.pack_out.option_last_num_packet = None;
        context.pack_out.private_datas = vec![];
    }

    /// Termine la transaction `pack-in` en cours
    fn end_transaction(context: &mut Context, afsec_service: &mut DatabaseAfsecComm) {
        if !context.pack_out.is_transaction {
            // Pas de transaction en cours...
            return;
        }

        // Mise à jour de la database avec les informations collectées en privé pendant la transaction
        // On recherche tout d'abord l'adresse mot de base de la zone pour le pack_out dans la zone
        // de supervision (zone 4)
        let id_tag = IdTag::new(4, TAG_DATA_PACK, [0, 0, 0]);
        let some_base_word_address = {
            // Verrouiller la database partagée
            let db: std::sync::MutexGuard<'_, crate::database::Database> =
                afsec_service.thread_db.lock().unwrap();

            db.get_tag_from_id_tag(id_tag).map(|tag| tag.word_address)
        };

        if let Some(base_word_address) = some_base_word_address {
            // Parcourt des paquets de la copie privée mémorisée pendant la transaction
            for (word_address, vec_u8) in &context.pack_out.private_datas {
                #[allow(clippy::cast_lossless)]
                let word_address = base_word_address + *word_address as u16;
                {
                    // Verrouiller la database partagée
                    let mut db: std::sync::MutexGuard<'_, crate::database::Database> =
                        afsec_service.thread_db.lock().unwrap();

                    println!("AFSEC Comm: AF_PACK_OUT update @{word_address:04X} = {vec_u8:?}");
                    db.set_vec_u8_to_word_address(afsec_service.id_user, word_address, vec_u8);
                };
            }
        } else {
            println!("AFSEC Comm: AF_PACK_OUT with no word address in database for {id_tag} ???");
        }

        // Clear des données de la transaction
        context.pack_out.option_nb_total_packets = None;
        context.pack_out.option_last_num_packet = None;
        context.pack_out.private_datas = vec![];

        // Hors transaction maintenant
        context.pack_out.is_transaction = false;
        println!("AFSEC Comm: AF_PACK_OUT ends transaction");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::{Arc, Mutex};

    use crate::afsec::tlv_frame::DataItem;
    use crate::database::ID_ANONYMOUS_USER;
    use crate::t_data::TFormat;
    use crate::{database::Tag, Database};

    #[test]
    fn test_conversation() {
        // Création d'une database
        let mut db = Database::default();

        // Adresse (arbitraire) de la zone 'pack-out' dans la database
        let word_address_pack_out = 0x0010;

        // id_tag correspondant à la 1ere zone 'pack-out (en zone 4) dans la database
        let id_tag = IdTag::new(4, TAG_DATA_PACK, [0, 0, 0]);
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
        let mut context = Context::default();
        let mut afsec_service = DatabaseAfsecComm::new(db_afsec, "fake".to_string());

        // Création d'une requête AFSEC+ AF_PACK_OUT pour changer la valeur dans le pack-out
        let mut request = RawFrame::new_message(id_message::AF_PACK_OUT);
        let mut payload = vec![];
        // Octet #0: num_paquet/total_paquet
        payload.push(0x11);
        // Octet #1: adresse mot
        #[allow(clippy::cast_possible_truncation)]
        payload.push(test_address as u8);
        // Octets suivants avec les valeurs
        payload.extend(test_values.clone());
        let data_item = DataItem::new(
            id_message::D_PACK_PAYLOAD,
            TValue::VecU8(payload.len(), payload),
        );
        request.try_extend_data_item(&data_item).unwrap();
        let request = DataFrame::try_from(request).unwrap();

        // Envoi du message au middleware
        let middleware = MPackOut::default();
        let response = middleware
            .get_conversation(&mut context, &mut afsec_service, &request)
            .unwrap();

        // Le middleware doit avoir réponde ACK
        assert_eq!(response, RawFrame::new_ack());

        // Et on doit maintenant lire les valeurs dans la zone pack-out de la database
        {
            // Verrouiller la database partagée
            let db: std::sync::MutexGuard<'_, crate::database::Database> =
                afsec_service.thread_db.lock().unwrap();

            assert_eq!(
                db.get_vec_u8_from_word_address(
                    ID_ANONYMOUS_USER,
                    word_address_pack_out + test_address,
                    test_values.len()
                ),
                test_values
            );
        }
    }
}
