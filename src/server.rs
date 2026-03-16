use axum::{
    Json, Router, extract::Query, extract::State, http::StatusCode, routing::get, routing::post,
};
use serde::{Deserialize, Serialize};
use tokio;
use xplm::debugln;

use crate::{
    config::ServerConfig,
    dataref::{
        DataRefInfo, RefValue, RefValues, get_ref_value, get_ref_values, set_ref_value,
        set_ref_values,
    },
};

#[derive(Clone)]
pub struct Server {
    pub port: u16,
    pub address: String,
    pub data_ref_info: Vec<DataRefInfo>,
}

#[derive(Clone)]
struct ServerState {
    server: Server,
}

#[derive(Serialize)]
struct GenericResponse {
    pub status: u16,
    pub message: String,
}

#[derive(Deserialize)]
struct GetRefValueRequestParams {
    ref_name: String,
}

#[derive(Serialize)]
struct GetRefValueResponse {
    pub status: u16,
    pub message: String,
    pub ref_name: String,
    pub ref_value: Option<RefValue>,
}

#[derive(Deserialize)]
struct SetRefValueRequestParams {
    ref_name: String,
}

#[derive(Deserialize)]
struct SetRefValueRequestBody {
    ref_value: RefValue,
}

#[derive(Serialize)]
struct SetRefValueResponse {
    pub status: u16,
    pub message: String,
    pub ref_name: String,
    pub ref_value: RefValue,
}

#[derive(Deserialize)]
struct GetRefValuesRequestParams {
    ref_name: String,
}

#[derive(Serialize)]
struct GetRefValuesResponse {
    pub status: u16,
    pub message: String,
    pub ref_name: String,
    pub ref_values: Option<RefValues>,
}

#[derive(Deserialize)]
struct SetRefValuesRequestParams {
    ref_name: String,
}

#[derive(Deserialize)]
struct SetRefValuesRequestBody {
    ref_values: RefValues,
}

#[derive(Serialize)]
struct SetRefValuesResponse {
    pub status: u16,
    pub message: String,
    pub ref_name: String,
    pub ref_values: RefValues,
}

#[derive(Deserialize)]
struct GetAllRefsRequestParams {
    // 4 billion should be enough for all datarefs
    fetch_size: Option<u32>,
}

#[derive(Serialize)]
struct GetAllRefsResponse {
    pub status: u16,
    pub message: String,
    pub refs: Vec<DataRefInfo>,
}

impl Server {
    pub fn new(config: ServerConfig, data_ref_info: Vec<DataRefInfo>) -> Self {
        Self {
            port: config.port,
            address: config.address,
            data_ref_info,
        }
    }

    pub async fn start(self) {
        debugln!("XPHTTPBridge: Creating app");

        let state = ServerState {
            server: self.clone(),
        };

        let app = Router::new()
            .route("/api/v1/healthz", get(Server::health_handler))
            .route("/api/v1/refs/value/get", get(Server::get_ref_handler))
            .route("/api/v1/refs/value/set", post(Server::set_ref_handler))
            .route(
                "/api/v1/refs/values/get",
                get(Server::get_ref_multiple_handler),
            )
            .route(
                "/api/v1/refs/values/set",
                post(Server::set_ref_multiple_handler),
            )
            .route("/api/v1/refs/all", get(Server::get_all_refs_handler))
            .fallback(Server::fallback_handler)
            .with_state(state);

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

    async fn fallback_handler() -> (StatusCode, Json<GenericResponse>) {
        (
            StatusCode::NOT_FOUND,
            Json(GenericResponse {
                status: 404,
                message: "not found".to_string(),
            }),
        )
    }

    async fn get_ref_handler(
        params: Query<GetRefValueRequestParams>,
    ) -> (StatusCode, Json<GetRefValueResponse>) {
        let params: GetRefValueRequestParams = params.0;
        let ref_name = params.ref_name;
        let ref_value = get_ref_value(&ref_name);

        if let Some(ref_value) = ref_value {
            (
                StatusCode::OK,
                Json(GetRefValueResponse {
                    status: 200,
                    message: "OK".to_string(),
                    ref_name: ref_name,
                    ref_value: Some(ref_value),
                }),
            )
        } else {
            (
                StatusCode::NOT_FOUND,
                Json(GetRefValueResponse {
                    status: 404,
                    message: "ref not found".to_string(),
                    ref_name: ref_name,
                    ref_value: None,
                }),
            )
        }
    }

    async fn get_ref_multiple_handler(
        params: Query<GetRefValuesRequestParams>,
    ) -> (StatusCode, Json<GetRefValuesResponse>) {
        let params: GetRefValuesRequestParams = params.0;
        let ref_name = params.ref_name;
        let ref_values = get_ref_values(&ref_name);

        if let Some(ref_values) = ref_values {
            (
                StatusCode::OK,
                Json(GetRefValuesResponse {
                    status: 200,
                    message: "OK".to_string(),
                    ref_name: ref_name,
                    ref_values: Some(ref_values),
                }),
            )
        } else {
            (
                StatusCode::NOT_FOUND,
                Json(GetRefValuesResponse {
                    status: 404,
                    message: "ref not found".to_string(),
                    ref_name: ref_name,
                    ref_values: None,
                }),
            )
        }
    }

    async fn set_ref_handler(
        params: Query<SetRefValueRequestParams>,
        Json(request): Json<SetRefValueRequestBody>,
    ) -> (StatusCode, Json<SetRefValueResponse>) {
        let params: SetRefValueRequestParams = params.0;
        let ref_name = params.ref_name;
        let ok = set_ref_value(&ref_name, request.ref_value.clone());
        let status = if ok { 200 } else { 500 };
        let message = if ok {
            "OK".to_string()
        } else {
            "failed to set ref".to_string()
        };
        (
            StatusCode::from_u16(status).unwrap(),
            Json(SetRefValueResponse {
                status,
                message,
                ref_name: ref_name,
                ref_value: request.ref_value,
            }),
        )
    }

    async fn set_ref_multiple_handler(
        params: Query<SetRefValuesRequestParams>,
        Json(request): Json<SetRefValuesRequestBody>,
    ) -> (StatusCode, Json<SetRefValuesResponse>) {
        let params: SetRefValuesRequestParams = params.0;
        let ref_name = params.ref_name;
        let ok = set_ref_values(&ref_name, request.ref_values.clone());
        let status = if ok { 200 } else { 500 };
        let message = if ok {
            "OK".to_string()
        } else {
            "failed to set ref".to_string()
        };
        (
            StatusCode::from_u16(status).unwrap(),
            Json(SetRefValuesResponse {
                status,
                message,
                ref_name: ref_name,
                ref_values: request.ref_values,
            }),
        )
    }

    async fn get_all_refs_handler(
        params: Query<GetAllRefsRequestParams>,
        State(state): State<ServerState>,
    ) -> (StatusCode, Json<GetAllRefsResponse>) {
        let refs = state.server.data_ref_info.clone();
        (
            StatusCode::from_u16(200).unwrap(),
            Json(GetAllRefsResponse {
                status: 200,
                message: "OK".to_string(),
                refs: refs
                    .into_iter()
                    .take(params.fetch_size.unwrap_or(u32::MAX) as usize)
                    .collect(),
            }),
        )
    }
}
