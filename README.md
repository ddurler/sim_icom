# SIM_ICOM

Cet outil simule le fonctionnement de la carte ICOM pour l'AFSEC+.

Le répertoire courant doit contenir un fichier 'database.csv' qui contient les informations de la database de l'ICOM (fichier dont le contenu est identique au fichier database*.csv dans la `µSD` de l'ICOM).

L'outil est également un serveur MODBUS/TCP pour interagir avec le contenu de la database.

```
Usage: sim_icom.exe [OPTIONS] <PORT_NAME>

Arguments:
  <PORT_NAME>
          Nom du port série pour communiquer avec l'AFSEC+ ('fake' pour simuler une communication inexistante)

Options:
  -f, --filename <FILENAME>
          Fichier descriptif de la database au format .csv

          [default: database.csv]

  -p, --port <PORT>
          Numéro du port MODBUS/TCP

          [default: 502]

  -w, --watcher <WATCHER>
          Timer (en millisecondes) pour le watcher (0 pour inhiber le watcher)

          [default: 1000]

  -d, --debug <DEBUG>
          Debug show level (0: None, 1: Some, 2 ou +: All)

          [default: 1]

  -h, --help
          Print help (see a summary with '-h')
```

## Fonctionnalités

Le simulateur crée une `database` en mémoire de l'ensemble du mapping 0x0000-0x7FFF pour les adresses 'mot' et référence les tags définis dans le fichier local `database.csv` (même format que le fichier 'database' à copier sur la µSD de l'ICOM).

3 threads sont démarrés ensuite :

* **Serveur MODDBUS/TCP** répond aux différentes requêtes de lecture/écriture de mots dans la 'database'
* **Watcher** trace chaque seconde les modifications de la 'database' (tag, valeur, user)
* **Afsec** répond aux requêtes TLV reçues sur la canal série selon le protocole de la ST DEV 006

## Non implémenté

* Gestion des tags RFID
* Gestion des journaux (enregistrement des résultats de mesurage et des événements)
* Gestion des menus
* Gestion des 'téléchargements' de fichier vers l'AFSEC+

## Éléments techniques

Outil développé en [Rust](https://www.google.com/search?client=firefox-b-d&q=rust+language) v1.73.0 avec [`tokio`](https://tokio.rs/), `tokio-modbus` et `tokio-serial`.

Commandes pour le développement (sous Windows ou Linux (et macOS non)) :

* `cargo run fake` : Compilation et exécution d'une version de développement de l'outil
* `cargo clippy --tests -- -W clippy::pedantic` : Analyse statique du code
* `cargo test` : Exécution de tous les tests unitaires
* `cargo doc --open --no-deps` : Compilation et affichage de la documentation du logiciel
* `cargo build --release` : Génération de l'exécutable pour production
