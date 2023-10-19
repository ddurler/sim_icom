//! `middleware` pour le traitement `AF_DATA_OUT`

use super::{
    id_message, utils, CommonMiddlewareTrait, Context, DataFrame, DatabaseAfsecComm, IdTag, IdUser,
    RawFrame, TValue,
};

#[derive(Default)]
pub struct MDataOut {}

impl CommonMiddlewareTrait for MDataOut {
    fn reset_conversation(&self, context: &mut Context) {
        // Table index et le numéro de zone sont contextuels et peuvent être valides pour plusieurs trames
        context.option_table_index = None;
        context.option_zone = None;
    }

    fn get_conversation(
        &self,
        context: &mut Context,
        afsec_service: &mut DatabaseAfsecComm,
        request_data_frame: &DataFrame,
        _is_already_conversing: bool,
    ) -> Option<RawFrame> {
        if request_data_frame.get_tag() != id_message::AF_DATA_OUT {
            return None;
        }
        // Décompte des AF_DATA_OUT traités
        context.nb_data_out += 1;
        println!("AFSEC Comm.: AF_DATA_OUT #{}...", context.nb_data_out);

        // Init avant traitement
        context.option_vec_u8_tag = None;
        context.option_t_value = None;

        // Exploitation des informations reçues et mise à jour de la database
        for data_item in request_data_frame.get_data_items() {
            match data_item.tag {
                id_message::D_DATA_ZONE => context.option_zone = Some(u8::from(&data_item.t_value)),
                id_message::D_DATA_TABLE_INDEX => {
                    context.option_table_index = Some(u64::from(&data_item.t_value));
                }
                id_message::D_DATA_TAG => {
                    let tag_as_string = data_item.t_value.to_t_value_vec_u8(5);
                    if let TValue::VecU8(_, vec_u8) = tag_as_string {
                        context.option_vec_u8_tag = Some(vec_u8);
                    }
                }
                id_message::D_DATA_VALUE => context.option_t_value = Some(data_item.t_value),
                _ => (),
            }

            // Si on a reçu au moins zone + str5_tag + t_value
            if let Some(zone) = context.option_zone {
                if let Some(vec_u8_tag) = &context.option_vec_u8_tag {
                    let id_tag = utils::get_id_tag_from_zone_vec_u8_tag(zone, vec_u8_tag);
                    if let Some(t_value) = &context.option_t_value {
                        // Mise à jour de la database
                        utils::update_database(afsec_service, id_tag, t_value.clone());
                        // RAZ après traitement
                        context.option_vec_u8_tag = None;
                        context.option_t_value = None;
                    }
                }
            }
        }

        // Réponse
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
