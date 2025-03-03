#![allow(dead_code)]
use reqwest::{Response, StatusCode};
use serde::Serialize;
use serde_json::json;
use sqlx::PgPool;
use std::net::SocketAddr;
use todo_web_app::{get_config, serve_app};
use tokio::net::TcpListener;
use uuid::Uuid;

pub struct TestApp {
    pub client: reqwest::Client,
    pub address: SocketAddr,
}

pub struct TestUser {
    email: String,
    password: String,
    username: String,
}

impl TestApp {
    pub async fn new(pool: PgPool) -> Self {
        let config = get_config();

        let listener = TcpListener::bind("localhost:0")
            .await
            .expect("should be able to bind to a free port on localhost");
        let addr = listener
            .local_addr()
            .expect("should be able to get the local address");

        let _ = tokio::spawn(serve_app(config, pool, listener));

        Self {
            client: reqwest::ClientBuilder::new()
                .redirect(reqwest::redirect::Policy::none())
                .cookie_store(true)
                .build()
                .expect("should be able to build client"),
            address: addr,
        }
    }

    pub fn route_url(&self, route: &str) -> String {
        format!("http://{}{}", self.address, route)
    }

    pub async fn post_login<B: Serialize>(&self, body: &B) -> Response {
        self.client
            .post(self.route_url("/users/login"))
            .form(body)
            .send()
            .await
            .expect("could not send post request")
    }

    pub async fn get_login(&self) -> Response {
        self.client
            .get(self.route_url("/users/login"))
            .send()
            .await
            .expect("could not send get request")
    }

    pub async fn get_todo(&self) -> Response {
        self.client
            .get(self.route_url("/todo"))
            .send()
            .await
            .expect("could not send get request to /todo")
    }

    pub async fn register_test_user(&mut self) -> TestUser {
        let test_user = TestUser {
            email: Uuid::new_v4().to_string(),
            password: Uuid::new_v4().to_string(),
            username: Uuid::new_v4().to_string(),
        };

        let register_form = json!({
            "email": &test_user.email,
            "password": &test_user.password,
            "username": &test_user.username
        });

        let response = self
            .client
            .post(self.route_url("/users/register"))
            .form(&register_form)
            .send()
            .await
            .expect("could not post registration form");

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

        test_user
    }

    pub async fn login_test_user(&self, test_user: &TestUser) {
        let response = self
            .client
            .post(self.route_url("/users/login"))
            .form(&json!({
                "email": &test_user.email,
                "password": &test_user.password
            }))
            .send()
            .await
            .expect("could not post login form");

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response
                .headers()
                .get("location")
                .unwrap()
                .to_str()
                .unwrap(),
            "/todo"
        );
    }
}
