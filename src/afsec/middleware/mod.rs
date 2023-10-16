//! Gestion des différents types de conversation entre l'AFSEC+ et l'ICOM
//!
//! Le protocole TLV utilisé entre l'AFSEC+ et l'ICOM définit différentes 'conversations'
//! pourtant sur des sujets particuliers pour synchroniser les informations entre les 2
//! parties (`DATA_IN`, `DATA_OUT`, etc.)
//!
//! La prise en charge d'une conversation particulière est assumée par un `middleware`
//! qui gère le `contexte` de la conversation et sait répondre aux requêtes de l'AFSEC+

use crate::{
    afsec::tlv_frame::DataItem,
    database::{IdTag, IdUser},
    t_data::{TFormat, TValue},
};

use super::{DataFrame, DatabaseAfsecComm, RawFrame};

mod id_message;
pub use id_message::*;

// Pour bien faire, il faudrait implémenter des `middlewares` qu'on peut désigner dynamiquement
// par `&dyn CommonMiddlewareTrait`.
// Mais cette solution nécessite de gérer la `lifetime` des différents `middlewares` ce qui n'est
// pas facile via la structure commune également partagée pour accéder à la `database` de manière
// exclusive (snif).
//
// On simplifie donc en identifiant les `middlewares` par un `IdMidddleware`.

/// Identifiant d'un `middleware`
pub type IdMidddleware = usize;

/// Trait à implémenter pour chaque `middleware`
pub trait CommonMiddlewareTrait {
    /// Initialisation une fois au démarrage du 'middleware'
    fn init(&mut self);

    /// Fonction appelée lorsqu'un `AF_INIT/IC_INIT` de la conversation est fait entre
    /// l'AFSEC+ et l'ICOM.
    /// Termine la conversation en cours (s'il y en a une) et réinitialise le contexte
    /// de tous les `middleware`
    fn init_conversation(&mut self);

    /// Fonction appelée lorsque la conversation en cours (s'il y en a une) est terminée.
    /// Indique qu'une nouvelle conversation va débuter
    fn reset_conversation(&mut self);

    /// Fonction appelée pour indiquer qu'une trame `RawFrame` est reçue de l'AFSEC+.
    /// Retourne la réponse à faire à l'AFSEC+ si la conversation est prise en charge par ce `middleware`
    fn get_conversation(
        &mut self,
        afsec_service: &mut DatabaseAfsecComm,
        req_frame: &DataFrame,
        is_already_conversing: bool,
    ) -> Option<RawFrame>;

    /// Fonction appelée pour indiquer une modification dans le contenu de la `database`
    fn notification_change(
        &mut self,
        afsec_service: &mut DatabaseAfsecComm,
        id_user: IdUser,
        id_tag: IdTag,
        t_value: &TValue,
    );
}

/// Structure pour la gestion des `middlewares`
pub struct Middlewares {
    /// IDMiddleware qui détient la conversation
    option_cur_id_middleware: Option<IdMidddleware>,
}

impl Middlewares {
    /// Constructeur
    pub fn new() -> Self {
        let mut ret = Middlewares {
            option_cur_id_middleware: None,
        };
        ret.init_all_middlewares();
        ret
    }

    /// Initialisation des `middlewares` (fait une fois au démarrage du thread de communication avec l'AFSEC+)
    fn init_all_middlewares(&mut self) {
        println!("AFSEC Comm: init_all_middlewares");
    }

    /// Reset conversation de tous les `middlewares`
    fn reset_conversation_all_middlewares(&mut self) {
        println!("AFSEC Comm: reset_conversation_all_middlewares");
    }

    /// Dispatch un changement dans la database à tous les middlewares
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
    }

    /// Traite (public) une requête TLV de l'AFSEC+ (au format `RawFrame`)
    /// et retourne la réponse à faire au format `RawFrame`
    pub fn handle_req_raw_frame(
        &mut self,
        afsec_service: &mut DatabaseAfsecComm,
        raw_frame: RawFrame,
    ) -> RawFrame {
        match DataFrame::try_from(raw_frame) {
            Ok(data_frame) => self.handle_req_data_frame(afsec_service, &data_frame),
            Err(e) => {
                println!("AFSEC Comm: Got frame with error : {e}");
                // On ne répond rien
                RawFrame::new(&[])
            }
        }
    }

    /// Traite (privé) une requête TLV de l'AFSEC+ au format `DataFrame` (après décodage de la `RawFrame` reçue)
    /// et retourne la réponse à faire au format `RawFrame`
    fn handle_req_data_frame(
        &mut self,
        afsec_service: &mut DatabaseAfsecComm,
        req_tlv: &DataFrame,
    ) -> RawFrame {
        if req_tlv.get_tag() == id_message::AF_INIT {
            // L'AFSEC+ annonce une initialisation des communications
            // TODO : println! des données de ce message
            println!("AFSEC Comm.: AF_INIT...");

            // Reset conversation de tous les middlewares
            self.reset_conversation_all_middlewares();

            // Répond AF_INIT
            let mut raw_frame = RawFrame::new_message(id_message::IC_INIT);
            raw_frame
                .try_extend_data_item(&DataItem::new(
                    id_message::D_PROTOCOLE_VERSION,
                    TFormat::U16,
                    TValue::U16(0),
                ))
                .unwrap();
            raw_frame
                .try_extend_data_item(&DataItem::new(
                    id_message::D_ICOM_VERSION,
                    TFormat::U16,
                    TValue::U16(0),
                ))
                .unwrap();

            return raw_frame;
        }

        {
            // Verrouiller la database partagée
            let mut db = afsec_service.thread_db.lock().unwrap();

            // TODO : Juste pour voir
            db.set_i8_to_word_address(afsec_service.id_user, 0, 5);
        }

        // Réponse vide pour l'instant
        RawFrame::new(&[])
    }
}
