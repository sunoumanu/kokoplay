use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AppState {
    pub tx: Sender<String>,
    pub queue_len: Arc<AtomicUsize>,
}

#[derive(Deserialize)]
struct SpeakRequest {
    text: String,
}

#[derive(Serialize)]
struct SpeakResponse {
    status: &'static str,
    queue_length: usize,
}

#[derive(Serialize)]
struct ErrorBody {
    detail: String,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/speak", post(speak_handler))
        .route("/health", get(health_handler))
        .with_state(state)
}

async fn speak_handler(
    State(state): State<AppState>,
    Json(req): Json<SpeakRequest>,
) -> Result<(StatusCode, Json<SpeakResponse>), (StatusCode, Json<ErrorBody>)> {
    if req.text.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorBody {
                detail: "text must not be empty".to_string(),
            }),
        ));
    }

    if state.tx.send(req.text).is_err() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody {
                detail: "worker unavailable".to_string(),
            }),
        ));
    }

    let queue_length = state.queue_len.fetch_add(1, Ordering::SeqCst) + 1;
    Ok((
        StatusCode::ACCEPTED,
        Json(SpeakResponse {
            status: "queued",
            queue_length,
        }),
    ))
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use serde_json::Value;
    use std::sync::mpsc::channel;
    use tower::ServiceExt;

    fn test_state() -> AppState {
        let (tx, rx) = channel::<String>();
        std::mem::forget(rx);
        AppState {
            tx,
            queue_len: Arc::new(AtomicUsize::new(0)),
        }
    }

    async fn body_json(resp: axum::response::Response) -> Value {
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let app = build_router(test_state());
        let resp = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(body_json(resp).await, serde_json::json!({"status": "ok"}));
    }

    #[tokio::test]
    async fn speak_queues_and_increments_queue_length() {
        let app = build_router(test_state());

        let req = |text: &str| {
            Request::builder()
                .method("POST")
                .uri("/speak")
                .header("content-type", "application/json")
                .body(Body::from(format!("{{\"text\": \"{text}\"}}")))
                .unwrap()
        };

        let resp1 = app.clone().oneshot(req("hello")).await.unwrap();
        assert_eq!(resp1.status(), StatusCode::ACCEPTED);
        assert_eq!(
            body_json(resp1).await,
            serde_json::json!({"status": "queued", "queue_length": 1})
        );

        let resp2 = app.clone().oneshot(req("world")).await.unwrap();
        assert_eq!(resp2.status(), StatusCode::ACCEPTED);
        assert_eq!(
            body_json(resp2).await,
            serde_json::json!({"status": "queued", "queue_length": 2})
        );
    }

    #[tokio::test]
    async fn speak_rejects_empty_text() {
        let app = build_router(test_state());
        let req = Request::builder()
            .method("POST")
            .uri("/speak")
            .header("content-type", "application/json")
            .body(Body::from("{\"text\": \"   \"}"))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            body_json(resp).await,
            serde_json::json!({"detail": "text must not be empty"})
        );
    }
}
