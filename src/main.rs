use std::sync::{Arc, Mutex};
use std::thread;

mod database;
use database::Database;

fn main() {
    // Créer une zone de données partagée mutable
    let shared_data = Arc::new(Mutex::new(Database::default()));

    // Cloner la référence à la zone de données partagée pour chaque thread
    let thread1_data = Arc::clone(&shared_data);
    let thread2_data = Arc::clone(&shared_data);

    // Créer les threads
    let thread1 = thread::spawn(move || {
        // Verrouiller la zone de données partagée pour accéder à sa valeur
        let mut db = thread1_data.lock().unwrap();

        // Modifier la valeur
        db.counter += 1;
    });

    let thread2 = thread::spawn(move || {
        // Verrouiller la zone de données partagée pour accéder à sa valeur
        let mut db = thread2_data.lock().unwrap();

        // Modifier la valeur
        db.counter += 2;
    });

    // Attendre que les threads se terminent
    thread1.join().unwrap();
    thread2.join().unwrap();

    // Accéder à la valeur finale de la zone de données partagée
    let db = shared_data.lock().unwrap();
    println!("Valeur finale : {db:?}");
}
