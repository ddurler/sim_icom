use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use tokio::net::TcpListener;
use tokio_modbus::server::tcp::{accept_tcp_connection, Server};

mod database;
use database::Database;

mod watcher;
use watcher::database_watcher_process;

mod afsec;
use afsec::{database_afsec_process, DatabaseAfsecComm};

mod server_modbus_tcp;
use server_modbus_tcp::DatabaseService;

/// Aide pour l'utilisateur
fn usage_help() -> &'static str {
    "
Simulateur de ICOM (c)ALMA - 2023

Cet outil simule le fonctionnement de la carte ICOM pour l'AFSEC+.

Usage:
    sim_icom <com>      Où <com> est le port série en communication avec l'AFSEC+
                        (Le port série 'fake' inhibe cette communication)

Le répertoire courant doit contenir un fichier 'database.csv' qui contient les informations
de la database de l'ICOM (fichier dont le contenu est identique au fichier database*.csv dans
la µSD de l'ICOM).

L'outil est également un serveur MODBUS/TCP pour interagir avec le contenu de la database.
    "
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Au moins un argument avec le port série
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("{}", usage_help());
        return Ok(());
    }

    let port_name = args[1].clone();

    // Initialisation de la database
    let mut db: Database = Database::from_file("database.csv");

    // Extrait un id_user pour le serveur MODBUS/TCP
    let id_user_tcp_server = db.get_id_user("Server MODBUS/TCP", false);

    // Créer la database partagée mutable
    let shared_db = Arc::new(Mutex::new(db));

    // Cloner la référence à la database partagée le `watcher`
    let db_watcher = Arc::clone(&shared_db);

    // Créer le watcher
    let handle_watcher =
        tokio::spawn(async move { database_watcher_process(db_watcher, 1000, true).await });

    // Cloner la référence à la database partagée pour la communication avec l'AFSEC+
    let db_afsec = Arc::clone(&shared_db);

    // Process communication avec l'AFSEC+ sur le port série
    let handle_afsec = tokio::spawn(async move {
        database_afsec_process(&mut DatabaseAfsecComm::new(db_afsec, port_name)).await;
    });

    // Serveur MODBUS
    // Linux n'autorise pas les ports < 1024
    #[cfg(target_os = "linux")]
    let socket_addr: SocketAddr = "0.0.0.0:1502".parse().unwrap();
    #[cfg(not(target_os = "linux"))]
    let socket_addr: SocketAddr = "0.0.0.0:502".parse().unwrap();

    println!("Starting up server on {socket_addr}");
    let listener = TcpListener::bind(socket_addr).await?;
    let server = Server::new(listener);
    let new_service = |_socket_addr| {
        let thread_db = Arc::clone(&shared_db);
        Ok(Some(DatabaseService::new(thread_db, id_user_tcp_server)))
    };
    let on_connected = |stream, socket_addr| async move {
        accept_tcp_connection(stream, socket_addr, new_service)
    };
    let on_process_error = |err| {
        eprintln!("{err}");
    };
    println!("[Note: Entrer ctrl+C pour stopper l'application]");
    server.serve(&on_connected, on_process_error).await?;

    // Attendre que les threads se terminent
    handle_watcher.await.unwrap();
    handle_afsec.await.unwrap();

    Ok(())
}
