//! Process pour surveiller les changements dans la [`Database`] et
//! les afficher à l'écran

use std::sync::{Arc, Mutex};

use crate::Database;

/// Routine d'un thread qui trace les modifications effectuées dans la [`Database`]
/// En paramètre, le temps de cycle entre chaque trace (en millisecondes)
/// Et un booléen pour indiquer si on trace également les modifications 'anonymes'
pub async fn database_watcher_process(
    thread_db: Arc<Mutex<Database>>,
    cycle_in_msecs: u64,
    include_anonymous_changes: bool,
) {
    println!("WATCHER: Starting...");

    let id_user;
    {
        // Verrouiller la database partagée
        let mut db = thread_db.lock().unwrap();

        // Obtient un id_user pour les opérations
        id_user = db.get_id_user("Watcher", true);
    }

    loop {
        loop {
            // Verrouiller la database partagée
            let mut db = thread_db.lock().unwrap();

            // Voir s'il y a un notification d'un autre utilisateur
            if let Some(notification_change) =
                db.get_change(id_user, false, include_anonymous_changes)
            {
                match db.get_tag_from_id_tag(notification_change.id_tag) {
                    Some(tag) => {
                        println!(
                            "WATCHER: {} = {} ({})",
                            tag,
                            db.get_t_value_from_tag(id_user, tag),
                            db.get_id_user_name(notification_change.id_user),
                        );
                    }
                    None => {
                        println!(
                            "WATCHER: Got id_tag = {} with no tag ({}) ???",
                            notification_change.id_tag,
                            db.get_id_user_name(notification_change.id_user),
                        );
                    }
                }
            } else {
                break;
            }
        }
        // Laisse la main...
        tokio::time::sleep(tokio::time::Duration::from_millis(cycle_in_msecs)).await;
    }
}
