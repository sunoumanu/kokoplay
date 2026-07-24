use thiserror::Error;

#[derive(Debug, Error)]
pub enum KokoroError {
    #[error("request to kokoro backend failed: {0}")]
    Request(#[from] reqwest::Error),
}

pub struct KokoroClient {
    http: reqwest::blocking::Client,
    url: String,
    voice: String,
}

impl KokoroClient {
    pub fn new(url: String, voice: String) -> Self {
        Self {
            http: reqwest::blocking::Client::new(),
            url,
            voice,
        }
    }

    pub fn synthesize(&self, text: &str) -> Result<Vec<u8>, KokoroError> {
        let body = serde_json::json!({
            "model": "kokoro",
            "input": text,
            "voice": self.voice,
            "response_format": "wav",
        });
        let resp = self
            .http
            .post(&self.url)
            .json(&body)
            .send()?
            .error_for_status()?;
        Ok(resp.bytes()?.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::post, Json, Router};
    use serde_json::Value;

    async fn ok_handler(Json(body): Json<Value>) -> Vec<u8> {
        assert_eq!(body["model"], "kokoro");
        assert_eq!(body["input"], "hello");
        assert_eq!(body["voice"], "af_heart");
        assert_eq!(body["response_format"], "wav");
        vec![1, 2, 3, 4]
    }

    async fn fail_handler() -> axum::http::StatusCode {
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    }

    async fn spawn_test_server(router: Router) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        format!("http://{addr}")
    }

    #[tokio::test]
    async fn synthesize_returns_bytes_on_success() {
        let base = spawn_test_server(Router::new().route("/speech", post(ok_handler))).await;
        let client = KokoroClient::new(format!("{base}/speech"), "af_heart".to_string());
        let result =
            tokio::task::spawn_blocking(move || client.synthesize("hello")).await.unwrap();
        assert_eq!(result.unwrap(), vec![1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn synthesize_errors_on_non_2xx() {
        let base = spawn_test_server(Router::new().route("/speech", post(fail_handler))).await;
        let client = KokoroClient::new(format!("{base}/speech"), "af_heart".to_string());
        let result =
            tokio::task::spawn_blocking(move || client.synthesize("hello")).await.unwrap();
        assert!(result.is_err());
    }
}
