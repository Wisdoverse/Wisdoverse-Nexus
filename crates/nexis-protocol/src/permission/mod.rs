//! Permission domain extensions for Wisdoverse Nexus.

pub use crate::{Action, Permissions};

use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct PermissionChecker {
    permissions: Permissions,
}

impl PermissionChecker {
    pub fn new(permissions: Permissions) -> Self {
        Self { permissions }
    }

    pub fn can_read(&self) -> bool {
        self.permissions.can(Action::Read)
    }

    pub fn can_write(&self) -> bool {
        self.permissions.can(Action::Write)
    }

    pub fn can_invoke(&self) -> bool {
        self.permissions.can(Action::Invoke)
    }

    pub fn is_admin(&self) -> bool {
        self.permissions.can(Action::Admin)
    }

    pub fn can_access_room(&self, room_id: &str) -> bool {
        self.permissions.can_access_room(room_id)
    }

    pub fn effective_permissions(&self, room_id: &str) -> HashSet<Action> {
        let mut actions = HashSet::new();
        if !self.can_access_room(room_id) {
            return actions;
        }
        for action in [Action::Read, Action::Write, Action::Invoke, Action::Admin] {
            if self.permissions.can(action) {
                actions.insert(action);
            }
        }
        actions
    }
}

impl From<Permissions> for PermissionChecker {
    fn from(permissions: Permissions) -> Self {
        Self::new(permissions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Action, Permissions};

    #[test]
    fn admin_implies_all_actions() {
        let perms = Permissions::new(vec!["*".into()], vec![Action::Admin]);
        let checker = PermissionChecker::new(perms);
        assert!(checker.is_admin());
        assert!(checker.can_read());
        assert!(checker.can_write());
        assert!(checker.can_invoke());
    }

    #[test]
    fn read_only_checker() {
        let perms = Permissions::new(vec!["room_1".into()], vec![Action::Read]);
        let checker = PermissionChecker::new(perms);
        assert!(checker.can_read());
        assert!(!checker.can_write());
        assert!(!checker.can_invoke());
        assert!(!checker.is_admin());
    }

    #[test]
    fn can_access_wildcard_room() {
        let perms = Permissions::new(vec!["*".into()], vec![Action::Read]);
        let checker = PermissionChecker::new(perms);
        assert!(checker.can_access_room("any_room"));
    }

    #[test]
    fn can_access_specific_room() {
        let perms = Permissions::new(vec!["room_a".into()], vec![Action::Read]);
        let checker = PermissionChecker::new(perms);
        assert!(checker.can_access_room("room_a"));
        assert!(!checker.can_access_room("room_b"));
    }

    #[test]
    fn effective_permissions_no_access() {
        let perms = Permissions::new(vec!["room_x".into()], vec![Action::Read]);
        let checker = PermissionChecker::new(perms);
        let eff = checker.effective_permissions("room_y");
        assert!(eff.is_empty());
    }

    #[test]
    fn effective_permissions_with_access() {
        let perms = Permissions::new(vec!["room_x".into()], vec![Action::Read, Action::Write]);
        let checker = PermissionChecker::new(perms);
        let eff = checker.effective_permissions("room_x");
        assert!(eff.contains(&Action::Read));
        assert!(eff.contains(&Action::Write));
        assert!(!eff.contains(&Action::Invoke));
    }

    #[test]
    fn effective_permissions_admin_gets_all() {
        let perms = Permissions::new(vec!["r".into()], vec![Action::Admin]);
        let checker = PermissionChecker::new(perms);
        let eff = checker.effective_permissions("r");
        assert_eq!(eff.len(), 4);
    }

    #[test]
    fn from_permissions_conversion() {
        let perms = Permissions::new(vec!["*".into()], vec![Action::Admin]);
        let checker: PermissionChecker = perms.into();
        assert!(checker.is_admin());
    }
}
