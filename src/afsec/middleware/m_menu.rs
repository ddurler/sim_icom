//! `middleware` pour le traitement `AF_MENU`
//!
//! Le simulateur ICOM ne gère pas de menu.
//! Toute tentative de conversation pour des menus par l'AFSEC+ aboutira à une réponse NACK

use super::{
    id_message, CommonMiddlewareTrait, Context, DataFrame, DatabaseAfsecComm, IdTag, IdUser,
    RawFrame, TValue,
};

#[derive(Default)]
pub struct MMenu {}

impl CommonMiddlewareTrait for MMenu {
    fn reset_conversation(&self, _context: &mut Context) {}

    fn get_conversation(
        &self,
        _context: &mut Context,
        _afsec_service: &mut DatabaseAfsecComm,
        request_data_frame: &DataFrame,
    ) -> Option<RawFrame> {
        if request_data_frame.get_tag() != id_message::AF_MENU {
            // Non concerné par cette conversation
            return None;
        }

        // Réponse
        println!("AFSEC Comm: AF_MENU NACK");
        Some(RawFrame::new_nack())
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
