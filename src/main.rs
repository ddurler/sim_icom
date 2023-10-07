use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use tokio::net::TcpListener;
use tokio_modbus::server::tcp::{accept_tcp_connection, Server};

mod database;
use database::{Database, IdTag};

mod server_modbus_tcp;
use server_modbus_tcp::DatabaseService;

async fn my_test_process(thread_db: Arc<Mutex<Database>>, option_id_tag: Option<IdTag>) {
    let id_user;
    {
        // Verrouiller la database partagée pour accéder à sa valeur
        let mut db = thread_db.lock().unwrap();

        // Obtient un id_user pour les opérations
        id_user = db.get_id_user();
    }

    // Laisse la main...
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    {
        // Verrouiller la database partagée pour accéder à sa valeur
        let mut db = thread_db.lock().unwrap();

        if let Some(id_tag) = option_id_tag {
            // Modifier une valeur (méthode via l'id_tag)
            let value = db.get_u8_from_id_tag(id_user, id_tag);
            db.set_u8_to_word_address(id_user, 0, value + 20);
        } else {
            // Modifier une valeur (méthode via l'adresse)
            let value = db.get_u8_from_word_address(id_user, 0);
            db.set_u8_to_word_address(id_user, 0, value + 10);
        }
    }

    for _ in 0..300 {
        /* Pendant 5 minutes... */
        // Laisse la main...
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        // Verrouiller la database partagée pour accéder à sa valeur
        let mut db = thread_db.lock().unwrap();

        // Voir s'il y a un notification d'un autre utilisateur
        if let Some(tag) = db.get_change(id_user, false, false) {
            println!("User #{id_user} notifié du changement de {tag}");
        } else {
            println!("Pas de changement...");
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialisation de la database
    let mut db: Database = Database::from_file("./datas/database.csv");

    // Obtient un id_user pour les opérations
    let id_user = db.get_id_user();

    // Choisi l'id_tag d'un tag à une adresse MODBUS
    let id_tag = db.get_tag_from_word_address(0x0).unwrap().id_tag;

    println!(
        "Valeur initiale : {}",
        db.get_u8_from_word_address(id_user, 0)
    );

    // Créer la database partagée mutable
    let shared_db = Arc::new(Mutex::new(db));

    // Cloner la référence à la database partagée pour chaque thread
    let thread1_data = Arc::clone(&shared_db);
    let thread2_data = Arc::clone(&shared_db);

    // Créer les threads
    let handle_1 = tokio::spawn(async move { my_test_process(thread1_data, None).await });
    let handle_2 = tokio::spawn(async move { my_test_process(thread2_data, Some(id_tag)).await });

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
    // Ok(())

    // Attendre que les threads se terminent
    handle_1.await.unwrap();
    handle_2.await.unwrap();

    // Accéder à la valeur finale de la zone de données partagée
    let db = shared_db.lock().unwrap();
    // println!("db = {db}");
    println!(
        "Valeur finale : {}",
        db.get_u8_from_word_address(id_user, 0)
    );

    Ok(())
}
