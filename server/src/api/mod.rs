//! HTTP API.

mod binary_cache;
mod console;
mod v1;

use std::env;
use std::path::PathBuf;

use axum::{
    response::Html,
    routing::{get, patch, post},
    Router,
};
use tower_http::services::{ServeDir, ServeFile};

async fn placeholder() -> Html<&'static str> {
    Html(include_str!("placeholder.html"))
}

fn web_dir() -> Option<PathBuf> {
    env::var_os("ATTIC_WEB_DIR").map(PathBuf::from)
}

pub(crate) fn get_router() -> Router {
    let router = Router::new()
        .merge(binary_cache::get_router())
        .merge(v1::get_router())
        .route("/_api/web/summary", get(console::summary))
        .route("/_api/web/usage", get(console::usage))
        .route("/_api/web/admin-token", post(console::admin_token))
        .route(
            "/_api/web/caches/:cache/objects",
            get(console::list_objects),
        )
        .route(
            "/_api/web/caches/:cache/objects/:store_path_hash",
            get(console::object_detail),
        )
        .route(
            "/_api/web/caches/:cache/config",
            patch(v1::cache_config::configure_cache).delete(v1::cache_config::destroy_cache),
        );

    if let Some(web_dir) = web_dir() {
        let web_index = web_dir.join("index.html");

        router
            .route_service("/", ServeFile::new(web_index.clone()))
            .route_service("/cache", ServeFile::new(web_index.clone()))
            .route_service("/cache/", ServeFile::new(web_index.clone()))
            .route_service("/guide", ServeFile::new(web_index.clone()))
            .route_service("/guide/", ServeFile::new(web_index))
            .nest_service("/_app", ServeDir::new(web_dir.join("_app")))
            .nest_service("/favicon.ico", ServeFile::new(web_dir.join("favicon.ico")))
    } else {
        router.route("/", get(placeholder))
    }
}
