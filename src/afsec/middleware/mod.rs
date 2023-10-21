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

use std::collections::HashSet;

use crate::{
    afsec::tlv_frame::DataItem,
    database::{IdTag, IdUser},
    t_data::{TFormat, TValue},
};

use super::{DataFrame, DatabaseAfsecComm, RawFrame};

mod id_message;
pub use id_message::*;

mod utils;

mod records;
use records::RecordData;

mod m_init;
use m_init::MInit;

mod m_pack_in;
use m_pack_in::MPackIn;

mod m_data_out;
use m_data_out::MDataOut;

mod m_data_in;
use m_data_in::MDataIn;

mod m_data_out_table_index;
use m_data_out_table_index::MDataOutTableIndex;

/// Tag pour un `END_OF_RECORD` d'un `DATA_OUT` lors d'un enregistrement d'un journal
/// Voir SR DEV 004
pub const TAG_NUM_END_OF_RECORD: u16 = 0x7210;

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

/// Structure de contexte commune à tous les `middlewares`
// ATTENTION: Chaque `middleware` ne doit pas avoir sa propre structure de données
// (la liste des `middlewares` est régénérée périodiquement (voir commentaire ci-dessus))
// => C'est la structure générique `Context` qui doit être utilisée comme `context` pour ce besoin
#[derive(Debug, Default)]
pub struct Context {
    /// Nombre de INIT depuis le début
    nb_init: usize,

    /// Nombre de PACK_IN depuis le début
    nb_pack_in: usize,

    /// Nombre de DATA_OUT depuis le début
    nb_data_out: usize,

    /// Nombre de DATA_IN depuis le début
    nb_data_in: usize,

    /// Numéro de zone de la conversation en cours
    option_zone: Option<u8>,

    /// `TABLE_INDEX` de la conversation en cours
    option_table_index: Option<u64>,

    /// `Tag` de la conversation en cours (`Vec<u8>` de 5 = U16 + 3 x U8)
    option_vec_u8_tag: Option<Vec<u8>>,

    /// `TValue` de la conversation en cours
    option_t_value: Option<TValue>,

    /// `RecordData` vus pendant la conversation DATA_OUT
    record_datas: Vec<RecordData>,

    /// Liste des notification_changes pour la conversation DATA_IN
    notification_changes: Vec<(IdTag, TValue)>,

    /// Indicateur à true lorsqu'une transaction 'pack_in' est en cours
    pack_in_transaction: bool,

    /// Ensemble des `PACK_IN`` à transmettre à l'ICOM pour la transaction `pack_in`
    /// On représente ici les 8 zones de 32 mots `TAG_DATA_PACK` de la zone de commande (zone 5)
    /// par un entier de 0 à 7 dans un `HashSet`
    set_pack_in: HashSet<u8>,

    /// Copie privée des données de la transaction `pack-in` en cours
    pack_in_datas: Vec<(u8, Vec<u8>)>,

    /// Ensemble des PACK_IN à pour la transaction `pack_in` à suivre
    set_pending_pack_in: HashSet<u8>,
}

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
        is_already_conversing: bool,
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
            Box::<MPackIn>::default(),
            Box::<MDataOut>::default(),
            Box::<MDataIn>::default(),
            Box::<MDataOutTableIndex>::default(),
        ]
    }

    /// Reset conversation de tous les `middlewares`
    fn reset_conversation_all_middlewares(&mut self) {
        println!("AFSEC Comm: reset_conversation_all_middlewares");
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
            if let Some(response_raw_frame) = middleware.get_conversation(
                &mut self.context,
                afsec_service,
                request_data_frame,
                false,
            ) {
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
                false,
            ) {
                Some(response_raw_frame) => response_raw_frame,
                None => RawFrame::new_nack(),
            };
        }

        // Sinon, on regarde si un `middleware` est déjà en cours de conversation
        if let Some(id_middleware) = &self.option_cur_middleware {
            // Conversation en cours, on passe la requête à ce `middleware`
            let middleware = &Self::all_middlewares()[*id_middleware];
            if let Some(response_raw_frame) = middleware.get_conversation(
                &mut self.context,
                afsec_service,
                request_data_frame,
                true,
            ) {
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
            // On répond IC_ALIVE
            println!("AFSEC Comm: AF_ALIVE...");
            RawFrame::new_message(id_message::IC_ALIVE)
        } else {
            // Répond NACK
            println!("AFSEC Comm: NACK...");
            RawFrame::new_nack()
        }
    }
}
