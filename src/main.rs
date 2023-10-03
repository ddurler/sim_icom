use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

mod database;
use database::Database;

fn main() {
    // Initialisation de la database
    // let db = Database::from_file("./datas/database_test.csv");
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
    let thread1 = thread::spawn(move || {
        let id_user;
        {
            // Verrouiller la database partagée pour accéder à sa valeur
            let mut db = thread1_data.lock().unwrap();

            // Obtient un id_user pour les opérations
            id_user = db.get_id_user();
        }

        std::thread::sleep(Duration::from_millis(100));

        {
            // Verrouiller la database partagée pour accéder à sa valeur
            let mut db = thread1_data.lock().unwrap();

            // Modifier une valeur (méthode via l'id_tag)
            let value = db.get_u8_from_word_address(id_user, 0);
            db.set_u8_to_word_address(id_user, 0, value + 10);
        }

        std::thread::sleep(Duration::from_millis(100));

        {
            // Verrouiller la database partagée pour accéder à sa valeur
            let mut db = thread1_data.lock().unwrap();

            // Voir s'il y a un notification d'un autre utilisateur
            if let Some(tag) = db.get_change(id_user, false, false) {
                println!("User #{id_user} notifié du changement de {tag}");
            }
        }

    });

    let thread2 = thread::spawn(move || {
        let id_user;
        {
            // Verrouiller la database partagée pour accéder à sa valeur
            let mut db = thread2_data.lock().unwrap();

            // Obtient un id_user pour les opérations
            id_user = db.get_id_user();
        }

        std::thread::sleep(Duration::from_millis(100));

        {
            // Verrouiller la database partagée pour accéder à sa valeur
            let mut db = thread2_data.lock().unwrap();

            // Modifier une valeur (méthode via l'id_tag)
            let value = db.get_u8_from_id_tag(id_user, id_tag);
            db.set_u8_to_word_address(id_user, 0, value + 20);
        }

        std::thread::sleep(Duration::from_millis(100));

        {
            // Verrouiller la database partagée pour accéder à sa valeur
            let mut db = thread2_data.lock().unwrap();

            // Voir s'il y a un notification d'un autre utilisateur
            if let Some(tag) = db.get_change(id_user, false, false) {
                println!("User #{id_user} notifié du changement de {tag}");
            }
        }
    });

    // Attendre que les threads se terminent
    thread1.join().unwrap();
    thread2.join().unwrap();

    // Accéder à la valeur finale de la zone de données partagée
    let db = shared_db.lock().unwrap();
    // println!("db = {db}");
    println!(
        "Valeur finale : {}",
        db.get_u8_from_word_address(id_user, 0)
    );
}
