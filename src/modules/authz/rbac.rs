use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use thiserror::Error;

use super::auth::AuthError;

#[derive(Debug, Error)]
pub enum RbacError {
    #[error("Failed to acquire lock")]
    LockError,

    #[error("Role already exists")]
    RoleAlreadyExists,

    #[error("Role does not exist")]
    RoleDoesNotExist,

    #[error("Permission already exists")]
    PermissionAlreadyExists,

    #[error("Permission does not exist")]
    PermissionDoesNotExist,
}

#[derive(Debug, Clone)]
pub struct Role {
    pub name: String,
    pub permissions: HashSet<String>,
}

pub struct RbacManager {
    roles: RwLock<HashMap<String, Role>>,
}

impl RbacManager {
    pub fn new() -> Self {
        let mut roles = HashMap::new();

        // Add default roles
        let admin_role = Role {
            name: "admin".to_string(),
            permissions: ["*"].iter().map(|s| s.to_string()).collect(),
        };

        let user_role = Role {
            name: "user".to_string(),
            permissions: ["read:models", "execute:chat"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        };

        roles.insert(admin_role.name.clone(), admin_role);
        roles.insert(user_role.name.clone(), user_role);

        Self {
            roles: RwLock::new(roles),
        }
    }

    pub fn add_role(&self, name: &str) -> Result<(), RbacError> {
        let mut roles = self.roles.write().map_err(|_| RbacError::LockError)?;

        if roles.contains_key(name) {
            return Err(RbacError::RoleAlreadyExists);
        }

        roles.insert(
            name.to_string(),
            Role {
                name: name.to_string(),
                permissions: HashSet::new(),
            },
        );

        Ok(())
    }

    pub fn remove_role(&self, name: &str) -> Result<(), RbacError> {
        let mut roles = self.roles.write().map_err(|_| RbacError::LockError)?;

        if !roles.contains_key(name) {
            return Err(RbacError::RoleDoesNotExist);
        }

        roles.remove(name);

        Ok(())
    }

    pub fn add_permission_to_role(
        &self,
        role_name: &str,
        permission: &str,
    ) -> Result<(), RbacError> {
        let mut roles = self.roles.write().map_err(|_| RbacError::LockError)?;

        let role = roles
            .get_mut(role_name)
            .ok_or(RbacError::RoleDoesNotExist)?;

        if role.permissions.contains(permission) {
            return Err(RbacError::PermissionAlreadyExists);
        }

        role.permissions.insert(permission.to_string());

        Ok(())
    }

    pub fn remove_permission_from_role(
        &self,
        role_name: &str,
        permission: &str,
    ) -> Result<(), RbacError> {
        let mut roles = self.roles.write().map_err(|_| RbacError::LockError)?;

        let role = roles
            .get_mut(role_name)
            .ok_or(RbacError::RoleDoesNotExist)?;

        if !role.permissions.contains(permission) {
            return Err(RbacError::PermissionDoesNotExist);
        }

        role.permissions.remove(permission);

        Ok(())
    }

    pub fn has_permission(
        &self,
        role_names: &[String],
        permission: &str,
    ) -> Result<bool, RbacError> {
        let roles = self.roles.read().map_err(|_| RbacError::LockError)?;

        for role_name in role_names {
            if let Some(role) = roles.get(role_name) {
                if role.permissions.contains("*") || role.permissions.contains(permission) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    pub fn get_role(&self, name: &str) -> Result<Option<Role>, RbacError> {
        let roles = self.roles.read().map_err(|_| RbacError::LockError)?;
        Ok(roles.get(name).cloned())
    }

    pub fn list_roles(&self) -> Result<Vec<Role>, RbacError> {
        let roles = self.roles.read().map_err(|_| RbacError::LockError)?;
        Ok(roles.values().cloned().collect())
    }
}

pub fn check_permission(
    auth_context: &super::auth::AuthContext,
    rbac_manager: &Arc<RbacManager>,
    permission: &str,
) -> Result<bool, AuthError> {
    rbac_manager
        .has_permission(&auth_context.api_key.roles, permission)
        .map_err(|e| AuthError::InternalError(e.to_string()))
}
