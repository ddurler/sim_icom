//! Simulateur logiciel de l'ICOM d'une solution AFSEC+ ALMA
//!
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use tokio::net::TcpListener;
use tokio_modbus::server::tcp::{accept_tcp_connection, Server};

mod command_args;
use command_args::CommandArgs;

mod t_data;

mod database;
use database::Database;

mod watcher;
use watcher::database_watcher_process;

mod afsec;
use afsec::{database_afsec_process, DatabaseAfsecComm};

mod server_modbus_tcp;
use server_modbus_tcp::DatabaseService;

/// Point d'entrée du simulateur ICOM
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let command_args = CommandArgs::new();

    // Initialisation de la database
    let mut db: Database = Database::from_file(&command_args.filename);

    // Extrait un id_user pour le serveur MODBUS/TCP
    let id_user_tcp_server = db.get_id_user("Server MODBUS/TCP", false);

    // Niveau de debug pour les traces
    let debug_level = match command_args.debug {
        0 => 0,
        1 => {
            println!("Active DEBUG level SOME...");
            1
        }
        _ => {
            println!("Active DEBUG level ALL...");
            2
        }
    };

    // Créer la database partagée mutable
    let shared_db = Arc::new(Mutex::new(db));

    // Cloner la référence à la database partagée le `watcher`
    let db_watcher = Arc::clone(&shared_db);

    // Créer le watcher
    let handle_watcher = tokio::spawn(async move {
        database_watcher_process(db_watcher, command_args.watcher, true).await;
    });

    // Cloner la référence à la database partagée pour la communication avec l'AFSEC+
    let db_afsec = Arc::clone(&shared_db);

    // Process communication avec l'AFSEC+ sur le port série
    let port_name = command_args.port_name; // Need 'copy'
    let handle_afsec = tokio::spawn(async move {
        database_afsec_process(&mut DatabaseAfsecComm::new(
            db_afsec,
            port_name,
            debug_level,
        ))
        .await;
    });

    // Serveur MODBUS
    let socket_addr: SocketAddr = format!("0.0.0.0:{}", command_args.port).parse().unwrap();

    println!("Starting up server on {socket_addr}");
    let listener = TcpListener::bind(socket_addr).await?;
    let server = Server::new(listener);
    let new_service = |_socket_addr| {
        let thread_db = Arc::clone(&shared_db);
        Ok(Some(DatabaseService::new(
            thread_db,
            id_user_tcp_server,
            debug_level,
        )))
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
