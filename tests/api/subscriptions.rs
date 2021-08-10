use crate::helpers::spawn_app;

#[actix_rt::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let test_app = spawn_app().await;
    let body = "name=nicolas%20bourbaki&email=nick_bourbaki%40gmail.com";

    // Act
    let response = test_app.post_subscriptions(body.into()).await;

    // Assert
    assert_eq!(200, response.status().as_u16());

    // Verify

    // This macro needs a DATABASE_URL env var defined, so we need to
    // add a .env at top-level to compile this code. Note that it's
    // pushed to repo, because we need it to run the CI pipeline
    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "nick_bourbaki@gmail.com");
    assert_eq!(saved.name, "nicolas bourbaki");
}

// This is an example of table-driven test (aka parametrised test). It is particularly
// helpful when dealing with bad inputs - instead of duplicating test logic several
// times we can simply run the same assertion against a collection of known invalid
// bodies that we expect to fail in the same way
#[actix_rt::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let test_app = spawn_app().await;
    let test_cases = vec![
        ("name=nicolas%20bourbaki", "missing the email"),
        ("email=nick_bourbaki%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = test_app.post_subscriptions(invalid_body.into()).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[actix_rt::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() {
    // Arrange
    let test_app = spawn_app().await;
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];

    for (body, description) in test_cases {
        // Act
        let response = test_app.post_subscriptions(body.into()).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 Bad Request when the payload was {}.",
            description
        );
    }
}
