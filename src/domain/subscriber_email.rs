use validator::validate_email;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<SubscriberEmail, String> {
        if validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(format!("{} is not a valid subscriber email.", s))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use claim::assert_err;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use quickcheck::{Arbitrary, Gen};

    #[derive(Clone, Debug)]
    struct ValidEmailFixture(pub String);

    impl Arbitrary for ValidEmailFixture {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            // `Property-based testing`. We create some random
            // canonical valid data to test not concrete tokens
            // but the concept of a valid input.
            //
            // `g` is an argument of type G, and G is constrained
            // by a trait bound, Gen, therefore it must implement the
            // Gen trait in `quickcheck`, where `G` stands for "generator".
            //
            // `SafeEmail` implements the `Fake` trait. And `fake_with_rng()`
            // will accept any type that implements `rand:Rng`, that means,
            // any types implementing `RngCore` (`G`).
            //
            // `quickcheck` and `fake` can be used together because of this
            // match.
            let email = SafeEmail().fake_with_rng(g);
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_succesfully(valid_email: ValidEmailFixture) -> bool {
        // run `cargo test valid_emails -- --nocapture` to see the randomly
        // generated valid emails
        dbg!(&valid_email.0);
        SubscriberEmail::parse(valid_email.0).is_ok()
    }

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }
}
