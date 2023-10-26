//! Process en communication avec l'AFSEC+ via un port série

use std::sync::{Arc, Mutex};
use tokio::time::Instant;

use tokio_serial::{SerialPortBuilderExt, SerialStream};

use crate::database::{Database, IdUser, ID_ANONYMOUS_USER};

mod tlv_frame;
use tlv_frame::{DataFrame, FrameState, RawFrame};

mod middleware;
pub use middleware::Middlewares;

/// Temporisation entre chaque surveillance pour les `notification_changes`
const DURATION_NOTIFICATION_CHANGES_SECS: f32 = 1.0;

/// Niveau debug Some
pub const DEBUG_LEVEL_SOME: u8 = 1;

/// Niveau debug All
pub const DEBUG_LEVEL_ALL: u8 = 2;

/// Wrapper de [`Database`] pour la communication série avec l'AFSEC
pub struct DatabaseAfsecComm {
    /// Mutex pour l'accès à la base de données
    thread_db: Arc<Mutex<Database>>,

    /// [`IdUser`] attribué au thread en communication avec l'AFSEC+
    id_user: IdUser,

    /// Nom du port série choisi par l'utilisateur pour communiquer avec l'AFSEC+
    port_name: String,

    /// Niveau de debug pour les affichages (0: None, 1: Some, 2: All)
    debug_level: u8,
}

impl DatabaseAfsecComm {
    /// Constructeur
    pub fn new(thread_db: Arc<Mutex<Database>>, port_name: String, debug_level: u8) -> Self {
        Self {
            thread_db,
            id_user: ID_ANONYMOUS_USER, // Overwrite si le port est OK
            port_name,
            debug_level,
        }
    }
}

/// Routine d'un thread en communication avec l'AFSEC+ via un port série.
pub async fn database_afsec_process(afsec_service: &mut DatabaseAfsecComm) {
    if afsec_service.port_name.to_uppercase() == "FAKE" {
        println!("AFSEC communication skipped (fake usage) !!!");
        return;
    }

    println!("AFSEC Comm: Starting on '{}'...", afsec_service.port_name);

    let mut port = match tokio_serial::new(&afsec_service.port_name, 115_200).open_native_async() {
        Ok(port) => port,
        Err(e) => {
            eprintln!(
                "!!! Erreur fatale ouverture du port '{}': {}",
                afsec_service.port_name, e
            );
            std::process::exit(1);
        }
    };

    {
        // Verrouiller la database partagée
        let mut db = afsec_service.thread_db.lock().unwrap();

        // Obtient un id_user pour les opérations
        afsec_service.id_user = db.get_id_user("AFSEC Comm", true);
    }

    // Création du gestionnaire des `middlewares` pour les conversations avec l'AFSEC+
    let mut middlewares = Middlewares::new(afsec_service.debug_level);

    // Timer pour surveiller les notifications
    let mut date_last_notification_changes = Instant::now();

    loop {
        // Gestion communication AFSEC+ sur le port
        let tempo = read_and_write(&mut port, afsec_service, &mut middlewares);

        // Laisse la main...
        tokio::time::sleep(tokio::time::Duration::from_millis(tempo)).await;

        let current_date = Instant::now();
        let duration = current_date.duration_since(date_last_notification_changes);
        if duration.as_secs_f32() > DURATION_NOTIFICATION_CHANGES_SECS {
            date_last_notification_changes = current_date;
            // Gestion des notification_changes pour les `middlewares`
            check_notification_changes(afsec_service, &mut middlewares);
        }

        // Laisse la main encore un peu...
        // tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
}

/// Gestion communication avec l'AFSEC+ sur le port
/// Retourne une temporisation en millisecondes avant de tenter à nouveau un cycle
/// de gestion de la communication avec l'AFSEC+
fn read_and_write(
    port: &mut SerialStream,
    afsec_service: &mut DatabaseAfsecComm,
    middlewares: &mut Middlewares,
) -> u64 {
    let mut request_raw_frame = RawFrame::default();
    let mut buff = [0_u8; 256];

    loop {
        // Tentative de lecture (retour n octets lus)
        let n = match port.try_read(&mut buff) {
            Ok(n) => {
                // println!("AFSEC Comm: Read {}  bytes = '{:?}'", n, &buff[..n]);
                n
            }
            Err(_e) => {
                // println!("AFSEC Comm Got read error: '{e}'");
                0
            }
        };

        if n > 0 {
            request_raw_frame.extend(&buff[..n]);
            match request_raw_frame.get_state() {
                // Ne doit pas arriver...
                FrameState::Empty => {
                    break 1;
                }

                // Trame en cours mais pas encore complète, on continue à lire sur le port
                FrameState::Building => (),

                // Reçu un message inexploitable... On zappe
                FrameState::Junk => {
                    if afsec_service.debug_level >= DEBUG_LEVEL_ALL {
                        println!("AFSEC Comm: Got junk frame '{request_raw_frame}'");
                    }
                    break 1;
                }

                // Trame correcte reçue. On traite pour répondre...
                FrameState::Ok => {
                    if afsec_service.debug_level >= DEBUG_LEVEL_ALL {
                        println!("AFSEC Comm: -> REQ {request_raw_frame}");
                    }
                    let response_raw_frame =
                        middlewares.handle_request_raw_frame(afsec_service, request_raw_frame);
                    match port.try_write(&response_raw_frame.encode()) {
                        Ok(_n) => {
                            if afsec_service.debug_level >= DEBUG_LEVEL_ALL {
                                println!("AFSEC Comm: <- REP {response_raw_frame}");
                            }
                        }
                        Err(e) => {
                            if afsec_service.debug_level >= DEBUG_LEVEL_SOME {
                                println!("AFSEC Comm: Got error while writing: {e}");
                            }
                        }
                    }
                    break 1;
                }
            }
        } else {
            // Aucune donnée reçue
            break 1;
        }
    }
}

/// Surveillances des `notification_changes` dans la `database` pour informer les `middlewares`
/// (public car utilisé pour les tests...)
pub fn check_notification_changes(
    afsec_service: &mut DatabaseAfsecComm,
    middlewares: &mut Middlewares,
) {
    // On créée une liste des notification_changes à signaler après avoir tout récupéré
    let mut vec_changes = vec![];

    loop {
        // Verrouiller la database partagée
        let mut db = afsec_service.thread_db.lock().unwrap();

        // Voir s'il y a une notification d'un autre utilisateur
        if let Some(notification_change) = db.get_change(afsec_service.id_user, false, true) {
            if let Some(tag) = db.get_tag_from_id_tag(notification_change.id_tag) {
                let id_user = notification_change.id_user;
                let id_tag = notification_change.id_tag;
                let t_value = db.get_t_value_from_tag(id_user, tag);

                vec_changes.push((id_user, id_tag, t_value));
            }
        } else {
            break;
        }
    }

    // Informe les `middlewares`
    for (id_user, id_tag, t_value) in vec_changes {
        middlewares.notification_change(afsec_service, id_user, id_tag, &t_value);
    }
}
