//! Liste des constantes pour les types de messages TLV entre l'AFSEC+ et l'ICOM et
//! les types de données dans les messages

#![allow(dead_code)]

// Codage des types de messages AFSEC+ (préfixe 'AF') et ICOM (préfixe 'IC')

pub const AF_ALIVE: u8 = 0x00;
pub const IC_ALIVE: u8 = 0x80;

pub const AF_INIT: u8 = 0x01;
pub const IC_INIT: u8 = 0x81;

pub const AF_MENU: u8 = 0x02;
pub const IC_MENU: u8 = 0x82;

pub const AF_DATA_OUT: u8 = 0x03;
pub const IC_DATA_OUT: u8 = 0x83;

pub const AF_DATA_IN: u8 = 0x04;
pub const IC_DATA_IN: u8 = 0x84;

pub const AF_DATA_OUT_TABLE_INDEX: u8 = 0x05;
pub const IC_DATA_OUT_TABLE_INDEX: u8 = 0x85;

pub const AF_DOWNLOAD: u8 = 0x06;
pub const IC_DOWNLOAD: u8 = 0x86;

pub const AF_TEST: u8 = 0x7F;
pub const IC_TEST: u8 = 0xFF;

pub const AF_PACK_OUT: u8 = 0x0B;
pub const IC_PACK_OUT: u8 = 0x8B;

pub const AF_PACK_IN: u8 = 0x0C;
pub const IC_PACK_IN: u8 = 0x8C;

// Codage des types de données dans les messages

pub const D_PROTOCOLE_VERSION: u8 = 0x01;
pub const D_ICOM_VERSION: u8 = 0x02;
pub const D_RESIDENT_VERSION: u8 = 0x03;
pub const D_APPLI_NUMBER: u8 = 0x04;
pub const D_APPLI_VERSION: u8 = 0x05;
pub const D_APPLI_CONFIG: u8 = 0x06;
pub const D_MODE_AFSEC: u8 = 0x07;
pub const D_LANGUAGE: u8 = 0x08;

pub const D_MENU_ID: u8 = 0x10;
pub const D_MENU_ID_IN_PROGRESS: u8 = 0x11;
pub const D_MENU_SHORT_DISPLAY: u8 = 0x12;
pub const D_MENU_LONG_DISPLAY: u8 = 0x13;
pub const D_MENU_PICTOS: u8 = 0x14;
pub const D_MENU_ID_ON_BP_OK: u8 = 0x15;
pub const D_MENU_ID_ON_BP_MENU: u8 = 0x16;
pub const D_MENU_ID_ON_BP_CLEAR: u8 = 0x17;
pub const D_MENU_VALUE_INIT: u8 = 0x18;
pub const D_MENU_CHOICE_LIST: u8 = 0x19;
pub const D_MENU_INPUT_MASK: u8 = 0x1A;
pub const D_MENU_USER_INPUT: u8 = 0x1B;

pub const D_DATA_ERROR: u8 = 0x30;
pub const D_DATA_ZONE: u8 = 0x31;
pub const D_DATA_TABLE_INDEX: u8 = 0x32;
pub const D_DATA_TAG: u8 = 0x33;
pub const D_DATA_VALUE: u8 = 0x35;
pub const D_DATA_FIRST_TABLE_INDEX: u8 = 0x50;
pub const D_DATA_LAST_TABLE_INDEX: u8 = 0x51;

pub const D_DOWNLOAD_SECTION: u8 = 0x60;
pub const D_DOWNLOAD_NAME: u8 = 0x61;
pub const D_DOWNLOAD_NB_RECORDS: u8 = 0x62;
pub const D_DOWNLOAD_STATUS: u8 = 0x63;
pub const D_DOWNLOAD_RECORD: u8 = 0x64;
pub const D_DOWNLOAD_END: u8 = 0x65;

pub const D_TEST_NB_REQS: u8 = 0x71;
pub const D_TEST_NB_REPS: u8 = 0x72;

pub const D_PACK_PAYLOAD: u8 = 0xB0;
