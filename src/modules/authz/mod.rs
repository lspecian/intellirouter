// Commented out due to lifetime issues
// pub mod auth;
pub mod auth_fixed;
pub use auth_fixed as auth;
pub mod middleware;
pub mod rbac;
pub mod routes;

use std::sync::Arc;

pub use auth::{ApiKey, AuthContext, AuthError, AuthManager};
pub use middleware::{auth_middleware, require_permission};
pub use rbac::{RbacError, RbacManager, Role};

pub fn create_auth_manager() -> Arc<AuthManager> {
    Arc::new(AuthManager::new())
}

pub fn create_rbac_manager() -> Arc<RbacManager> {
    Arc::new(RbacManager::new())
}

#[cfg(all(test, not(feature = "production")))]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_auth_manager() {
        let auth_manager = AuthManager::new();

        // Create a test API key
        let api_key = ApiKey {
            key: "test-key".to_string(),
            name: "Test Key".to_string(),
            roles: vec!["user".to_string()],
            created_at: Utc::now(),
        };

        // Add the API key
        auth_manager.add_api_key(api_key.clone()).unwrap();

        // Validate the API key
        let validated_key = auth_manager.validate_api_key("test-key").unwrap();
        assert!(validated_key.is_some());
        assert_eq!(validated_key.unwrap().name, "Test Key");

        // Check role
        let has_role = auth_manager.has_role("test-key", "user").unwrap();
        assert!(has_role);

        let has_admin_role = auth_manager.has_role("test-key", "admin").unwrap();
        assert!(!has_admin_role);

        // Remove the API key
        let removed = auth_manager.remove_api_key("test-key").unwrap();
        assert!(removed);

        // Validate the API key again
        let validated_key = auth_manager.validate_api_key("test-key").unwrap();
        assert!(validated_key.is_none());
    }

    #[test]
    fn test_rbac_manager() {
        let rbac_manager = RbacManager::new();

        // Add a new role
        rbac_manager.add_role("test-role").unwrap();

        // Add a permission to the role
        rbac_manager
            .add_permission_to_role("test-role", "test:permission")
            .unwrap();

        // Check permission
        let has_permission = rbac_manager
            .has_permission(&["test-role".to_string()], "test:permission")
            .unwrap();
        assert!(has_permission);

        let has_other_permission = rbac_manager
            .has_permission(&["test-role".to_string()], "other:permission")
            .unwrap();
        assert!(!has_other_permission);

        // Remove the permission
        rbac_manager
            .remove_permission_from_role("test-role", "test:permission")
            .unwrap();

        // Check permission again
        let has_permission = rbac_manager
            .has_permission(&["test-role".to_string()], "test:permission")
            .unwrap();
        assert!(!has_permission);

        // Remove the role
        rbac_manager.remove_role("test-role").unwrap();

        // Try to get the role
        let role = rbac_manager.get_role("test-role").unwrap();
        assert!(role.is_none());
    }
}
