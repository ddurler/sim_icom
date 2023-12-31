//! `middleware` pour le traitement `AF_DATA_OUT_TABLE`
//!
//! Prend en charge une conversation pour synchroniser les tables des enregistrements avec à l'AFSEC+.
//! Il peut y avoir une table par zone mais dans la pratique il n'y a que les enregistrements
//! des résultats de mesurages (zone = 2, associée à la zone = 6 pour sa relecture) et les
//! enregistrements des événements (zone = 3, associée à la zone = 7 pour sa relecture)
//!
//! Le simulateur n'enregistre que les min/max des indices vus pour les différentes zone (voir `context.records`)

use crate::afsec::DEBUG_LEVEL_SOME;

use super::{
    id_message, CommonMiddlewareTrait, Context, DataFrame, DataItem, DatabaseAfsecComm, IdTag,
    IdUser, RawFrame, TValue,
};

#[derive(Default)]
pub struct MDataOutTableIndex {}

impl CommonMiddlewareTrait for MDataOutTableIndex {
    fn reset_conversation(&self, _context: &mut Context) {}

    fn get_conversation(
        &self,
        context: &mut Context,
        _afsec_service: &mut DatabaseAfsecComm,
        request_data_frame: &DataFrame,
    ) -> Option<RawFrame> {
        if request_data_frame.get_tag() != id_message::AF_DATA_OUT_TABLE_INDEX {
            // Non concerné par cette conversation
            return None;
        }

        // Il doit y avoir un numéro de zone dans la requête de l'AFSEC+
        let mut option_zone: Option<u8> = None;
        for data_item in request_data_frame.get_data_items() {
            if data_item.tag == id_message::D_DATA_ZONE {
                option_zone = Some(u8::from(&data_item.t_value));
                break;
            }
        }

        if option_zone.is_none() {
            // Étrange
            if context.debug_level >= DEBUG_LEVEL_SOME {
                println!("AFSEC Com: Got AF_DATA_OUT_TABLE_INDEX message without zone ???");
            }
            return Some(RawFrame::new_nack());
        }

        let cur_zone = option_zone.unwrap();

        // Préparation d'un message `IC_DATA_OUT_TABLE_INDEX` pour transmettre les indices à l'AFSEC+
        let mut raw_frame = RawFrame::new_message(id_message::IC_DATA_OUT_TABLE_INDEX);

        // Zone
        let data_item = DataItem::new(id_message::D_DATA_ZONE, TValue::U8(cur_zone));
        raw_frame.try_extend_data_item(&data_item).unwrap();

        // First index
        let index_min = context.records.get_index_min(cur_zone);
        let data_item = DataItem::new(id_message::D_DATA_FIRST_TABLE_INDEX, TValue::U64(index_min));
        raw_frame.try_extend_data_item(&data_item).unwrap();

        // Last index
        let index_max = context.records.get_index_max(cur_zone);
        let data_item = DataItem::new(id_message::D_DATA_FIRST_TABLE_INDEX, TValue::U64(index_max));
        raw_frame.try_extend_data_item(&data_item).unwrap();

        // Réponse
        Some(raw_frame)
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
