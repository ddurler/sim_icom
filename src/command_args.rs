//! Gestion de la configuration selon les arguments de la ligne de commande

use clap::Parser;

/// Simulateur ICOM (c)ALMA - 2023
///
/// Cet outil simule le fonctionnement de la carte ICOM pour l'AFSEC+.
///
/// Le répertoire courant doit contenir un fichier 'database.csv' qui contient les informations
/// de la database de l'ICOM (fichier dont le contenu est identique au fichier database*.csv dans
/// la `µSD` de l'ICOM).
///
/// L'outil est également un serveur MODBUS/TCP pour interagir avec le contenu de la database.
#[derive(Parser)]
pub struct CommandArgs {
    /// Nom du port série pour communiquer avec l'AFSEC+
    /// ('fake' pour simuler une communication inexistante)
    pub port_name: String,

    /// Fichier descriptif de la database au format .csv
    #[arg(short, long, default_value_t = String::from("database.csv"))]
    pub filename: String,

    /// Numéro du port MODBUS/TCP
    #[arg(short, long, default_value_t = 502)]
    pub port: usize,

    /// Timer (en millisecondes) pour le watcher (0 pour inhiber le watcher)
    #[arg(short, long, default_value_t = 1000)]
    pub watcher: u64,

    /// Debug show level (0: None, 1: Some, 2 ou +: All)
    #[arg(short, long, default_value_t = 1)]
    pub debug: u8,
}

impl CommandArgs {
    /// Constructeur selon la ligne de commande
    pub fn new() -> Self {
        // Parse des arguments avec le crate `clap`
        CommandArgs::parse()
    }
}
