//! HTTP API for Claude integration

use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use image::GenericImageView;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use crate::diff;
use crate::state::{AppState, ParamDef};

type SharedState = Arc<RwLock<AppState>>;

/// Run the API server
pub async fn run_server(state: SharedState, port: u16) -> Result<()> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // State & info
        .route("/state", get(get_state))
        .route("/params", get(get_params).post(set_params))
        .route("/metrics", get(get_metrics))
        // Shader
        .route("/shader", post(load_shader))
        // Images
        .route("/reference", get(get_reference).post(load_reference))
        .route("/input", post(load_input))
        // Screenshots
        .route("/screenshot", get(screenshot))
        .route("/screenshot/ui", get(screenshot_ui))
        // Comparison
        .route("/diff", get(get_diff))
        .layer(cors)
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("API listening on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}

// === Response Types ===

#[derive(Serialize)]
struct StateResponse {
    shader_name: String,
    shader_code: String,
    status: String,
    fps: f32,
    has_reference: bool,
    params: HashMap<String, f32>,
    errors: Option<String>,
}

#[derive(Serialize)]
struct ParamsResponse {
    params: HashMap<String, ParamDef>,
}

#[derive(Serialize)]
struct MetricsResponse {
    fps: f32,
    frame_time_ms: f32,
    gpu_time_ms: f32,
    shader_compile_ms: f32,
}

#[derive(Serialize)]
struct SuccessResponse {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    compile_time_ms: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<u32>,
}

#[derive(Serialize)]
struct ErrorResponse {
    status: String,
    error: String,
}

#[derive(Serialize)]
struct ImageResponse {
    image_data: String, // base64
}

#[derive(Serialize)]
struct DiffResponse {
    similarity: f32,
    mae: f32,
    peak_error: f32,
    diff_regions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    diff_image: Option<String>, // base64
}

// === Request Types ===

#[derive(Deserialize)]
struct ShaderRequest {
    code: String,
    #[serde(default)]
    name: String,
}

#[derive(Deserialize)]
struct ImageRequest {
    image: String, // base64
    #[serde(default)]
    path: String,
}

// === Handlers ===

async fn get_state(State(state): State<SharedState>) -> Json<StateResponse> {
    let s = state.read();

    let params: HashMap<String, f32> = s
        .params
        .iter()
        .map(|(k, v)| (k.clone(), v.value))
        .collect();

    Json(StateResponse {
        shader_name: s.shader_name.clone(),
        shader_code: s.shader_code.clone(),
        status: s.status.clone(),
        fps: s.fps,
        has_reference: s.reference_image.is_some(),
        params,
        errors: s.shader_error.clone(),
    })
}

async fn get_params(State(state): State<SharedState>) -> Json<ParamsResponse> {
    let s = state.read();
    Json(ParamsResponse {
        params: s.params.clone(),
    })
}

async fn set_params(
    State(state): State<SharedState>,
    Json(params): Json<HashMap<String, f32>>,
) -> Json<SuccessResponse> {
    let mut s = state.write();

    for (name, value) in params {
        s.set_param(&name, value);
    }

    Json(SuccessResponse {
        status: "ok".to_string(),
        compile_time_ms: None,
        width: None,
        height: None,
    })
}

async fn get_metrics(State(state): State<SharedState>) -> Json<MetricsResponse> {
    let s = state.read();
    Json(MetricsResponse {
        fps: s.fps,
        frame_time_ms: s.frame_time_ms,
        gpu_time_ms: s.gpu_time_ms,
        shader_compile_ms: s.shader_compile_ms,
    })
}

async fn load_shader(
    State(state): State<SharedState>,
    Json(req): Json<ShaderRequest>,
) -> Response {
    let mut s = state.write();

    let name = if req.name.is_empty() {
        "unnamed".to_string()
    } else {
        req.name
    };

    s.set_shader(name, req.code);

    // Note: actual compilation happens in render loop
    // We'll check for errors on next state query

    Json(SuccessResponse {
        status: "ok".to_string(),
        compile_time_ms: Some(0.0), // Will be updated after compile
        width: None,
        height: None,
    })
    .into_response()
}

async fn get_reference(State(state): State<SharedState>) -> Response {
    let s = state.read();

    match &s.reference_image {
        Some(data) => {
            let encoded = BASE64.encode(data);
            Json(ImageResponse { image_data: encoded }).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                status: "error".to_string(),
                error: "No reference image loaded".to_string(),
            }),
        )
            .into_response(),
    }
}

async fn load_reference(
    State(state): State<SharedState>,
    Json(req): Json<ImageRequest>,
) -> Response {
    match BASE64.decode(&req.image) {
        Ok(data) => {
            // Decode to get dimensions
            match image::load_from_memory(&data) {
                Ok(img) => {
                    let (width, height) = img.dimensions();

                    let mut s = state.write();
                    s.reference_image = Some(data);

                    Json(SuccessResponse {
                        status: "ok".to_string(),
                        compile_time_ms: None,
                        width: Some(width),
                        height: Some(height),
                    })
                    .into_response()
                }
                Err(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        status: "error".to_string(),
                        error: format!("Invalid image: {}", e),
                    }),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                status: "error".to_string(),
                error: format!("Invalid base64: {}", e),
            }),
        )
            .into_response(),
    }
}

async fn load_input(State(state): State<SharedState>, Json(req): Json<ImageRequest>) -> Response {
    match BASE64.decode(&req.image) {
        Ok(data) => {
            let mut s = state.write();
            s.input_image = Some(data);

            Json(SuccessResponse {
                status: "ok".to_string(),
                compile_time_ms: None,
                width: None,
                height: None,
            })
            .into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                status: "error".to_string(),
                error: format!("Invalid base64: {}", e),
            }),
        )
            .into_response(),
    }
}

async fn screenshot(State(state): State<SharedState>) -> Response {
    let s = state.read();

    match &s.output_image {
        Some(data) => {
            let encoded = BASE64.encode(data);
            Json(ImageResponse { image_data: encoded }).into_response()
        }
        None => {
            // Return a placeholder or error
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    status: "error".to_string(),
                    error: "No frame rendered yet".to_string(),
                }),
            )
                .into_response()
        }
    }
}

async fn screenshot_ui(State(state): State<SharedState>) -> Response {
    // For now, same as regular screenshot
    // TODO: Capture with egui overlay
    screenshot(State(state)).await
}

async fn get_diff(State(state): State<SharedState>) -> Response {
    let s = state.read();

    let (output, reference) = match (&s.output_image, &s.reference_image) {
        (Some(o), Some(r)) => (o, r),
        (None, _) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    status: "error".to_string(),
                    error: "No output image".to_string(),
                }),
            )
                .into_response()
        }
        (_, None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    status: "error".to_string(),
                    error: "No reference image loaded".to_string(),
                }),
            )
                .into_response()
        }
    };

    match diff::compare_images(output, reference) {
        Ok(result) => Json(DiffResponse {
            similarity: result.similarity,
            mae: result.mae,
            peak_error: result.peak_error,
            diff_regions: result.diff_regions,
            diff_image: result.diff_image.map(|d| BASE64.encode(&d)),
        })
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: "error".to_string(),
                error: format!("Diff failed: {}", e),
            }),
        )
            .into_response(),
    }
}
