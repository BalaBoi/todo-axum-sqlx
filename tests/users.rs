use helpers::TestApp;
use reqwest::StatusCode;
use serde_json::json;
use sqlx::{test, PgPool};
use uuid::Uuid;

mod helpers;

#[test]
async fn login_failure_refreshes_page_with_flash_error(pool: PgPool) {
    let app = TestApp::new(pool).await;

    let login_body = json!({
        "email": Uuid::new_v4().to_string(),
        "password": Uuid::new_v4().to_string()
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap(),
        "/users/login"
    );
    let flash_cookie = response
        .cookies()
        .find(|p| p.name() == "error_flash")
        .unwrap();
    println!("{}", flash_cookie.value());
    let flash_message = urlencoding::decode(flash_cookie.value()).unwrap();
    assert!(flash_message.contains("Incorrect Credentials"));

    let response = app
        .client
        .get(app.route_url("/users/login"))
        .send()
        .await
        .unwrap();

    let response_body = response.text().await.unwrap();
    println!("{response_body}");
    assert!(response_body.contains("Incorrect Credentials"));
}
