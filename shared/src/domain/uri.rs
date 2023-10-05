use std::fmt::{Display, Formatter};
use std::str::FromStr;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use validator::validate_url;

#[derive(Debug, Clone)]
pub struct Uri(String);

impl Uri {
    pub fn parse(s: String) -> Result<Uri, String> {
        if validate_url(&s) && (s.starts_with("http://") || s.starts_with("https://")) {
            Ok(Self(s))
        } else {
            Err(format!("{} is not a valid URI.", s))
        }
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Display for Uri {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'de> Deserialize<'de> for Uri {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Uri::parse(s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for Uri {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_str(self.0.as_str())
    }
}

impl AsRef<str> for Uri {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl FromStr for Uri {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uri::parse(s.to_string())
    }
}

impl From<String> for Uri {
    fn from(s: String) -> Self {
        Uri::parse(s).expect("Invalid URI format")
    }
}

#[cfg(test)]
mod tests {
    use super::Uri;
    use fake::Fake;
    use fake::faker::internet::en::DomainSuffix;
    use fake::faker::internet::en::Username;
    use quickcheck::{Arbitrary, Gen};
    use rand::rngs::StdRng;
    use rand::{SeedableRng};

    #[test]
    fn empty_string_is_rejected() {
        let uri = "".to_string();
        assert!(Uri::parse(uri).is_err());
    }

    #[test]
    fn invalid_scheme_is_rejected() {
        let uri = "htp://www.example.com".to_string();
        let result = Uri::parse(uri);
        assert!(result.is_err(), "Expected error, but got: {:?}", result);
    }

    #[test]
    fn missing_domain_is_rejected() {
        let uri = "https://".to_string();
        assert!(Uri::parse(uri).is_err());
    }

    #[derive(Debug, Clone)]
    struct ValidUriFixture(pub String);

    impl Arbitrary for ValidUriFixture {
        fn arbitrary(g: &mut Gen) -> Self {
            let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));
            let scheme = if *g.choose(&[true, false]).unwrap() { "http" } else { "https" };
            let name: String = Username().fake_with_rng(&mut rng);
            let suffix: String = DomainSuffix().fake_with_rng(&mut rng);

            let domain = format!("www.{}.{}", name, suffix);
            let uri = format!("{}://{}", scheme, domain);

            Self(uri)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_uris_are_parsed_successfully(valid_uri: ValidUriFixture) -> bool {
        Uri::parse(valid_uri.0).is_ok()
    }
}
