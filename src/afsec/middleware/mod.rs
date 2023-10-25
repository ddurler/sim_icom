//! Gestion des différents types de conversation entre l'AFSEC+ et l'ICOM
//!
//! Le protocole TLV utilisé entre l'AFSEC+ et l'ICOM définit différentes 'conversations'
//! pourtant sur des sujets particuliers pour synchroniser les informations entre les 2
//! parties (`DATA_IN`, `DATA_OUT`, etc.)
//!
//! La prise en charge d'une conversation particulière est assumée par un `middleware`
//! qui gère le `contexte` de la conversation et sait répondre aux requêtes de l'AFSEC+
//!
//! Messages:
//! * `AF_ALIVE` / `IC_ALIVE`: Pris en charge par `handle_request_data_frame`
//! * `AF_INIT` / `IC_INIT`: Détecté par `handle_request_data_frame`, pris en charge par le middleware `MInit`
//! * `AF_DATA_OUT` / `IC_DATA_OUT`: pris en charge par le middleware `MDataOut`
//! * `AF_DATA_IN` / `IC_DATA_IN`: pris en charge par le middleware `MDataIn`
//! * `AF_DATA_OUT_TABLE_INDEX` / `IC_DATA_OUT_TABLE_INDEX`: pris en charge par le middleware `MDataOutTableIndex`

use crate::{
    afsec::tlv_frame::DataItem,
    database::{IdTag, IdUser},
    t_data::TValue,
};

use super::{DataFrame, DatabaseAfsecComm, RawFrame};

mod id_message;
pub use id_message::*;

mod context;
pub use context::Context;

mod utils;

mod records;
use records::RecordData;

mod m_init;
use m_init::MInit;

mod m_pack_out;
use m_pack_out::MPackOut;

mod m_pack_in;
use m_pack_in::MPackIn;

mod m_data_out;
use m_data_out::MDataOut;

mod m_data_in;
use m_data_in::MDataIn;

mod m_data_out_table_index;
use m_data_out_table_index::MDataOutTableIndex;

mod m_menu;
use m_menu::MMenu;

/// Tag pour la zone `PACK_IN` (en zone 5) ou `PACK_OUT` (en zone 4)
/// Voir SR DEV 004
pub const TAG_DATA_PACK: u16 = 0x0F45;

// On implémente des `middlewares` qu'on peut désigner dynamiquement par `&dyn CommonMiddlewareTrait`.
//
// Mais cette solution nécessite de gérer la `lifetime` des différents `middlewares` ce qui n'est
// pas facile via la structure commune également partagée pour accéder à la `database` de manière
// exclusive (snif).
//
// On simplifie donc en identifiant les `middlewares` dans une liste des `middlewares` qu'on génère
// dynamiquement à chaque fois besoin par `Self::all_middlewares`

/// Identifiant des `middlewares`
/// Il s'agit ici de l'indice du `middleware` dans la liste des `middlewares`
type IdMiddleware = usize;

/// Trait à implémenter pour chaque `middleware`
pub trait CommonMiddlewareTrait {
    /// Fonction appelée lorsque la conversation en cours (s'il y en a une) est terminée.
    /// Indique qu'une nouvelle conversation va débuter
    /// Attention, self n'est pas mutable, il faut utiliser le `context`
    fn reset_conversation(&self, context: &mut Context);

    /// Fonction appelée pour indiquer qu'une trame `RawFrame` est reçue de l'AFSEC+.
    /// Retourne la réponse à faire à l'AFSEC+ si la conversation est prise en charge par ce `middleware`
    /// Attention, self n'est pas mutable, il faut utiliser le `context`
    fn get_conversation(
        &self,
        context: &mut Context,
        afsec_service: &mut DatabaseAfsecComm,
        request_data_frame: &DataFrame,
    ) -> Option<RawFrame>;

    /// Fonction appelée pour indiquer une modification dans le contenu de la `database`
    /// Attention, self n'est pas mutable, il faut utiliser le `context`
    fn notification_change(
        &self,
        context: &mut Context,
        afsec_service: &mut DatabaseAfsecComm,
        id_user: IdUser,
        id_tag: IdTag,
        t_value: &TValue,
    );
}

/// Structure pour la gestion des `middlewares`
pub struct Middlewares {
    /// Contexte pour tous les `middlewares`
    context: Context,

    /// IDMiddleware en cours de conversation
    option_cur_middleware: Option<IdMiddleware>,
}

impl Middlewares {
    /// Constructeur
    pub fn new() -> Self {
        Middlewares {
            context: Context::default(),
            option_cur_middleware: None,
        }
    }

    /// Retourne la liste des `middlewares`
    fn all_middlewares() -> Vec<Box<dyn CommonMiddlewareTrait>> {
        vec![
            // Box::<MInit>::default(),  // Construit sur demande `AF_INIT`
            Box::<MPackOut>::default(),
            Box::<MPackIn>::default(),
            Box::<MDataOut>::default(),
            Box::<MDataIn>::default(),
            Box::<MDataOutTableIndex>::default(),
            Box::<MMenu>::default(),
        ]
    }

    /// Reset conversation de tous les `middlewares`
    fn reset_conversation_all_middlewares(&mut self) {
        for middleware in Self::all_middlewares() {
            middleware.reset_conversation(&mut self.context);
        }
    }

    /// Recherche un `middleware` pour accepter la conversation
    /// Si un `middleware` accepte la conversation, il retourne sa réponse à faire à l'AFSEC+
    /// et il est enregistré comme le `middleware` en cours pour converser.
    fn accept_conversation_all_middlewares(
        &mut self,
        afsec_service: &mut DatabaseAfsecComm,
        request_data_frame: &DataFrame,
    ) -> Option<RawFrame> {
        for (id_middleware, middleware) in Self::all_middlewares().iter().enumerate() {
            if let Some(response_raw_frame) =
                middleware.get_conversation(&mut self.context, afsec_service, request_data_frame)
            {
                self.option_cur_middleware = Some(id_middleware);
                return Some(response_raw_frame);
            }
        }

        // Personne pour cette conversation
        None
    }

    /// Dispatch un changement dans la database à tous les `middlewares`
    pub fn notification_change(
        &mut self,
        afsec_service: &mut DatabaseAfsecComm,
        id_user: IdUser,
        id_tag: IdTag,
        t_value: &TValue,
    ) {
        println!(
            "AFSEC Comm: notification_change id_user={id_user} id_tag={id_tag}, t_value={t_value}"
        );
        for middleware in Self::all_middlewares() {
            middleware.notification_change(
                &mut self.context,
                afsec_service,
                id_user,
                id_tag,
                t_value,
            );
        }
    }

    /// Traite (public) une requête TLV de l'AFSEC+ (au format `RawFrame`)
    /// et retourne la réponse à faire au format `RawFrame`
    pub fn handle_request_raw_frame(
        &mut self,
        afsec_service: &mut DatabaseAfsecComm,
        request_raw_frame: RawFrame,
    ) -> RawFrame {
        match DataFrame::try_from(request_raw_frame) {
            Ok(request_data_frame) => {
                self.handle_request_data_frame(afsec_service, &request_data_frame)
            }
            Err(e) => {
                println!("AFSEC Comm: Got frame with error: {e}");
                // On ne répond rien
                RawFrame::new(&[])
            }
        }
    }

    /// Traite (privé) une requête TLV de l'AFSEC+ au format `DataFrame` (après décodage de la `RawFrame` reçue)
    /// et retourne la réponse à faire au format `RawFrame`
    fn handle_request_data_frame(
        &mut self,
        afsec_service: &mut DatabaseAfsecComm,
        request_data_frame: &DataFrame,
    ) -> RawFrame {
        if request_data_frame.get_tag() == id_message::AF_INIT {
            // L'AFSEC+ annonce une initialisation des communications

            // Reset conversation de tous les `middlewares`
            self.reset_conversation_all_middlewares();

            // Traitement AF_INIT
            let middleware = MInit::default();
            return match middleware.get_conversation(
                &mut self.context,
                afsec_service,
                request_data_frame,
            ) {
                Some(response_raw_frame) => response_raw_frame,
                None => RawFrame::new_nack(),
            };
        }

        // Sinon, on regarde si un `middleware` est déjà en cours de conversation
        if let Some(id_middleware) = &self.option_cur_middleware {
            // Conversation en cours, on passe la requête à ce `middleware`
            let middleware = &Self::all_middlewares()[*id_middleware];
            if let Some(response_raw_frame) =
                middleware.get_conversation(&mut self.context, afsec_service, request_data_frame)
            {
                return response_raw_frame;
            }
            // Le `middleware` qui conversait ne veut plus cette conversation
            self.option_cur_middleware = None;
        }

        // On annonce à tous les `middlewares` qu'une nouvelle conversation peut débuter
        self.reset_conversation_all_middlewares();

        // On recherche un nouveau `middleware` pour accepter la conversation
        if let Some(response_raw_frame) =
            self.accept_conversation_all_middlewares(afsec_service, request_data_frame)
        {
            return response_raw_frame;
        }

        // Pas de `middleware` pour répondre...
        if request_data_frame.get_tag() == id_message::AF_ALIVE {
            // On peut répondre IC_ALIVE ou ACK
            println!("AFSEC Comm: AF_ALIVE...");
            RawFrame::new_ack()
        } else {
            // Répond NACK
            println!("AFSEC Comm: NACK...");
            RawFrame::new_nack()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::{Arc, Mutex};

    use crate::afsec::check_notification_changes;
    use crate::afsec::tlv_frame::DataItem;
    use crate::afsec::tlv_frame::FrameState;
    use crate::afsec::DEBUG_LEVEL_ALL;
    use crate::database::Tag;
    use crate::database::ID_ANONYMOUS_USER;
    use crate::t_data::TFormat;
    use crate::Database;

    // Adresse mot de base pour les 'pack-out'
    const ADDRESS_WORD_PACK_OUT: u16 = 0x4000;

    // Adresse mot de base pour les 'pack-in'
    const ADDRESS_WORD_PACK_IN: u16 = 0x5000;

    // Retourne un tag UI_16 de la database pour faire les tests
    fn test_tag() -> Tag {
        let id_tag = IdTag::new(4, 0x1234, [0, 0, 0]);
        Tag {
            word_address: 0x0800,
            id_tag,
            t_format: TFormat::U16,
            ..Default::default()
        }
    }

    // Création du Mutex database pour le process en communication avec l'AFSEC+
    fn database_setup() -> DatabaseAfsecComm {
        // Création d'une database
        let mut db = Database::default();

        // Création d'un id_user pour le test
        let id_user = db.get_id_user("TEST", true);

        // Création du tag de test
        db.add_tag(&test_tag());

        // Création tags pour les zones 'pack-out' et 'pack-in'
        for (zone, base_address) in [(4_u8, ADDRESS_WORD_PACK_OUT), (5_u8, ADDRESS_WORD_PACK_IN)] {
            for n in 0..8 {
                let id_tag = IdTag::new(zone, TAG_DATA_PACK, [0, 0, n]);
                #[allow(clippy::cast_lossless)]
                let tag = Tag {
                    word_address: base_address + 32 * n as u16,
                    id_tag,
                    t_format: TFormat::VecU8(64),
                    ..Default::default()
                };
                db.add_tag(&tag);
            }
        }

        // Créer la database partagée mutable
        let shared_db = Arc::new(Mutex::new(db));
        // Cloner la référence à la database partagée pour la communication avec l'AFSEC+
        let db_afsec = Arc::clone(&shared_db);

        // Structure pour le thread en communication avec l'AFSEC+
        let mut afsec_service =
            DatabaseAfsecComm::new(db_afsec, "fake".to_string(), DEBUG_LEVEL_ALL);
        afsec_service.id_user = id_user;
        afsec_service
    }

    // Création d'une trame AF_INIT
    fn request_raw_frame_init() -> RawFrame {
        let mut req = RawFrame::new_message(id_message::AF_INIT);
        req.try_extend_data_item(&DataItem::new(
            id_message::D_PROTOCOLE_VERSION,
            TValue::U32(0),
        ))
        .unwrap();
        req.try_extend_data_item(&DataItem::new(
            id_message::D_RESIDENT_VERSION,
            TValue::U32(5_02_00),
        ))
        .unwrap();
        req.try_extend_data_item(&DataItem::new(
            id_message::D_APPLI_NUMBER,
            TValue::I16(-352),
        ))
        .unwrap();
        req.try_extend_data_item(&DataItem::new(
            id_message::D_APPLI_VERSION,
            TValue::U32(13_01_00),
        ))
        .unwrap();
        req.try_extend_data_item(&DataItem::new(
            id_message::D_APPLI_CONFIG,
            TValue::VecU8(4, "TEST".as_bytes().to_vec()),
        ))
        .unwrap();
        req.try_extend_data_item(&DataItem::new(
            id_message::D_LANGUAGE,
            TValue::VecU8(2, "fr".as_bytes().to_vec()),
        ))
        .unwrap();

        req
    }

    // Création d'une trame AF_ALIVE
    fn request_raw_frame_alive() -> RawFrame {
        RawFrame::new_message(id_message::AF_ALIVE)
    }

    // Création d'une trame AF_DATA_OUT
    #[allow(clippy::cast_possible_truncation)]
    fn request_raw_frame_data_out(datas: &[(IdTag, TValue)]) -> RawFrame {
        let mut req = RawFrame::new_message(id_message::AF_DATA_OUT);
        let mut cur_zone = 0xFF;
        for (id_tag, t_value) in datas {
            if cur_zone != id_tag.zone {
                cur_zone = id_tag.zone;
                req.try_extend_data_item(&DataItem::new(
                    id_message::D_DATA_ZONE,
                    TValue::U8(id_tag.zone),
                ))
                .unwrap();
            }

            let vec_u8_tag = vec![
                (id_tag.num_tag / 256) as u8,
                (id_tag.num_tag % 256) as u8,
                id_tag.indice_0,
                id_tag.indice_1,
                id_tag.indice_2,
            ];
            req.try_extend_data_item(&DataItem::new(
                id_message::D_DATA_TAG,
                TValue::VecU8(5, vec_u8_tag),
            ))
            .unwrap();

            req.try_extend_data_item(&DataItem::new(id_message::D_DATA_VALUE, t_value.clone()))
                .unwrap();
        }

        req
    }

    // Création d'une trame AF_PACK_OUT
    #[allow(clippy::cast_possible_truncation)]
    fn request_raw_frame_pack_out(datas: &[(u8, Vec<u8>)]) -> RawFrame {
        let mut req = RawFrame::new_message(id_message::AF_PACK_OUT);
        let nb_packets = datas.len();
        for (i, (address, vec_u8)) in datas.iter().enumerate() {
            let mut vec_u8_payload = vec![];
            // Octet #0: Message num/total (0x12 pour message #1/2)
            vec_u8_payload.push(((i as u8) + 1) * 16 + nb_packets as u8);
            // Octet #1: Adresse mot
            vec_u8_payload.push(*address);
            // Autres octets: Data
            vec_u8_payload.extend(vec_u8);
            req.try_extend_data_item(&DataItem::new(
                id_message::D_PACK_PAYLOAD,
                TValue::VecU8(vec_u8_payload.len(), vec_u8_payload),
            ))
            .unwrap();
        }

        req
    }

    // Vérifie une réponse RawFrame de l'ICOM
    fn ok_response_raw_frame(tag: u8, response: &RawFrame) -> bool {
        assert_eq!(response.get_state(), FrameState::Ok);
        let response = match DataFrame::try_from(response.clone()) {
            Ok(t) => t,
            Err(e) => {
                panic!("{e}");
            }
        };
        assert_eq!(response.get_tag(), tag);

        for data_item in response.get_data_items() {
            assert!(
                data_item.tag != id_message::D_DATA_ERROR,
                "Réponse avec D_DATA_ERROR"
            );
        }

        true
    }

    // Vérifie qu'un ACK est reçu en réponse RawFrame de l'ICOM
    fn ok_ack_raw_frame(response: &RawFrame) -> bool {
        assert_eq!(response.get_state(), FrameState::Ok);
        let response = match DataFrame::try_from(response.clone()) {
            Ok(t) => t,
            Err(e) => {
                panic!("{e}");
            }
        };
        response.is_simple_ack()
    }

    // Simule une modification de la valeur du `test_tag` dans la database
    fn do_update_test_tag(
        afsec_service: &mut DatabaseAfsecComm,
        middlewares: &mut Middlewares,
        value: u16,
    ) {
        // Modification de la database
        {
            // Verrouiller la database partagée
            let mut db: std::sync::MutexGuard<'_, crate::database::Database> =
                afsec_service.thread_db.lock().unwrap();

            db.set_u16_to_id_tag(ID_ANONYMOUS_USER, test_tag().id_tag, value);
        }

        // Active le système de notification
        check_notification_changes(afsec_service, middlewares);
    }

    // Simule une modification du 'pack-in' dans la database
    fn do_update_pack_in(
        afsec_service: &mut DatabaseAfsecComm,
        middlewares: &mut Middlewares,
        address: u16,
        value: &[u8],
    ) {
        // Contrôle cohérence de la modification (256 mots max dans la zone `pack-in`)
        assert!((0..256).contains(&address));
        assert!(address as usize + value.len() / 2 < 256);

        {
            // Verrouiller la database partagée
            let mut db: std::sync::MutexGuard<'_, crate::database::Database> =
                afsec_service.thread_db.lock().unwrap();

            db.set_vec_u8_to_word_address(ID_ANONYMOUS_USER, ADDRESS_WORD_PACK_IN + address, value);
        }

        // Active le système de notification
        check_notification_changes(afsec_service, middlewares);
    }

    #[test]
    fn test_conversation() {
        let mut afsec_service = database_setup();
        let mut middlewares = Middlewares::new();

        // Conversation AF_INIT/IC_INIT (pour débuter)
        let request = request_raw_frame_init();
        let response = middlewares.handle_request_raw_frame(&mut afsec_service, request);
        assert!(ok_response_raw_frame(id_message::IC_INIT, &response));

        // Conversation AF_ALIVE/IC_ALIVE ou ACK (personne n'a rien à dire)
        let request = request_raw_frame_alive();
        let response = middlewares.handle_request_raw_frame(&mut afsec_service, request);
        assert!(
            ok_ack_raw_frame(&response) || ok_response_raw_frame(id_message::IC_ALIVE, &response)
        );

        // Conversation AF_DATA_OUT/IC_DATA_OUT ou ACK
        let request = request_raw_frame_data_out(&[
            (IdTag::new(0, 0x1234, [5, 6, 7]), TValue::U16(123)),
            (IdTag::new(0, 0x2345, [6, 7, 8]), TValue::F32(-123.0)),
            (
                IdTag::new(1, 0x3456, [7, 8, 9]),
                TValue::VecU8(3, "123".as_bytes().to_vec()),
            ),
        ]);
        let response = middlewares.handle_request_raw_frame(&mut afsec_service, request);
        assert!(
            ok_ack_raw_frame(&response)
                || ok_response_raw_frame(id_message::IC_DATA_OUT, &response)
        );

        // Conversation AF_PACK_OUT/IC_PACK_OUT ou ACK
        let request = request_raw_frame_pack_out(&[
            (0, vec![0_u8, 1_u8, 2_u8, 3_u8]),
            (100, vec![100_u8, 101_u8, 102_u8, 103_u8]),
        ]);
        let response = middlewares.handle_request_raw_frame(&mut afsec_service, request);
        assert!(
            ok_ack_raw_frame(&response)
                || ok_response_raw_frame(id_message::IC_PACK_OUT, &response)
        );

        // Simule une modification de la valeur du tag de test
        do_update_test_tag(&mut afsec_service, &mut middlewares, 123);

        // Conversation AF_ALIVE -> DATA_IN pour informer de cette modification
        let request = request_raw_frame_alive();
        let response = middlewares.handle_request_raw_frame(&mut afsec_service, request);
        assert!(ok_response_raw_frame(id_message::IC_DATA_IN, &response));

        // Simule une modification de la zone 'pack-in'
        do_update_pack_in(
            &mut afsec_service,
            &mut middlewares,
            10,
            &[1_u8, 2_u8, 3_u8, 4_u8],
        );

        // Conversation AF_ALIVE -> PACK_IN pour informer de cette modification
        let request = request_raw_frame_alive();
        let response = middlewares.handle_request_raw_frame(&mut afsec_service, request);
        assert!(ok_response_raw_frame(id_message::IC_PACK_IN, &response));

        // Conversation AF_ALIVE/IC_ALIVE ou ACK (pour confirmer que plus personne n'a rien à dire)
        let request = request_raw_frame_alive();
        let response = middlewares.handle_request_raw_frame(&mut afsec_service, request);
        assert!(
            ok_ack_raw_frame(&response) || ok_response_raw_frame(id_message::IC_ALIVE, &response)
        );
    }
}
