use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use tokio::net::TcpListener;
use tokio_modbus::server::tcp::{accept_tcp_connection, Server};

mod database;
use database::Database;

mod server_modbus_tcp;
use server_modbus_tcp::DatabaseService;

/// Routine d'un thread qui trace les modifications effectuées dans la [`Database`]
/// En paramètre, le temps de cycle entre chaque trace (en millisecondes)
/// Et un booléen pour indiquer si on trace également les modifications 'anonymes'
async fn database_watcher(
    thread_db: Arc<Mutex<Database>>,
    cycle_in_msecs: u64,
    include_anonymous_changes: bool,
) {
    println!("WATCHER: Starting...");

    let id_user;
    {
        // Verrouiller la database partagée pour accéder à sa valeur
        let mut db = thread_db.lock().unwrap();

        // Obtient un id_user pour les opérations
        id_user = db.get_id_user("Watcher", true);
    }

    loop {
        loop {
            // Verrouiller la database partagée pour accéder à sa valeur
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialisation de la database
    let mut db: Database = Database::from_file("./datas/database.csv");

    // Extrait un id_user pour le serveur
    let id_user = db.get_id_user("Server MODBUS/TCP", false);

    // Créer la database partagée mutable
    let shared_db = Arc::new(Mutex::new(db));

    // Cloner la référence à la database partagée pour chaque thread
    let db_watcher = Arc::clone(&shared_db);

    // Créer un watcher
    let handle_watcher =
        tokio::spawn(async move { database_watcher(db_watcher, 1000, true).await });

    // Serveur MODBUS
    let socket_addr: SocketAddr = "127.0.0.1:502".parse().unwrap();
    println!("Starting up server on {socket_addr}");
    let listener = TcpListener::bind(socket_addr).await?;
    let server = Server::new(listener);
    let new_service = |_socket_addr| {
        let thread_db = Arc::clone(&shared_db);
        Ok(Some(DatabaseService::new(thread_db, id_user)))
    };
    let on_connected = |stream, socket_addr| async move {
        accept_tcp_connection(stream, socket_addr, new_service)
    };
    let on_process_error = |err| {
        eprintln!("{err}");
    };
    server.serve(&on_connected, on_process_error).await?;

    // Attendre que les threads se terminent
    handle_watcher.await.unwrap();

    Ok(())
}
