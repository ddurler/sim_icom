//! Gestion de l'historique des modifications de la [`Database`] pour les différents [`IdUser`]
//!
//! Le système de notification mis en place ici n'est pas un modèle 'observateur' avec un callback
//! vers une fonction de l'utilisateur : Pas facile à mettre en place en Rust via les `threads`.
//!
//! Ici, l'utilisateur doit 'poller' pour s'enquérir des dernières modifications dans la [`Database`].

use std::collections::HashMap;

#[cfg(test)]
use super::{IdTag, TFormat};

use super::{Database, IdUser, Tag, WordAddress, ID_ANONYMOUS_USER};

/// Structure pour les suivis des différents [`IdUser`] identifiés
#[derive(Debug, Default)]
pub struct IdUsers {
    /// Générateur d'[`IdUser`]
    id_user_seed: IdUser,

    /// Historique des modifications de la [`Database`]
    vec_changes: Vec<(IdUser, WordAddress)>,

    /// Hash [`IdUser`] -> premier index dans `vec_changes` qui n'a pas été notifié à ce [`IdUser`],
    hash_notifications: HashMap<IdUser, usize>,
}

impl Database {
    /// Retourne un [`IdUser`] pour les opérations de la [`Database`]
    #[allow(dead_code)]
    pub fn get_id_user(&mut self) -> IdUser {
        // Si le nombre d'utilisateurs différents devient trop grand,
        // on attribue le même [`IdUser`] à tous les nouveaux...
        self.id_users.id_user_seed = self.id_users.id_user_seed.saturating_add(1);

        // Mémorise le 1er offset pour les notifications à suivre
        let offset = self.id_users.vec_changes.len();
        self.id_users
            .hash_notifications
            .insert(self.id_users.id_user_seed, offset);

        self.id_users.id_user_seed
    }

    /// Informe qu'un utilisateur accède à la [`Database`] en ÉCRITURE
    /// (Ici database est mutable)
    pub fn user_write_tag(&mut self, id_user: IdUser, tag: &Tag) {
        // println!("{tag} written by user #{id_user}");
        self.id_users.vec_changes.push((id_user, tag.word_address));
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
    ) -> Option<Tag> {
        // Pas d'historique de notification pour les anonymes et les utilisateurs non identifiés
        if !self.id_users.hash_notifications.contains_key(&id_user) {
            return None;
        }
        // Dernier offset non notifié à cet utilisateur
        let offset = *self
            .id_users
            .hash_notifications
            .get(&id_user)
            .unwrap_or(&self.id_users.vec_changes.len());

        // Parcours des offsets de l'historique
        let mut notification_offset = offset;
        while self.id_users.vec_changes.len() > notification_offset {
            let (change_id_user, change_word_address) =
                self.id_users.vec_changes[notification_offset];
            // A notifier ?
            if (include_anonymous_changes || change_id_user != ID_ANONYMOUS_USER)
                && (include_my_changes || change_id_user != id_user)
            {
                // Mémorisation du dernier offset non notifié à cet utilisateur
                self.id_users
                    .hash_notifications
                    .insert(id_user, notification_offset + 1);
                // Modification de la database à retourner au demandeur
                let tag = self
                    .get_tag_from_word_address(change_word_address)
                    .unwrap()
                    .clone();
                return Some(tag);
            }
            notification_offset += 1;
        }

        // Rien à notifier au demandeur
        if notification_offset != offset {
            // Mémorisation du dernier offset non notifié à cet utilisateur
            self.id_users
                .hash_notifications
                .insert(id_user, notification_offset);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_users() {
        let mut db = Database::default();

        let id_user1 = db.get_id_user();
        assert!(id_user1 != ID_ANONYMOUS_USER);

        let id_user_2 = db.get_id_user();
        assert!(id_user1 != id_user_2);
    }

    #[test]
    fn test_anonymous_notifications() {
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
        let id_user = db.get_id_user();

        // Pas d'historique pour cet utilisateur
        assert!(db.get_change(id_user, true, true).is_none());

        // Mise à jour de la database par un utilisateur anonyme
        db.set_u16_to_id_tag(ID_ANONYMOUS_USER, tag.id_tag, 1);

        // Pas d'historique pour user s'il n'est pas intéressé par les modifications faites
        // par les utilisateurs anonymes
        assert!(db.get_change(id_user, true, false).is_none());

        // Pas d'historique non plus pour user s'il est maintenant intéressé par les modifications faites
        // par les utilisateurs anonymes (car la notification a été ignorée lors de l'appel précédent)
        assert!(db.get_change(id_user, true, true).is_none());

        // Nouvelle mise à jour de la database par un utilisateur anonyme
        db.set_u16_to_id_tag(ID_ANONYMOUS_USER, tag.id_tag, 2);

        // Cette modification est notifiée à user s'il s'intéresse aux modifications faites
        // par les utilisateurs anonymes
        let option_tag = db.get_change(id_user, true, true);
        assert!(option_tag.is_some());
        assert_eq!(option_tag.unwrap().word_address, tag.word_address);

        // Puis plus de notification à suivre
        assert!(db.get_change(id_user, true, true).is_none());
    }

    #[test]
    fn test_self_notifications() {
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
        let id_user = db.get_id_user();

        // Mise à jour de la database par user
        db.set_u16_to_id_tag(id_user, tag.id_tag, 1);

        // Pas d'historique pour user s'il n'est pas intéressé par ses propres modifications
        assert!(db.get_change(id_user, false, true).is_none());

        // Pas d'historique non plus pour user s'il est maintenant intéressé par propres modifications
        // (car la notification a été ignorée lors de l'appel précédent)
        assert!(db.get_change(id_user, true, true).is_none());

        // Nouvelle mise à jour de la database par un user
        db.set_u16_to_id_tag(id_user, tag.id_tag, 2);

        // Cette modification est notifiée à user s'il s'intéresse à ses propres modifications
        let option_tag = db.get_change(id_user, true, true);
        assert!(option_tag.is_some());
        assert_eq!(option_tag.unwrap().word_address, tag.word_address);
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
        let id_user_1 = db.get_id_user();

        // Création d'un user_2
        let id_user_2 = db.get_id_user();

        // Mise à jour de la database par user_1
        db.set_u16_to_id_tag(id_user_1, tag_1.id_tag, 1);

        // Mise à jour de la database par user_2
        db.set_u16_to_id_tag(id_user_2, tag_2.id_tag, 2);

        // User_1 est notifié de la modif de user_2
        let option_tag = db.get_change(id_user_1, false, true);
        assert!(option_tag.is_some());
        assert_eq!(option_tag.unwrap().word_address, tag_2.word_address);

        // User_2 est notifié de la modif de user_1
        let option_tag = db.get_change(id_user_2, false, true);
        assert!(option_tag.is_some());
        assert_eq!(option_tag.unwrap().word_address, tag_1.word_address);

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
        let id_user = db.get_id_user();

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
        let id_user = db.get_id_user();

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
}
