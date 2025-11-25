use validator::ValidateEmail;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<Self, String> {
        if !s.validate_email() {
            return Err(format!("{s} is not a valid subscriber email"));
        }

        Ok(Self(s))
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use claim::assert_err;

    use crate::domain::SubscriberEmail;

    #[test]
    fn empty_string_rejected() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_at_symbol_rejected() {
        let email = "abc.def".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_subject_rejected() {
        let email = "@abc.def".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }
}
