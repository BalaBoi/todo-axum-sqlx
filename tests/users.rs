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

#[test]
async fn must_be_logged_in_to_access_todo_page(pool: PgPool) {
    let app = TestApp::new(pool).await;

    let response = app.get_todo().await;

    //assert that you get redirected back to login
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
}
