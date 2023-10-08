//! Gestion de l'historique des modifications de la [`Database`] pour les différents [`IdUser`]
//!
//! Le système de notification mis en place ici n'est pas un modèle 'observateur' avec un callback
//! vers une fonction de l'utilisateur : Pas facile à mettre en place en Rust via les `threads`.
//!
//! Ici, l'utilisateur doit 'poller' pour s'enquérir des dernières modifications dans la [`Database`].

use std::time::SystemTime;

use super::IdTag;

#[cfg(test)]
use super::TFormat;

use super::{Database, Tag};

/// Identificateur d'un utilisateur de la [`Database`]
/// Il s'agit d'un numéro pour discriminer les utilisateurs et de proposer un historique dédié.
pub type IdUser = usize;

/// Utilisateur par défaut
/// L'`[``IdUser``] = 0` est un utilisateur anonyme
pub const ID_ANONYMOUS_USER: IdUser = 0; // Doit être le 0 dans la table des utilisateurs

/// Nom d'un utilisateur non identifié
const ANONYMOUS_USER_NAME: &str = "Anonymous user";

/// Durée pendant laquelle on filtre les modifications qui semblent identiques
const DURATION_CHANGE_FILTER_SECS: f32 = 1.0;

/// Structure pour mémoriser les informations d'un utilisateur
#[derive(Debug, Default)]
pub struct User {
    /// Nom de l'utilisateur
    name: String,

    /// Booléen à true si utilisateur intéressé par le système de notification
    use_notification: bool,

    /// Premier index dans `vec_changes` qui n'a pas été notifié à cet utilisateur
    next_notification_index: usize,
}

/// Structure pour mémoriser un changement dans la database
#[derive(Clone, Debug, Default)]
pub struct NotificationChange {
    /// Utilisateur qui a réalisé le changement
    pub id_user: IdUser,

    /// [`IdTag`] modifié
    pub id_tag: IdTag,
}

/// Structure pour les suivis des différents [`IdUser`] identifiés
#[derive(Debug)]
pub struct IdUsers {
    /// Liste des utilisateurs identifiés
    vec_users: Vec<User>,

    /// Historique des modifications de la [`Database`]
    vec_changes: Vec<NotificationChange>,

    // Si la modification est faite en 'découpant' l'écriture dans un même [`Tag`] (ce qui arrive lorsque
    // un client MODBUS écrit des `u16` consécutifs) alors autant de notification sont enregistrées.
    // Pour éviter de notifier plusieurs fois de la modification d'un même [`Tag`], on mémorise ici
    // la date et le contenu de la dernière notification et on n'enregistre rien si c'est la même
    // chose dans un même instant
    /// Date de la dernière notification
    date_last_change: SystemTime,
}

impl Default for IdUsers {
    fn default() -> Self {
        // L'utilisateur anonyme est en 0
        let anonymous_user = User {
            name: ANONYMOUS_USER_NAME.to_string(),
            use_notification: false,
            next_notification_index: 0,
        };
        let vec_users = vec![anonymous_user];
        Self {
            vec_users,
            vec_changes: vec![],
            date_last_change: SystemTime::now(),
        }
    }
}

impl IdUsers {
    /// Retourne un nouveau [`IdUser`]
    /// Un utilisateur s'identifie avec un nom et indique s'il souhaite pouvoir être notifié
    /// des changements dans la database par `get_change`
    pub fn get_id_user(&mut self, name: &str, use_notification: bool) -> IdUser {
        let new_id_user = self.vec_users.len();
        let next_notification_index = self.vec_changes.len();
        let new_user = User {
            name: name.to_string(),
            use_notification,
            next_notification_index,
        };
        self.vec_users.push(new_user);
        new_id_user
    }

    /// Retourne le nom d'un [`IdUser`]
    pub fn get_id_user_name(&self, id_user: IdUser) -> Option<String> {
        if id_user <= self.vec_users.len() {
            Some(self.vec_users[id_user].name.clone())
        } else {
            None
        }
    }

    /// Purge les nb premiers changements dans l'historique des changements
    fn do_purge_changes(&mut self, nb: usize) {
        // Supprime les nb premiers éléments de vec_changes
        for _ in 0..nb {
            self.vec_changes.remove(0);
        }

        // Remet à jour le `next_notification_index` pour tous les utilisateurs
        for user in &mut self.vec_users {
            user.next_notification_index = if user.next_notification_index >= nb {
                user.next_notification_index - nb
            } else {
                0
            };
        }
    }

    /// Purge l'historique des changements lorsque tous les utilisateurs ont été notifiés
    fn purge_changes(&mut self) {
        if self.vec_changes.is_empty() {
            return;
        }

        // Recherche l'index minimum qui reste à notifier
        let mut min_changes_index = self.vec_changes.len();
        for user in &self.vec_users {
            if user.use_notification && user.next_notification_index < min_changes_index {
                min_changes_index = user.next_notification_index;
            }
        }

        if min_changes_index > 0 {
            // Ici, on peut supprimer de l'historique tous les changements déjà notifier aux intéressés
            self.do_purge_changes(min_changes_index);
        }
    }

    /// Indique si au moins un utilisateur utilise le système de notification
    fn is_some_users_use_notification(&self) -> bool {
        for user in &self.vec_users {
            if user.use_notification {
                return true;
            }
        }
        false
    }

    /// Indique si le changement annoncé est le même que celui qui vient d'être enregistré
    /// C'est la temporisation de filtrage `DURATION_CHANGE_FILTER_SECS` entre 2 changements
    /// consécutifs qui filtre les changements
    fn is_same_as_last_change(&self, notification_change: &NotificationChange) -> bool {
        if self.vec_changes.is_empty() {
            return false;
        }
        let last_notification = &self.vec_changes[self.vec_changes.len() - 1];
        if last_notification.id_user == notification_change.id_user
            && last_notification.id_tag == notification_change.id_tag
        {
            let current_date = SystemTime::now();
            if let Ok(elapsed) = current_date.duration_since(self.date_last_change) {
                if elapsed.as_secs_f32() < DURATION_CHANGE_FILTER_SECS {
                    return true;
                }
            }
        }
        false
    }

    /// Enregistre un nouveau changement
    /// Rien n'est enregistré si la modification est identique à la précédente dans une temporisation
    /// de filtrage ou si aucun utilisateur n'est intéressé par un historique
    pub fn add_change(&mut self, notification_change: &NotificationChange) {
        if !self.is_same_as_last_change(notification_change)
            && self.is_some_users_use_notification()
        {
            // Enregistrement du changement
            self.vec_changes.push(notification_change.clone());
            self.date_last_change = SystemTime::now();

            // On en profite pour purger la table des changements déjà notifiés
            self.purge_changes();
        }
    }

    /// Indique s'il y a une notification à faire pour un utilisateur
    /// Possibilité de filtrer les modifications des utilisateurs anonymes ou les modifications
    /// faite par l'utilisateur demandeur
    pub fn get_change(
        &mut self,
        id_user: IdUser,
        include_my_changes: bool,
        include_anonymous_changes: bool,
    ) -> Option<NotificationChange> {
        if id_user >= self.vec_users.len() {
            return None; // Utilisateur non identifié
        }

        if !self.vec_users[id_user].use_notification {
            return None; // Utilisateur qui a indiqué ne pas vouloir utiliser cette fonction
        }

        // Dernier offset non notifié à cet utilisateur
        let offset = self.vec_users[id_user].next_notification_index;

        // Parcours des offsets de l'historique
        let mut notification_offset = offset;
        while self.vec_changes.len() > notification_offset {
            let notification = &self.vec_changes[notification_offset];
            // A notifier ?
            if (include_anonymous_changes || notification.id_user != ID_ANONYMOUS_USER)
                && (include_my_changes || notification.id_user != id_user)
            {
                // Mémorisation du dernier offset non notifié à cet utilisateur
                self.vec_users[id_user].next_notification_index = notification_offset + 1;
                // Modification de la database à retourner au demandeur
                return Some(notification.clone());
            }
            notification_offset += 1;
        }

        // Rien à notifier au demandeur
        if notification_offset != offset {
            // Mémorisation du dernier offset non notifié à cet utilisateur
            self.vec_users[id_user].next_notification_index = notification_offset;
        }

        None
    }
}

impl Database {
    /// Retourne un [`IdUser`] pour les opérations de la [`Database`]
    /// L'utilisateur doit donner un 'nom' et indiquer s'il est intéressé par le système de notification
    pub fn get_id_user(&mut self, name: &str, use_notification: bool) -> IdUser {
        self.id_users.get_id_user(name, use_notification)
    }

    /// Retourne le nom d'un [`IdUser`].
    /// Si [`IdUser`] n'est pas identifié, retourne `ANONYMOUS_USER_NAME`
    pub fn get_id_user_name(&self, id_user: IdUser) -> String {
        match self.id_users.get_id_user_name(id_user) {
            Some(name) => name,
            None => ANONYMOUS_USER_NAME.to_string(),
        }
    }

    /// Informe qu'un utilisateur accède à la [`Database`] en ÉCRITURE
    /// (Ici database est mutable)
    pub fn user_write_tag(&mut self, id_user: IdUser, tag: &Tag) {
        // println!("{tag} written by user #{id_user}");
        let notification_change = NotificationChange {
            id_user,
            id_tag: tag.id_tag,
        };
        self.id_users.add_change(&notification_change);
    }

    /// Répond à un utilisateur pour lui signaler les mises à jour de la [`Database`]
    /// L'utilisateur à la possibilité d'indiquer qu'il souhaite être également notifié
    /// des mises à jour que lui-même à réalisées ou par les modifications effectuées par
    /// des utilisateurs anonymes.
    ///
    /// Un utilisateur non identifié n'a accès à aucun historique de modifications.
    ///
    /// Cette primitive retourne une modification qui est toujours postérieure aux modifications
    /// effectuées la notification retournée lors de précédente consultation (y compris si None).
    ///
    /// Dès lors, l'usage des sélecteurs pour ignorer ses propres notifications ou les notifications
    /// d'un utilisateur anonyme impacte la notification de modification qui sera retournée lors de
    /// l'interrogation suivante: Les modifications non notifiées par usage d'une sélection sont 'zappées'
    /// et ne sont plus notifiées ultérieurement
    #[allow(dead_code)]
    pub fn get_change(
        &mut self,
        id_user: IdUser,
        include_my_changes: bool,
        include_anonymous_changes: bool,
    ) -> Option<NotificationChange> {
        self.id_users
            .get_change(id_user, include_my_changes, include_anonymous_changes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_users() {
        let mut db = Database::default();

        let id_user_1 = db.get_id_user("user1", false);
        assert!(id_user_1 != ID_ANONYMOUS_USER);
        assert_eq!(db.get_id_user_name(id_user_1), "user1");

        let id_user_2 = db.get_id_user("user2", false);
        assert!(id_user_1 != id_user_2);
        assert_eq!(db.get_id_user_name(id_user_2), "user2");
    }

    #[test]
    fn test_anonymous_notifications() {
        let mut db = Database::default();

        // Création d'un tag
        let tag_1 = Tag {
            word_address: 0x0010,
            id_tag: IdTag::new(1, 1, [0, 0, 0]),
            t_format: TFormat::U16,
            ..Default::default()
        };
        db.add_tag(&tag_1);

        // Création d'un second tag
        let tag_2 = Tag {
            word_address: 0x0020,
            id_tag: IdTag::new(2, 2, [0, 0, 0]),
            t_format: TFormat::U16,
            ..Default::default()
        };
        db.add_tag(&tag_2);

        // Création d'un user
        let id_user = db.get_id_user("user", true);

        // Pas d'historique pour cet utilisateur
        assert!(db.get_change(id_user, true, true).is_none());

        // Mise à jour de la database par un utilisateur anonyme
        db.set_u16_to_id_tag(ID_ANONYMOUS_USER, tag_1.id_tag, 1);

        // Pas d'historique pour user s'il n'est pas intéressé par les modifications faites
        // par les utilisateurs anonymes
        assert!(db.get_change(id_user, true, false).is_none());

        // Pas d'historique non plus pour user s'il est maintenant intéressé par les modifications faites
        // par les utilisateurs anonymes (car la notification a été ignorée lors de l'appel précédent)
        assert!(db.get_change(id_user, true, true).is_none());

        // Nouvelle mise à jour de la database par un utilisateur anonyme
        // Autre tag que le précédent sinon filtrage
        db.set_u16_to_id_tag(ID_ANONYMOUS_USER, tag_2.id_tag, 2);

        // Cette modification est notifiée à user s'il s'intéresse aux modifications faites
        // par les utilisateurs anonymes
        let option_notification_change = db.get_change(id_user, true, true);
        assert!(option_notification_change.is_some());
        match db.get_tag_from_id_tag(option_notification_change.unwrap().id_tag) {
            Some(tag) => {
                assert_eq!(tag.word_address, tag_2.word_address);
            }
            None => panic!("notification_change sans tag"),
        }

        // Puis plus de notification à suivre
        assert!(db.get_change(id_user, true, true).is_none());
    }

    #[test]
    fn test_self_notifications() {
        let mut db = Database::default();

        // Création d'un tag
        let tag_1 = Tag {
            word_address: 0x0010,
            id_tag: IdTag::new(1, 1, [0, 0, 0]),
            t_format: TFormat::U16,
            ..Default::default()
        };
        db.add_tag(&tag_1);

        // Création d'un second tag
        let tag_2 = Tag {
            word_address: 0x0020,
            id_tag: IdTag::new(2, 2, [0, 0, 0]),
            t_format: TFormat::U16,
            ..Default::default()
        };
        db.add_tag(&tag_2);

        // Création d'un user
        let id_user = db.get_id_user("user", true);

        // Mise à jour de la database par user
        db.set_u16_to_id_tag(id_user, tag_1.id_tag, 1);

        // Pas d'historique pour user s'il n'est pas intéressé par ses propres modifications
        assert!(db.get_change(id_user, false, true).is_none());

        // Pas d'historique non plus pour user s'il est maintenant intéressé par propres modifications
        // (car la notification a été ignorée lors de l'appel précédent)
        assert!(db.get_change(id_user, true, true).is_none());

        // Nouvelle mise à jour de la database par un user
        // Autre tag sinon filtrage
        db.set_u16_to_id_tag(id_user, tag_2.id_tag, 2);

        // Cette modification est notifiée à user s'il s'intéresse à ses propres modifications
        let option_notification_change = db.get_change(id_user, true, true);
        assert!(option_notification_change.is_some());
        match db.get_tag_from_id_tag(option_notification_change.unwrap().id_tag) {
            Some(tag) => {
                assert_eq!(tag.word_address, tag_2.word_address);
            }
            None => panic!("notification_change sans tag"),
        }
    }

    #[test]
    fn test_multi_users_notifications() {
        let mut db = Database::default();

        // Création d'un tag_1
        let tag_1 = Tag {
            word_address: 0x0010,
            id_tag: IdTag::new(1, 1, [0, 0, 0]),
            t_format: TFormat::U16,
            ..Default::default()
        };
        db.add_tag(&tag_1);

        // Création d'un tag_2
        let tag_2 = Tag {
            word_address: 0x0020,
            id_tag: IdTag::new(2, 2, [0, 0, 0]),
            t_format: TFormat::U16,
            ..Default::default()
        };
        db.add_tag(&tag_2);

        // Création d'un user_1
        let id_user_1 = db.get_id_user("user1", true);

        // Création d'un user_2
        let id_user_2 = db.get_id_user("user2", true);

        // Mise à jour de la database par user_1
        db.set_u16_to_id_tag(id_user_1, tag_1.id_tag, 1);

        // Mise à jour de la database par user_2
        db.set_u16_to_id_tag(id_user_2, tag_2.id_tag, 2);

        // User_1 est notifié de la modif de user_2
        let option_notification_change = db.get_change(id_user_1, false, true);
        assert!(option_notification_change.is_some());
        let tag = db.get_tag_from_id_tag(option_notification_change.unwrap().id_tag);
        assert_eq!(tag.unwrap().word_address, tag_2.word_address);

        // User_2 est notifié de la modif de user_1
        let option_notification_change = db.get_change(id_user_2, false, true);
        assert!(option_notification_change.is_some());
        let tag = db.get_tag_from_id_tag(option_notification_change.unwrap().id_tag);
        assert_eq!(tag.unwrap().word_address, tag_1.word_address);

        // Pas d'autre notification pour user_2
        assert!(db.get_change(id_user_1, false, true).is_none());

        // Ni pour user_1
        assert!(db.get_change(id_user_2, false, true).is_none());
    }

    #[test]
    fn test_unknown_user_notifications() {
        let mut db = Database::default();

        // Création d'un tag
        let tag = Tag {
            word_address: 0x0010,
            id_tag: IdTag::new(1, 1, [0, 0, 0]),
            t_format: TFormat::U16,
            ..Default::default()
        };
        db.add_tag(&tag);

        // Création d'un user
        let id_user = db.get_id_user("user", true);

        // Mise à jour de la database par user
        db.set_u16_to_id_tag(id_user, tag.id_tag, 1);

        // User qui ne s'identifie pas correctement
        let id_unknown_user = 0x1234;

        // Pas d'historique pour user non identifié
        assert!(db.get_change(id_unknown_user, true, true).is_none());

        // Par contre l'utilisateur identifié a bien accès à ses propres modifications
        assert!(db.get_change(id_user, true, true).is_some());

        // Plus de notification pour l'utilisateur identifié
        assert!(db.get_change(id_user, true, true).is_none());

        // Modif de la database pas ce user non identifié
        db.set_u16_to_id_tag(id_unknown_user, tag.id_tag, 2);

        // Toujours pas d'historique pour user non identifié
        assert!(db.get_change(id_unknown_user, true, true).is_none());

        // Par contre l'utilisateur identifié a bien accès aux modifications de cet utilisateur non identifié
        assert!(db.get_change(id_user, true, true).is_some());
    }

    #[test]
    fn test_multi_tags_notifications() {
        let mut db = Database::default();

        // Création d'un id_tag_1/tag_1 en 0x10
        let id_tag_1 = IdTag::new(1, 10, [0, 0, 0]);
        let tag_1 = Tag {
            word_address: 0x0010,
            id_tag: id_tag_1,
            t_format: TFormat::U16,
            ..Default::default()
        };
        db.add_tag(&tag_1);

        // Création d'un id_tag_2/tag_2 en 0x11
        let id_tag_2 = IdTag::new(1, 11, [0, 0, 0]);
        let tag_2 = Tag {
            word_address: 0x0011,
            id_tag: id_tag_2,
            t_format: TFormat::U16,
            ..Default::default()
        };
        db.add_tag(&tag_2);

        // Création d'un user
        let id_user = db.get_id_user("user", true);

        // Pas de modification initialement
        assert!(db.get_change(id_user, true, true).is_none());

        // Mise à jour de la database par user
        // La mise à jour effectuée modifie les 2 tags tag_1 et tag_2
        db.set_u32_to_id_tag(id_user, id_tag_1, 0x0001_0002);

        // L'utilisateur doit pouvoir retrouver les notifications pour 2 tags modifiés
        let notif_1 = db.get_change(id_user, true, true);
        assert!(notif_1.is_some());
        let notif_2 = db.get_change(id_user, true, true);
        assert!(notif_2.is_some());

        // Et les notifications doivent référencer tag_1 et tag_2 (par forcément dans l'ordre...)
        let notif_1_id_tag = notif_1.unwrap().id_tag;
        let notif_2_id_tag = notif_2.unwrap().id_tag;
        assert!(notif_1_id_tag != notif_2_id_tag);
        assert!(notif_1_id_tag == id_tag_1 || notif_1_id_tag == id_tag_2);
        assert!(notif_2_id_tag == id_tag_1 || notif_2_id_tag == id_tag_2);

        // Plus de notification pour l'utilisateur identifié
        assert!(db.get_change(id_user, true, true).is_none());
    }

    #[test]
    fn test_purge_changes() {
        let mut db = Database::default();

        // Création d'un tag 1
        let tag_1 = Tag {
            word_address: 0x0010,
            id_tag: IdTag::new(1, 1, [0, 0, 0]),
            t_format: TFormat::U16,
            ..Default::default()
        };
        db.add_tag(&tag_1);

        // Création d'un tag 2
        let tag_2 = Tag {
            word_address: 0x0020,
            id_tag: IdTag::new(2, 2, [0, 0, 0]),
            t_format: TFormat::U16,
            ..Default::default()
        };
        db.add_tag(&tag_2);

        // Création d'un user
        let id_user = db.get_id_user("user", true);

        // Mise à jour de la database par user
        db.set_u16_to_id_tag(id_user, tag_1.id_tag, 1);
        db.set_u16_to_id_tag(id_user, tag_2.id_tag, 2);

        // Mesure de la taille de l'historique des changements avant les notifications
        // Ici 2 changements dans l'historique
        let start_vec_changes_len = db.id_users.vec_changes.len();

        // L'utilisateur récupère toutes les notifications
        loop {
            if db.get_change(id_user, true, true).is_none() {
                break;
            }
        }

        // La purge se fait lorsqu'un nouveau changement est fait
        db.set_u16_to_id_tag(id_user, tag_1.id_tag, 2);

        // La taille de l'historique des changements doit avoir diminué (plus que 1)
        assert!(db.id_users.vec_changes.len() < start_vec_changes_len);
    }

    #[test]
    fn test_multi_users_purge_changes() {
        let mut db = Database::default();

        // Création d'un tag 1
        let tag_1 = Tag {
            word_address: 0x0010,
            id_tag: IdTag::new(1, 1, [0, 0, 0]),
            t_format: TFormat::U16,
            ..Default::default()
        };
        db.add_tag(&tag_1);

        // Création d'un tag 2
        let tag_2 = Tag {
            word_address: 0x0020,
            id_tag: IdTag::new(2, 2, [0, 0, 0]),
            t_format: TFormat::U16,
            ..Default::default()
        };
        db.add_tag(&tag_2);

        // Création d'un user_1
        let id_user_1 = db.get_id_user("user1", true);

        // Création d'un user_2
        let id_user_2 = db.get_id_user("user2", true);

        // Mise à jour de la database par user1
        db.set_u16_to_id_tag(id_user_1, tag_1.id_tag, 1);
        db.set_u16_to_id_tag(id_user_2, tag_2.id_tag, 2);
        db.set_u16_to_id_tag(id_user_1, tag_1.id_tag, 3);
        db.set_u16_to_id_tag(id_user_2, tag_2.id_tag, 4);

        // Mesure de la taille de l'historique des changements avant les notifications
        let start_vec_changes_len = db.id_users.vec_changes.len();

        // User1 et User2 récupèrent toutes les notifications
        loop {
            if db.get_change(id_user_1, true, true).is_none()
                && db.get_change(id_user_2, true, true).is_none()
            {
                break;
            }
        }

        // La purge se fait lorsqu'un nouveau changement est fait
        db.set_u16_to_id_tag(id_user_1, tag_1.id_tag, 5);

        // La taille de l'historique des changements doit avoir diminué (plus que 1)
        assert!(db.id_users.vec_changes.len() < start_vec_changes_len);
    }
}
