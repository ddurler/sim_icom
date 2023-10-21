//! Contexte d'exécution pour les différents `middlewares`

use std::collections::HashSet;

use super::{IdTag, RecordData, TValue};

/// Structure de contexte commune à tous les `middlewares`
// ATTENTION: Chaque `middleware` ne doit pas avoir sa propre structure de données
// (la liste des `middlewares` est régénérée périodiquement)
// => C'est la structure générique `Context` qui doit être utilisée comme `context` pour ce besoin
#[derive(Debug, Default)]
pub struct Context {
    /// Nombre de INIT depuis le début
    pub nb_init: usize,

    /// Nombre de PACK_OUT depuis le début
    pub nb_pack_out: usize,

    /// Nombre de PACK_IN depuis le début
    pub nb_pack_in: usize,

    /// Nombre de DATA_OUT depuis le début
    pub nb_data_out: usize,

    /// Nombre de DATA_IN depuis le début
    pub nb_data_in: usize,

    /// Numéro de zone de la conversation en cours
    pub option_zone: Option<u8>,

    /// `TABLE_INDEX` de la conversation en cours
    pub option_table_index: Option<u64>,

    /// `Tag` de la conversation en cours (`Vec<u8>` de 5 = U16 + 3 x U8)
    pub option_vec_u8_tag: Option<Vec<u8>>,

    /// `TValue` de la conversation en cours
    pub option_t_value: Option<TValue>,

    /// `RecordData` vus pendant la conversation DATA_OUT
    pub record_datas: Vec<RecordData>,

    /// Liste des notification_changes pour la conversation DATA_IN
    pub notification_changes: Vec<(IdTag, TValue)>,

    /// Contexte pour les transactions 'pack-in'
    pub pack_in: PackIn,

    /// Contexte pour les transactions 'pack-out'
    pub pack_out: PackOut,
}

/// Sous-structure du contexte pour les transactions 'pack-in'
#[derive(Debug, Default)]
pub struct PackIn {
    /// Indicateur à true lorsqu'une transaction 'pack_in' est en cours
    pub is_transaction: bool,

    /// Ensemble des `PACK_IN`` à transmettre à l'ICOM pour la transaction `pack_in`
    /// On représente ici les 8 `blocs` de 32 mots `TAG_DATA_PACK` de la zone de commande (zone 5)
    /// par un entier de 0 à 7 dans un `HashSet`
    pub set_blocs: HashSet<u8>,

    /// Copie privée des données de la transaction `pack-in` en cours
    /// (.0 est le numéro de bloc 0-7 et .1 contient les données)
    pub private_datas: Vec<(u8, Vec<u8>)>,

    /// Ensemble des PACK_IN à pour la transaction `pack_in` à suivre
    pub set_pending_blocs: HashSet<u8>,
}

/// Sous-structure du contexte pour les transactions 'pack-out'
#[derive(Debug, Default)]
pub struct PackOut {
    /// Indicateur à true lorsqu'une transaction 'pack_in' est en cours
    pub is_transaction: bool,

    /// Nombre de paquets annoncés pour la transaction
    pub option_nb_total_packets: Option<u8>,

    /// Numéro du dernier paquets reçus
    pub option_last_num_packet: Option<u8>,

    /// Copie privée des données de la transaction `pack-in` en cours
    /// (.0 est l'adresse mot (0-255) de début et .1 contient les données)
    pub private_datas: Vec<(u8, Vec<u8>)>,
}
