use std::sync::{Arc, Mutex};
use std::thread;

mod database;
use database::{Database, IdTag, TValue, Tag};

fn main() {
    // Initialisation de la database
    let mut db = Database::default();

    // Populate database
    let id_tag = IdTag::default();
    let t_value = TValue::U16(0);
    let tag = Tag { t_value };

    db.push(&id_tag, &tag);

    // Créer la database partagée mutable
    let shared_db = Arc::new(Mutex::new(db));

    // Cloner la référence à la database partagée pour chaque thread
    let thread1_data = Arc::clone(&shared_db);
    let thread2_data = Arc::clone(&shared_db);

    // Créer les threads
    let thread1 = thread::spawn(move || {
        // Verrouiller la database partagée pour accéder à sa valeur
        let mut db = thread1_data.lock().unwrap();

        // Modifier une valeur (méthode via le tag)
        let id_tag = IdTag::new(0, 0, [0, 0, 0]);
        if let Some(tag) = db.get_mut(&id_tag) {
            let mut value = tag.t_value.extract_u16();
            value += 1;
            tag.t_value = TValue::U16(value);
        }
    });

    let thread2 = thread::spawn(move || {
        // Verrouiller la database partagée pour accéder à sa valeur
        let mut db = thread2_data.lock().unwrap();

        // Modifier la valeur (méthode via t_value)
        let id_tag = IdTag::new(0, 0, [0, 0, 0]);
        if let Some(t_value) = db.get_mut_t_value(&id_tag) {
            let mut value = t_value.extract_u16();
            value += 2;
            *t_value = TValue::U16(value);
        }
    });

    // Attendre que les threads se terminent
    thread1.join().unwrap();
    thread2.join().unwrap();

    // Accéder à la valeur finale de la zone de données partagée
    let db = shared_db.lock().unwrap();
    println!("Valeur finale : {db:?}");
}
