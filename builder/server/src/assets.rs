//! Serves the compiled Angular SPA, embedded into this binary via `rust-embed`.
//! Only the server links these assets — the CLI never does.
//!
//! The frontend is built to `frontend/dist/ubihome-builder/browser` (Angular's
//! default application-builder layout). If the dir is absent at compile time
//! (e.g. building the server without first building the frontend), the embed is
//! empty and we serve a small placeholder so the binary still runs.

use axum::body::Body;
use axum::http::{header, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../frontend/dist/ubihome-builder/browser"]
struct Assets;

const PLACEHOLDER: &str = r#"<!doctype html><html><head><meta charset="utf-8">
<title>UbiHome Builder</title></head><body style="font-family:sans-serif;padding:2rem">
<h1>UbiHome Builder</h1>
<p>The web UI bundle was not embedded in this build. The REST API is available under
<code>/api</code>. Build the frontend (<code>npm --prefix frontend ci &amp;&amp; npm --prefix frontend run build</code>)
and rebuild the server to get the dashboard.</p></body></html>"#;

/// SPA handler: serve the requested asset, or fall back to `index.html` so that
/// client-side routes (e.g. `/history`) resolve.
pub async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    if let Some(content) = Assets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response();
    }

    // SPA fallback to index.html.
    match Assets::get("index.html") {
        Some(content) => ([(header::CONTENT_TYPE, "text/html")], content.data).into_response(),
        None => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html")],
            Body::from(PLACEHOLDER),
        )
            .into_response(),
    }
}
