//! Database de l'ICOM
//!
//! La [`Database`] est une zone de 32768 mots dont le contenu peut être accédé via une
//! [`WordAddress`] (adresse MODBUS en `u16`) ou via un [`IdTag`] (zone+tag+indices).
//!
//! En interne, la [`Database`] est un `vec<u8>` de 2 * 32736 Bytes où les données sont encodées
//! en 'big endian'.

use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Read;

mod t_format;
pub use t_format::TFormat;

mod t_value;
pub use t_value::TValue;

mod database_csv;

mod id_tag;
pub use id_tag::IdTag;

mod tag;
pub use tag::Tag;

mod database_rw;

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
}

impl Default for Database {
    fn default() -> Self {
        Self {
            vec_u8: [0_u8; 2 * 0x8000].to_vec(),
            hash_word_address: HashMap::new(),
            hash_tag: HashMap::new(),
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
                let t_value = self.get_t_value_from_tag(tag);
                let unity = tag.unity.clone();
                ret += &format!("{tag} = {t_value} {unity}\n");
            }
        }
        write!(f, "{ret}")
    }
}

impl Database {
    /// Construction de la [`Database`] depuis le contenu d'un fichier datafile*.csv
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
                eprintln!("\nErreur ouverture du fichier '{filename}' : {e}\n");
                std::process::exit(1);
            }
        };
        let mut buf = vec![];
        match file.read_to_end(&mut buf) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("\nErreur lecture du fichier '{filename}' : {e}\n");
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
                            db.set_value(&tag, &tag.default_value);
                        }
                    }
                }
                Err(msg) => {
                    eprintln!(
                        "\nErreur fichier '{}', line {} : {}\n",
                        filename,
                        n + 1,
                        msg
                    );
                    std::process::exit(1);
                }
            }
        }

        db
    }

    /// Ajoute un [`Tag`] à une [`WordAddress`] dans la [`Database`]
    /// Cette fonction n'autorise pas de définir un [`Tag`] à une [`WordAddress`] déjà affectée.
    /// Par contre, cette fonction ne contrôle pas le recouvrement d'[`WordAddress`] entre les
    /// différents [`Tag`] de la [`Database`] (des données qui empiètent sur d'autres [`Tag`])
    /// # panics
    /// panic! si l'[`WordAddress`] est déjà attribuée
    pub fn add_tag(&mut self, tag: &Tag) {
        let tag = tag.clone();
        let word_address = tag.word_address;
        assert!(
            self.get_tag_from_word_address(word_address).is_none(),
            "Ajout {tag} à une adresse déjà attribuée"
        );
        let id_tag = tag.id_tag.clone();
        self.hash_word_address.insert(word_address, id_tag.clone());
        self.hash_tag.insert(id_tag.clone(), tag);
    }

    /// Extrait un [`Tag`] (non mutable) de la [`Database`] selon son [`IdTag`]
    #[allow(dead_code)]
    pub fn get_tag_from_id_tag(&self, id_tag: &IdTag) -> Option<&Tag> {
        self.hash_tag.get(id_tag)
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

    /// Extrait un [`Tag`] (non mutable) possible de la [`Database`] selon son [`WordAddress`]
    /// Cette fonction retour le même [`Tag`] de `get_tag_from_address` si [`WordAddress`] correspond avec
    /// un [`Tag`] existant.
    /// Sinon, la fonction retourne le [`Tag`] dont [`WordAddress`] et sa longueur 'pourrait' correspondre
    /// Utiliser la fonction `Tag::contains_word_address` pour contrôler ensuite si le [`Tag`] retourné
    /// utilise effectivement [`WordAddress`] soumise
    /// # panics
    /// Cette fonction panic! si la [`Database`] est 'vide'
    /// Cette fonction panic! si [`WordAddress`] soumis est en amont du 1er [`Tag`] de la [`Database`]
    #[allow(dead_code)]
    #[allow(while_true)]
    pub fn get_tag_from_word_address_unstable(&self, word_address: WordAddress) -> &Tag {
        // Retourne un tag si correspondance direct
        if let Some(id_tag) = self.get_tag_from_word_address(word_address) {
            return id_tag;
        }
        // panic! si la [`Database`] est vide
        assert!(
            !self.hash_word_address.is_empty(),
            "La database ets vide (aucun tag défini !)"
        );
        // Sinon recherche en remontant dans les [`WordAddress`]...
        let mut previous_word_address = word_address;
        while true {
            assert!(
                previous_word_address != 0,
                "Aucun tag avant {word_address:04X} dans la database !)"
            );
            previous_word_address -= 1;
            if let Some(id_tag) = self.get_tag_from_word_address(previous_word_address) {
                return id_tag;
            }
        }
        unreachable!()
    }

    /// Extrait un [`Tag`] mutable de la [`Database`] selon son [`IdTag`]
    #[allow(dead_code)]
    pub fn get_mut_tag_from_id_tag(&mut self, id_tag: &IdTag) -> Option<&mut Tag> {
        self.hash_tag.get_mut(id_tag)
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
