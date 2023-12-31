//! Database de l'ICOM
//!
//! La [`Database`] est une zone de 32768 mots dont le contenu peut être accédé via une
//! [`WordAddress`] (adresse MODBUS en `u16`) ou via un [`IdTag`] (zone+tag+indices).
//!
//! En interne, la [`Database`] est un `vec<u8>` de 2 * 32736 Bytes où les données sont encodées
//! en 'big endian'.
//!
//! Chaque 'entrée' ([`WordAddress`] ou [`IdTag`]) de la [`Database`] donne accès à un [`Tag`].
//! Ce [`Tag`] porte également une valeur d'un type défini [`TFormat`] pour accéder à une valeur
//! générique [`TValue`]. Voir `Database::get_t_value_from_tag`
//!
//! Par ailleurs, on accède directement aux valeurs typées `bool`, `u8`, `u16`, ..., f32, f64 et string
//! de la [`Database`]. Voir `Database::get_bool_from_word_address`, `Database::set_bool_to_word_address`,
//! `Database::get_bool_from_id_tag` et `Database::set_bool_to_id_tag` par exemple pour un `bool`.
//! Idem pour tous les autres types supportés.
//!
//! La [`Database`] peut être créée par la lecture d'un fichier au format .csv avec la primitive
//! `Database::from_file`
//!
//! Sinon, une [`Database`] vide est créée par `Database::default` et il est nécessaire ensuite
//! de définir tous les [`Tag`] de la [`Database`] avec la primitive `Database::add_tag`
//!
//! Pour accéder en lecture ou en écriture dans la  [`Database`], il faut spécifier un [`IdUser`].
//!
//! Dans la plupart des situations, on peut utiliser `ID_ANONYMOUS_USER`.
//!
//! `ID_ANONYMOUS_USER` est un [`IdUser`] qu'il est possible d'utiliser sans demander un [`IdUser`] spécifique
//! mais l'utilisateur n'aura alors pas accès à un historique dédié de notification
//!
//! La primitive `Database::get_id_user` permet d'obtenir un nouveau [`IdUser`]
//!

use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Read;

use crate::t_data::{TFormat, TValue};

mod database_csv;

mod id_tag;
pub use id_tag::IdTag;

mod tag;
pub use tag::Tag;

mod database_rw;

mod id_users;
pub use id_users::{IdUser, IdUsers, NotificationChange, ID_ANONYMOUS_USER};

/// Adresse MODBUS pour accéder la [`Database`]
/// Il s'agit d'une valeur entière `u16`.
pub type WordAddress = u16;

/// [`Database`] de l'ICOM
#[derive(Debug)]
pub struct Database {
    /// Table `u8` de la table MODBUS
    /// Plage d'[`WordAddress`] possibles entre 0x0000 et 0x7FFF
    /// L'[`WordAddress`] (`u16`) dans cette table correspond aux 2 Bytes consécutifs à l'offset
    /// 2 * addr et 2 * addr + 1 avec un encodage 'big endian'.
    vec_u8: Vec<u8>,

    /// Correspondances [`WordAddress`] -> [`IdTag`]
    hash_word_address: HashMap<WordAddress, IdTag>,

    /// Correspondances [`IdTag`] -> [`Tag`]
    hash_tag: HashMap<IdTag, Tag>,

    /// Gestion des [`IdUsers`]
    id_users: IdUsers,
}

impl Default for Database {
    fn default() -> Self {
        Self {
            vec_u8: [0_u8; 2 * 0x8000].to_vec(),
            hash_word_address: HashMap::new(),
            hash_tag: HashMap::new(),
            id_users: IdUsers::default(),
        }
    }
}

impl fmt::Display for Database {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ret = String::new();
        // La [`Database`] est affichée par ordre croissant de [`WordAddress`]
        let mut word_addresses: Vec<WordAddress> = self.hash_word_address.keys().copied().collect();
        word_addresses.sort_unstable();
        for word_address in word_addresses {
            if let Some(tag) = self.get_tag_from_word_address(word_address) {
                let t_value = self.get_t_value_from_tag(ID_ANONYMOUS_USER, tag);
                let unity = tag.unity.clone();
                ret += &format!("{tag} = {t_value} {unity}\n");
            }
        }
        write!(f, "{ret}")
    }
}

impl Database {
    /// Construction de la [`Database`] depuis le contenu d'un fichier database*.csv
    /// (fichier .csv standard de production)
    /// Cette fonction autorise du contenu non UTF-8 dans le fichier (souvent le cas pour les unités)
    /// Les champs retenus sont ceux de la structure [`Tag`]
    /// Si une valeur par défaut est définie (non vide), la [`Database`] est initialisées avec cette valeur
    /// (si la conversion de cette valeur par défaut dans le type est possible)
    /// # panics
    /// panic! si le fichier ne peut pas être lu
    /// panic! si syntaxe incorrecte dans une ligne du fichier
    #[allow(dead_code)]
    pub fn from_file(filename: &str) -> Self {
        let mut db = Database::default();

        // Il se peut que le fichier ne contienne pas que de l'UTF-8...
        // Aussi on le 'parse' en utf8_lossy....
        let mut file = match File::open(filename) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("\nErreur ouverture du fichier '{filename}': {e}\n");
                std::process::exit(1);
            }
        };
        let mut buf = vec![];
        match file.read_to_end(&mut buf) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("\nErreur lecture du fichier '{filename}': {e}\n");
                std::process::exit(1);
            }
        };
        let contents: String = String::from_utf8_lossy(&buf).into();

        for (n, line) in contents.lines().enumerate() {
            match database_csv::from_line_csv(line) {
                Ok(option_tag) => {
                    if let Some(tag) = option_tag {
                        // Ajout du [`Tag`] dans la liste des [`Tag`] connus
                        db.add_tag(&tag);

                        // Valeur par défaut ?
                        if !tag.default_value.is_empty() {
                            db.set_value(ID_ANONYMOUS_USER, &tag, &tag.default_value);
                        }
                    }
                }
                Err(msg) => {
                    eprintln!("\nErreur fichier '{}', line {}: {}\n", filename, n + 1, msg);
                    std::process::exit(1);
                }
            }
        }

        println!("Database `{filename}` loaded OK");
        db
    }

    /// Ajoute un [`Tag`] à une [`WordAddress`] dans la [`Database`]
    /// Cette fonction n'autorise pas de définir un [`Tag`] à une [`WordAddress`] déjà affectée.
    /// Cette fonction n'autorise pas de définir un [`Tag`] avec un [`IdTag`] déjà affectée.
    /// Par contre, cette fonction ne contrôle pas le recouvrement d'[`WordAddress`] entre les
    /// différents [`Tag`] de la [`Database`] (des données qui empiètent sur d'autres [`Tag`])
    /// # panics
    /// panic! si l'[`WordAddress`] est déjà attribuée
    /// panic! si l'[`IdTag`] du [`Tag`] est déjà attribué
    pub fn add_tag(&mut self, tag: &Tag) {
        let tag = tag.clone();
        let word_address = tag.word_address;
        assert!(
            self.get_tag_from_word_address(word_address).is_none(),
            "Ajout {tag} à une adresse déjà attribuée"
        );
        assert!(
            self.get_tag_from_id_tag(tag.id_tag).is_none(),
            "Ajout {tag} avec un id_tag déjà attribué"
        );
        self.hash_word_address.insert(word_address, tag.id_tag);
        self.hash_tag.insert(tag.id_tag, tag);
    }

    /// Extrait un [`Tag`] (non mutable) de la [`Database`] selon son [`IdTag`]
    #[allow(dead_code)]
    pub fn get_tag_from_id_tag(&self, id_tag: IdTag) -> Option<&Tag> {
        self.hash_tag.get(&id_tag)
    }

    /// Extrait un [`Tag`] (non mutable) de la [`Database`] selon [`WordAddress`]
    #[allow(dead_code)]
    pub fn get_tag_from_word_address(&self, word_address: WordAddress) -> Option<&Tag> {
        let option_id_tag = self.hash_word_address.get(&word_address);
        match option_id_tag {
            Some(id_tag) => self.hash_tag.get(id_tag),
            None => None,
        }
    }

    /// Extrait la liste des [`Tag`] (non mutable) de la [`Database`] selon son [`WordAddress`] et le
    /// nombre de mots à partir de cette [`WordAddress`] dans la [`Database`]
    #[allow(dead_code)]
    #[allow(while_true)]
    pub fn get_tags_from_word_address_area(
        &self,
        word_address: WordAddress,
        nb_words: usize,
    ) -> Vec<Tag> {
        let mut ret_tags = vec![];

        // Recherche le premier tag dans l'espace d'adresses
        let mut previous_word_address = word_address;
        if let Some(tag) = self.get_tag_from_word_address(word_address) {
            // L'adresse spécifiée correspond avec un tag défini
            ret_tags.push(tag.clone());
        } else {
            // Sinon recherche en remontant dans les [`WordAddress`]...
            while true {
                if previous_word_address == 0 {
                    // Pas de tag trouvé en amont du word_address spécifié
                    return vec![];
                }
                previous_word_address -= 1;
                if let Some(tag) = self.get_tag_from_word_address(previous_word_address) {
                    // Un tag trouvé en amont du word_address spécifié
                    if tag.contains_word_address_area(word_address, nb_words) {
                        ret_tags.push(tag.clone());
                        break;
                    }
                    // Ce de tag trouvé en très en amont du word_address/nb_words annoncé
                    return vec![];
                }
            }
        }

        // Ici, ret_tags contient tag en qui empiète sur la zone à partir de word_address
        // On va inclure également tous les tags suivants qui empiètent...
        let mut forward_word_address = word_address;
        while true {
            #[allow(clippy::cast_possible_truncation)]
            if forward_word_address > word_address + nb_words as u16 {
                // On est en dehors de la zone spécifiée
                break;
            }
            forward_word_address += 1;
            if let Some(tag) = self.get_tag_from_word_address(forward_word_address) {
                if tag.contains_word_address_area(word_address, nb_words) {
                    // Tag suivant qui est également dans la zone spécifiée
                    ret_tags.push(tag.clone());
                } else {
                    break;
                }
            }
        }

        ret_tags
    }

    /// Extrait un [`Tag`] mutable de la [`Database`] selon son [`IdTag`]
    #[allow(dead_code)]
    pub fn get_mut_tag_from_id_tag(&mut self, id_tag: IdTag) -> Option<&mut Tag> {
        self.hash_tag.get_mut(&id_tag)
    }

    /// Extrait un [`Tag`] mutable de la [`Database`] selon [`WordAddress`]
    #[allow(dead_code)]
    pub fn get_mut_tag_from_word_address(&mut self, word_address: WordAddress) -> Option<&mut Tag> {
        let option_id_tag = self.hash_word_address.get(&word_address);
        match option_id_tag {
            Some(id_tag) => self.hash_tag.get_mut(id_tag),
            None => None,
        }
    }
}
