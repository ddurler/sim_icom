//! Process en communication avec l'AFSEC+ via un port série

use std::sync::{Arc, Mutex};

use tokio_serial::SerialPortBuilderExt;

use crate::database::{Database, IdUser, ID_ANONYMOUS_USER};

/// Wrapper de [`Database`] pour la communication série avec l'AFSEC
pub struct DatabaseAfsecComm {
    thread_db: Arc<Mutex<Database>>,
    id_user: IdUser,
    port_name: String,
}

impl DatabaseAfsecComm {
    /// Constructeur
    pub fn new(thread_db: Arc<Mutex<Database>>, port_name: String) -> Self {
        Self {
            thread_db,
            id_user: ID_ANONYMOUS_USER, // Overwrite si le port est OK
            port_name,
        }
    }
}

/// Routine d'un thread en communication avec l'AFSEC+ via un port série.
pub async fn database_afsec_process(afsec_service: &mut DatabaseAfsecComm) {
    if afsec_service.port_name.to_uppercase() == "FAKE" {
        println!("AFSEC+ communication skipped !!!");
        return;
    }

    println!("AFSEC Communication: Starting...");

    let mut port = match tokio_serial::new(&afsec_service.port_name, 9600).open_native_async() {
        Ok(port) => port,
        Err(e) => {
            eprintln!(
                "!!! Erreur fatal ouverture du port '{}': {}",
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

    loop {
        let mut buff = [0_u8; 256];
        match port.try_read(&mut buff) {
            Ok(n) => {
                println!("AFSEC: Read {n}  bytes = '{buff:?}'");
            }
            Err(e) => {
                println!("AFSEC: Got read error: '{e}'");
            }
        }

        // Laisse la main...
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }
}
