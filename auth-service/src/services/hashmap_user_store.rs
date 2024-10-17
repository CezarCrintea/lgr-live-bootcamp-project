use std::collections::HashMap;

use crate::domain::User;

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<String, User>,
}

impl HashmapUserStore {
    pub fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if let Some(_) = self.users.get(&user.email) {
            return Err(UserStoreError::UserAlreadyExists);
        }

        self.users.insert(user.email.clone(), user);
        Ok(())
    }

    pub fn get_user(&self, email: &str) -> Result<User, UserStoreError> {
        if let Some(existing_user) = self.users.get(email) {
            Ok(existing_user.clone())
        } else {
            Err(UserStoreError::UserNotFound)
        }
    }

    pub fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        let existing_user = self.get_user(email)?;
        if existing_user.password != password {
            Err(UserStoreError::InvalidCredentials)
        } else {
            Ok(())
        }
    }
}

// TODO: Add unit tests for your `HashmapUserStore` implementation
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let mut store = HashmapUserStore::default();
        let email = "1@email.com".to_string();
        let user = User::new(email.clone(), "pass1".to_owned(), false);

        let result = store.add_user(user.clone());
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);

        let added_user = store.users.get(&email);
        assert!(added_user.is_some(), "User was not added to the store");
        assert_eq!(added_user.unwrap(), &user, "The added user does not match");
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut store = HashmapUserStore::default();
        let email = "1@email.com".to_string();
        let user = User::new(email.clone(), "pass1".to_owned(), false);
        store.users.insert(email.clone(), user.clone());

        let result = store.get_user(&email);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), user);
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut store = HashmapUserStore::default();
        let email = "1@email.com".to_string();
        let password = "pass123".to_string();
        let user = User::new(email.clone(), password.clone(), false);
        store.users.insert(email.clone(), user.clone());

        let result = store.validate_user(&email, &password);
        assert!(result.is_ok());
    }
}
