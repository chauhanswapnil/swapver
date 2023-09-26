use axum::{routing::get, Router};
use std::net::SocketAddr;
use std::fs;
use std::process::Command;
use axum::http::StatusCode;
use axum::routing::post;

#[tokio::main]
async fn main() {

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app().into_make_service())
        .await
        .unwrap();
}

/// Having a function that produces our app makes it easy to call it from tests
/// without having to create an HTTP server.
#[allow(dead_code)]
fn app() -> Router {
    Router::new()
        .route("/", get(handler_get))
        .route("/lox_java", post(handler_post))
}

async fn handler_get() -> &'static str {
    "Hello, World!"
}

async fn handler_post(body: String) -> (StatusCode, String) {
    match execute_command(body).await {
        Ok(output) => (StatusCode::OK, output),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err),
    }
}

async fn execute_command(code: String) -> Result<String, String> {
    fs::write("./program.txt", code).map_err(|e| e.to_string())?;

    let output = Command::new("java")
        .current_dir("../../Slox/java")
        .arg("com.slox.lox.Lox")
        .arg("../../program.txt")
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        String::from_utf8(output.stdout).map_err(|e| e.to_string())
    } else {
        String::from_utf8(output.stderr).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::util::ServiceExt; // for `oneshot`

    #[tokio::test]
    async fn hello_world() {
        let app = app();

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"Hello, World!");
    }
}