use axum::{
    body::Body,
    extract::Path,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

pub async fn download_binary(Path(filename): Path<String>) -> Response {
    // Only allow downloading terma binaries
    if !filename.starts_with("terma-") {
        return (StatusCode::NOT_FOUND, "Binary not found").into_response();
    }

    // Check if GITHUB_REPO is set for production (redirect to GitHub Releases)
    if let Ok(github_repo) = std::env::var("GITHUB_REPO") {
        let github_url = format!(
            "https://github.com/{}/releases/latest/download/{}",
            github_repo, filename
        );

        return (
            StatusCode::TEMPORARY_REDIRECT,
            [(header::LOCATION, github_url.as_str())],
        )
            .into_response();
    }

    // Development mode: serve from local build
    let binary_path = std::path::PathBuf::from("target/release/terma");

    if !binary_path.exists() {
        return (StatusCode::NOT_FOUND, "Binary not found. Run `cargo build --release -p terma-client`").into_response();
    }

    let file = match File::open(&binary_path).await {
        Ok(file) => file,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to open binary").into_response(),
    };

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/octet-stream"),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"terma\""),
        ],
        body,
    )
        .into_response()
}
