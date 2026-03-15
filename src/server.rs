use axum::{Json, Router, http::StatusCode, routing::get, routing::post};
use serde::{Deserialize, Serialize};
use tokio;
use xplm::debugln;

use crate::{
    config::ServerConfig,
    dataref::{RefValue, get_ref_value},
};

pub struct Server {
    pub port: u16,
    pub address: String,
}

#[derive(Serialize)]
struct GenericResponse {
    pub status: u16,
    pub message: String,
}

#[derive(Deserialize)]
struct GetRefValueRequest {
    ref_name: String,
}

#[derive(Serialize)]
struct GetRefValueResponse {
    pub status: u16,
    pub message: String,
    pub ref_name: String,
    pub ref_value: RefValue,
}

impl Server {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            port: config.port,
            address: config.address,
        }
    }

    pub async fn start(&self) {
        debugln!("XPHTTPBridge: Creating app");

        let app = Router::new()
            .route("/api/v1/healthz", get(Server::health_handler))
            .route("/api/v1/refs/value", post(Server::get_ref_handler));

        let listener_res =
            tokio::net::TcpListener::bind(format!("{}:{}", self.address, self.port)).await;
        let listener = match listener_res {
            Ok(l) => l,
            Err(e) => {
                debugln!("XPHTTPBridge: Failed to bind listener: {}", e);
                return;
            }
        };

        debugln!("XPHTTPBridge: Listening on {}:{}", self.address, self.port);

        let serve_res = axum::serve(listener, app).await;
        match serve_res {
            Ok(_) => {}
            Err(e) => {
                debugln!("XPHTTPBridge: Failed to start server: {}", e);
            }
        }
    }

    async fn health_handler() -> (StatusCode, Json<GenericResponse>) {
        (
            StatusCode::OK,
            Json(GenericResponse {
                status: 200,
                message: "OK".to_string(),
            }),
        )
    }

    async fn get_ref_handler(
        Json(request): Json<GetRefValueRequest>,
    ) -> (StatusCode, Json<GetRefValueResponse>) {
        let ref_value = get_ref_value(&request.ref_name);

        if let Some(ref_value) = ref_value {
            (
                StatusCode::OK,
                Json(GetRefValueResponse {
                    status: 200,
                    message: "OK".to_string(),
                    ref_name: request.ref_name,
                    ref_value,
                }),
            )
        } else {
            (
                StatusCode::NOT_FOUND,
                Json(GetRefValueResponse {
                    status: 404,
                    message: "ref not found".to_string(),
                    ref_name: request.ref_name,
                    ref_value: RefValue::F32(0.0),
                }),
            )
        }
    }
}
