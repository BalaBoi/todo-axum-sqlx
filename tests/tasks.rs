use std::collections::HashMap;

use axum::http::HeaderValue;
use helpers::TestApp;
use reqwest::StatusCode;
use sqlx::{test, PgPool};
use uuid::Uuid;

mod helpers;

#[test]
async fn creating_a_task_redirects_to_todo_with_the_task(pool: PgPool) {
    let mut app = TestApp::new(pool).await;
    let test_user = app.register_test_user().await;
    app.login_test_user(&test_user).await;

    let mut task_create_body = HashMap::new();
    let title = Uuid::new_v4().to_string();
    let description = Uuid::new_v4().to_string();
    task_create_body.insert("title", title.as_str());
    task_create_body.insert("description", &description);

    //send a create task form to the backend
    let response = app
        .client
        .post(app.route_url("/todo"))
        .form(&task_create_body)
        .send()
        .await
        .expect("couldn't send request");

    //assert it's a redirect to the /todo page
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get("location").unwrap(),
        HeaderValue::from_str("/todo").unwrap()
    );

    let response = app
        .client
        .get(app.route_url("/todo"))
        .send()
        .await
        .expect("couldn't send request");

    //assert the redirected page contains the title and the description
    assert_eq!(response.status(), StatusCode::OK);
    let text = response.text().await.unwrap();
    assert_eq!(text.contains(&title), true);
    assert_eq!(text.contains(&description), true);
}
