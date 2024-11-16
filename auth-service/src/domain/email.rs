use color_eyre::eyre::{eyre, Result};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Email(String);

impl Email {
    pub fn parse(email: String) -> Result<Email> {
        if !email.contains("@") {
            return Err(eyre!(format!("{} is not a valid email.", email)));
        }

        Ok(Email(email))
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        let result = Email::parse("user@example.com".to_string());

        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_ref(), "user@example.com")
    }

    #[test]
    fn test_invalid_email_no_at_symbol() {
        let result = Email::parse("userexample.com".to_string());

        assert!(result.is_err());
    }

    #[test]
    fn test_as_ref_trait() {
        let email = Email::parse("user@example.com".to_string()).unwrap();

        assert_eq!(email.as_ref(), "user@example.com");
    }
}
