use httptest::matchers::{contains, eq, json_decoded, request, url_decoded};
use httptest::{all_of, responders, Expectation};

#[tokio::test]
async fn test_supabase() {
    env_logger::init();

    let mut server = httptest::Server::run();

    let dummy_apikey = "dummy_apikey";

    let client = crate::Supabase::new(
        &server.url_str(""),
        dummy_apikey,
        None,
        crate::auth::SessionChangeListener::Ignore,
    );

    let dummy_username = "dummy_username";
    let dummy_password = "dummy_password";
    let dummy_refresh_token = "dummy_refresh_token";
    let dummy_access_token = "dummy_access_token";
    let dummy_expiration = (chrono::Utc::now().timestamp() + 3600) as u64; // One hour ahead
    let dummy_session = crate::auth::Session {
        access_token: dummy_access_token.to_string(),
        token_type: "".to_string(),
        expires_in: 0,
        expires_at: dummy_expiration,
        refresh_token: dummy_refresh_token.to_string(),
        user: Default::default(),
    };

    server.expect(
        Expectation::matching(all_of!(
            request::method("POST"),
            request::path("//auth/v1/token"),
            request::query(url_decoded(contains(("grant_type", "password")))),
            request::headers(contains(("apikey", dummy_apikey))),
            request::body(json_decoded(eq(serde_json::json!({
                "email": dummy_username,
                "password": dummy_password,
            }))))
        ))
        .respond_with(responders::json_encoded(dummy_session.clone())),
    );

    let received_session = client
        .login_with_email(dummy_username, dummy_password)
        .await
        .unwrap();

    assert_eq!(received_session, dummy_session);
    server.verify_and_clear();

    let dummy_table = "table";

    #[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Clone)]
    struct DummyTableStruct {
        id: i32,
        name: String,
    }

    let dummy_table_content = vec![DummyTableStruct {
        id: 1,
        name: "John Doe".to_string(),
    }];

    server.expect(
        Expectation::matching(all_of!(
            request::method("GET"),
            request::path("//rest/v1/table"),
            request::query(url_decoded(contains(("select", "*")))),
            request::headers(contains(("apikey", dummy_apikey))),
            request::headers(contains((
                "authorization",
                format!("Bearer {dummy_access_token}")
            )))
        ))
        .respond_with(responders::json_encoded(dummy_table_content.clone())),
    );

    let response = client
        .from(dummy_table)
        .await
        .unwrap()
        .select("*")
        .execute()
        .await
        .unwrap()
        .json::<Vec<DummyTableStruct>>()
        .await
        .unwrap();

    assert_eq!(response, dummy_table_content);
}
