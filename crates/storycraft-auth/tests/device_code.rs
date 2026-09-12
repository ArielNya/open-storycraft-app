//! Recorded device-code HTTP fixtures against a fake auth server.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use storycraft_auth::{CancellationToken, DeviceAuth, OAuthConfig};
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn config(server: &MockServer) -> OAuthConfig {
    OAuthConfig {
        auth_base: server.uri(),
        client_id: "test-client".into(),
        scopes: "openid api:access".into(),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn pending_then_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth2/device/code"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{
                "device_code": "dev-1",
                "user_code": "ABCD-EFGH",
                "verification_uri": "https://auth.example/device",
                "verification_uri_complete": "https://auth.example/device?code=ABCD-EFGH",
                "expires_in": 600,
                "interval": 0
            }"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/oauth2/token"))
        .and(body_string_contains("device_code=dev-1"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_raw(r#"{"error":"authorization_pending"}"#, "application/json"),
        )
        .up_to_n_times(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{
                "access_token": "access-xyz",
                "refresh_token": "refresh-xyz",
                "expires_in": 3600,
                "token_type": "Bearer"
            }"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let auth = DeviceAuth::new(config(&server)).unwrap();
    let pending = auth.request_code().await.unwrap();
    assert_eq!(pending.user_code, "ABCD-EFGH");
    assert!(pending.verification_uri_complete.is_some());

    let tokens = auth
        .poll_token(&pending, &CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(tokens.access_token, "access-xyz");
    assert_eq!(tokens.refresh_token.as_deref(), Some("refresh-xyz"));
}

#[tokio::test(flavor = "current_thread")]
async fn expired_token_stops_polling() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth2/device/code"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{
                "device_code": "dev-2",
                "user_code": "ZZZZ",
                "verification_uri": "https://auth.example/device",
                "expires_in": 5,
                "interval": 0
            }"#,
            "application/json",
        ))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/oauth2/token"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_raw(r#"{"error":"expired_token"}"#, "application/json"),
        )
        .mount(&server)
        .await;

    let auth = DeviceAuth::new(config(&server)).unwrap();
    let pending = auth.request_code().await.unwrap();
    let err = auth
        .poll_token(&pending, &CancellationToken::new())
        .await
        .unwrap_err();
    assert!(matches!(err, storycraft_auth::Error::Expired));
}

#[tokio::test(flavor = "current_thread")]
async fn slow_down_then_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth2/device/code"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{
                "device_code": "dev-3",
                "user_code": "SLOW",
                "verification_uri": "https://auth.example/device",
                "expires_in": 600,
                "interval": 0
            }"#,
            "application/json",
        ))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/oauth2/token"))
        .respond_with(
            ResponseTemplate::new(400).set_body_raw(r#"{"error":"slow_down"}"#, "application/json"),
        )
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"access_token":"after-slow","token_type":"Bearer"}"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let auth = DeviceAuth::new(config(&server)).unwrap();
    let pending = auth.request_code().await.unwrap();
    let tokens = auth
        .poll_token(&pending, &CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(tokens.access_token, "after-slow");
}
