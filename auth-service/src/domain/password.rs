#[derive(Clone, Debug, PartialEq)]
pub struct Password(String);

impl Password {
    pub fn parse(password: String) -> Result<Password, String> {
        if password.chars().count() < 8 {
            return Err("password too short".to_string());
        }

        Ok(Password(password))
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_password() {
        let result = Password::parse("password123".to_string());

        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_ref(), "password123");
    }

    #[test]
    fn test_invalid_password_too_short() {
        let result = Password::parse("short".to_string());

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "password too short");
    }

    #[test]
    fn test_as_ref_trait() {
        let password = Password::parse("password123".to_string()).unwrap();
        assert_eq!(password.as_ref(), "password123");
    }
}
