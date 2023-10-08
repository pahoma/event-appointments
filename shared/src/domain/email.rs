use std::fmt::{Display, Formatter};
use std::str::FromStr;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use validator::validate_email;


#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub struct Email(String);

impl Email {

    pub fn parse(s: String) -> Result<Email, String> {
        if validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(format!("{} is not a valid email string.", s))
        }
    }

    pub fn into_inner(self) -> String {
        self.0
    }

}

impl Display for Email {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'de> Deserialize<'de> for Email {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Email::parse(s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for Email {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}


impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for Email {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Email::parse(s.to_string())
    }
}

impl TryFrom<String> for Email {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Email::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::Email;
    use claims::assert_err;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use quickcheck::Gen;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(Email::parse(email));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "somedomain.com".to_string();
        assert_err!(Email::parse(email));
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@somedomain.com".to_string();
        assert_err!(Email::parse(email));
    }

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut Gen) -> Self {
            let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));
            let email = SafeEmail().fake_with_rng(&mut rng);

            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        Email::parse(valid_email.0).is_ok()
    }
}