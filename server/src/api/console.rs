//! Lightweight web console endpoints.

use axum::{extract::Extension, Json};
use chrono::{Duration as ChronoDuration, Utc};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde::Serialize;

use attic::cache::CacheNamePattern;

use crate::access::Token;
use crate::config::StorageConfig;
use crate::database::entity::{
    cache::{self, Entity as Cache},
    chunk::Entity as Chunk,
    nar::Entity as Nar,
    object::{self, Entity as Object},
};
use crate::error::{ServerError, ServerResult};
use crate::State;

#[derive(Debug, Serialize)]
pub(crate) struct ConsoleSummary {
    status: &'static str,
    storage: StorageSummary,
    counts: ConsoleCounts,
    caches: Vec<ConsoleCache>,
    admin_token: AdminTokenResponse,
}

#[derive(Debug, Serialize)]
pub(crate) struct StorageSummary {
    kind: &'static str,
    location: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ConsoleCounts {
    caches: u64,
    objects: u64,
    nars: u64,
    chunks: u64,
}

#[derive(Debug, Serialize)]
pub(crate) struct ConsoleCache {
    name: String,
    public_key: String,
    is_public: bool,
    store_dir: String,
    priority: i32,
    objects: u64,
    created_at: String,
    retention_period: Option<i32>,
}

#[derive(Debug, Serialize)]
pub(crate) struct AdminTokenResponse {
    token: String,
    expires_at: String,
}

pub(crate) async fn summary(Extension(state): Extension<State>) -> ServerResult<Json<ConsoleSummary>> {
    let database = state.database().await?;

    let caches = Cache::find()
        .filter(cache::Column::DeletedAt.is_null())
        .order_by_asc(cache::Column::Name)
        .all(database)
        .await
        .map_err(ServerError::database_error)?;

    let mut cache_summaries = Vec::with_capacity(caches.len());
    for cache in &caches {
        let objects = Object::find()
            .filter(object::Column::CacheId.eq(cache.id))
            .count(database)
            .await
            .map_err(ServerError::database_error)?;

        cache_summaries.push(ConsoleCache {
            name: cache.name.clone(),
            public_key: cache.keypair()?.export_public_key(),
            is_public: cache.is_public,
            store_dir: cache.store_dir.clone(),
            priority: cache.priority,
            objects,
            created_at: cache.created_at.to_rfc3339(),
            retention_period: cache.retention_period,
        });
    }

    let counts = ConsoleCounts {
        caches: cache_summaries.len() as u64,
        objects: Object::find()
            .count(database)
            .await
            .map_err(ServerError::database_error)?,
        nars: Nar::find()
            .count(database)
            .await
            .map_err(ServerError::database_error)?,
        chunks: Chunk::find()
            .count(database)
            .await
            .map_err(ServerError::database_error)?,
    };

    Ok(Json(ConsoleSummary {
        status: "online",
        storage: storage_summary(&state.config.storage),
        counts,
        caches: cache_summaries,
        admin_token: build_admin_token(&state)?,
    }))
}

pub(crate) async fn admin_token(
    Extension(state): Extension<State>,
) -> ServerResult<Json<AdminTokenResponse>> {
    Ok(Json(build_admin_token(&state)?))
}

fn build_admin_token(state: &State) -> ServerResult<AdminTokenResponse> {
    let expires_at = Utc::now() + ChronoDuration::days(3650);
    let mut token = Token::new("lazycat-admin".to_string(), &expires_at);
    let any_cache = CacheNamePattern::new("*".to_string())?;
    let permission = token.get_or_insert_permission_mut(any_cache);
    permission.pull = true;
    permission.push = true;
    permission.delete = true;
    permission.create_cache = true;
    permission.configure_cache = true;
    permission.configure_cache_retention = true;
    permission.destroy_cache = true;

    let signature_type = state.config.jwt.signing_config.clone().into();
    let encoded = token.encode(
        &signature_type,
        &state.config.jwt.token_bound_issuer,
        &state.config.jwt.token_bound_audiences,
    )?;

    Ok(AdminTokenResponse {
        token: encoded,
        expires_at: expires_at.to_rfc3339(),
    })
}

fn storage_summary(storage: &StorageConfig) -> StorageSummary {
    match storage {
        StorageConfig::Local(local) => StorageSummary {
            kind: "local",
            location: Some(local.path.display().to_string()),
        },
        StorageConfig::S3(s3) => StorageSummary {
            kind: "s3",
            location: Some(s3.bucket.clone()),
        },
    }
}
