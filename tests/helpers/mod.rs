#![allow(dead_code)]
use reqwest::Response;
use serde::Serialize;
use sqlx::PgPool;
use std::net::SocketAddr;
use todo_web_app::{get_config, serve_app};
use tokio::net::TcpListener;

pub struct TestApp {
    pub client: reqwest::Client,
    pub address: SocketAddr,
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
}
