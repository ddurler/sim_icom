//! `middleware` pour le traitement `AF_INIT`

use super::{
    id_message, utils, CommonMiddlewareTrait, Context, DataFrame, DataItem, DatabaseAfsecComm,
    IdTag, IdUser, RawFrame, TFormat, TValue,
};

#[derive(Default)]
pub struct MInit {}

impl CommonMiddlewareTrait for MInit {
    fn reset_conversation(&self, _context: &mut Context) {}

    fn get_conversation(
        &self,
        context: &mut Context,
        afsec_service: &mut DatabaseAfsecComm,
        request_data_frame: &DataFrame,
        _is_already_conversing: bool,
    ) -> Option<RawFrame> {
        if request_data_frame.get_tag() != id_message::AF_INIT {
            return None;
        }
        // Décompte des AF_INIT traités
        context.nb_init += 1;
        println!("AFSEC Comm.: AF_INIT #{}...", context.nb_init);

        // Exploitation des informations reçues et mise à jour de la database
        for data_item in request_data_frame.get_data_items() {
            match data_item.tag {
                id_message::D_RESIDENT_VERSION => {
                    let version_revision_edition = u32::from(&data_item.t_value);
                    let (version, revision, edition) =
                        utils::get_version_revision_edition_from_u32(version_revision_edition);
                    utils::update_database(
                        afsec_service,
                        IdTag::new(0, 0x0001, [0, 0, 0]),
                        TValue::U16(version),
                    );
                    utils::update_database(
                        afsec_service,
                        IdTag::new(0, 0x0002, [0, 0, 0]),
                        TValue::U16(revision),
                    );
                    utils::update_database(
                        afsec_service,
                        IdTag::new(0, 0x0003, [0, 0, 0]),
                        TValue::U16(edition),
                    );
                }
                id_message::D_APPLI_NUMBER => {
                    utils::update_database(
                        afsec_service,
                        IdTag::new(0, 0x0010, [0, 0, 0]),
                        data_item.t_value,
                    );
                }
                id_message::D_APPLI_VERSION => {
                    let version_revision_edition = u32::from(&data_item.t_value);
                    let (version, revision, edition) =
                        utils::get_version_revision_edition_from_u32(version_revision_edition);
                    utils::update_database(
                        afsec_service,
                        IdTag::new(0, 0x0011, [0, 0, 0]),
                        TValue::U16(version),
                    );
                    utils::update_database(
                        afsec_service,
                        IdTag::new(0, 0x0012, [0, 0, 0]),
                        TValue::U16(revision),
                    );
                    utils::update_database(
                        afsec_service,
                        IdTag::new(0, 0x0013, [0, 0, 0]),
                        TValue::U16(edition),
                    );
                }
                id_message::D_APPLI_CONFIG => {
                    utils::update_database(
                        afsec_service,
                        IdTag::new(0, 0x0014, [0, 0, 0]),
                        data_item.t_value,
                    );
                }
                id_message::D_LANGUAGE => {
                    utils::update_database(
                        afsec_service,
                        IdTag::new(1, 0x2042, [0, 0, 0]),
                        data_item.t_value,
                    );
                }
                _ => (),
            }
        }

        // Création de la réponse
        let mut response_raw_frame = RawFrame::new_message(id_message::IC_INIT);
        response_raw_frame
            .try_extend_data_item(&DataItem::new(
                id_message::D_PROTOCOLE_VERSION,
                TFormat::U16,
                TValue::U16(0),
            ))
            .unwrap();
        response_raw_frame
            .try_extend_data_item(&DataItem::new(
                id_message::D_ICOM_VERSION,
                TFormat::U16,
                TValue::U16(0),
            ))
            .unwrap();

        // Réponse
        Some(response_raw_frame)
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
