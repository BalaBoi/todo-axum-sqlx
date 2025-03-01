#![allow(dead_code)]
use std::future::IntoFuture;
use std::net::SocketAddr;

use reqwest::Response;
use secrecy::SecretString;
use serde::Serialize;
use sqlx::PgPool;
use todo_web_app::{
    api_router,
    utilities::{ApiState, HmacKey},
};
use tokio::net::TcpListener;
use uuid::Uuid;

pub struct TestApp {
    pub client: reqwest::Client,
    pub address: SocketAddr,
    pub hmac_key: String,
}

impl TestApp {
    pub async fn new(pool: PgPool) -> Self {
        let test_hmac_key = Uuid::new_v4().to_string();
        let state = ApiState {
            pool,
            hmac_key: HmacKey(SecretString::from(test_hmac_key.as_str())),
        };

        let listener = TcpListener::bind("localhost:0")
            .await
            .expect("should be able to bind to a free port on localhost");
        let addr = listener
            .local_addr()
            .expect("should be able to get the local address");

        let _ = tokio::spawn(axum::serve(listener, api_router(state)).into_future());

        Self {
            client: reqwest::ClientBuilder::new()
                .redirect(reqwest::redirect::Policy::none())
                .cookie_store(true)
                .build()
                .expect("should be able to build client"),
            address: addr,
            hmac_key: test_hmac_key,
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
}
