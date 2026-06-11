//! Lightweight web console endpoints.

use axum::{
    extract::{Extension, Path, Query},
    Json,
};
use chrono::{Duration as ChronoDuration, Utc};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::Deserialize;
use serde::Serialize;

use attic::cache::CacheNamePattern;

use crate::access::Token;
use crate::config::StorageConfig;
use crate::database::entity::{
    cache::{self, Entity as Cache},
    chunk::Entity as Chunk,
    nar::{Entity as Nar, Model as NarModel},
    object::{self, Entity as Object},
};
use crate::error::{ErrorKind, ServerError, ServerResult};
use crate::{RequestState, State};

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
    upstream_cache_key_names: Vec<String>,
    api_endpoint: String,
    substituter_endpoint: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct AdminTokenResponse {
    token: String,
    expires_at: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ObjectListParams {
    limit: Option<u64>,
    offset: Option<u64>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ConsoleObjectList {
    cache: ConsoleCache,
    total: u64,
    limit: u64,
    offset: u64,
    objects: Vec<ConsoleObject>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ConsoleObjectDetail {
    cache: ConsoleCache,
    object: ConsoleObject,
}

#[derive(Debug, Serialize)]
pub(crate) struct ConsoleObject {
    store_path_hash: String,
    store_path: String,
    system: Option<String>,
    deriver: Option<String>,
    created_at: String,
    last_accessed_at: Option<String>,
    created_by: Option<String>,
    references: Vec<String>,
    sigs: Vec<String>,
    ca: Option<String>,
    nar: ConsoleNar,
    narinfo_url: String,
    nar_url: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ConsoleNar {
    nar_hash: String,
    nar_size: i64,
    compression: String,
    num_chunks: i32,
    completeness_hint: bool,
    holders_count: i32,
    created_at: String,
}

pub(crate) async fn summary(
    Extension(state): Extension<State>,
    Extension(req_state): Extension<RequestState>,
) -> ServerResult<Json<ConsoleSummary>> {
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

        cache_summaries.push(console_cache(cache, objects, &req_state)?);
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
        admin_token: current_admin_token(&state).await?,
    }))
}

pub(crate) async fn list_objects(
    Extension(state): Extension<State>,
    Extension(req_state): Extension<RequestState>,
    Path(cache_name): Path<String>,
    Query(params): Query<ObjectListParams>,
) -> ServerResult<Json<ConsoleObjectList>> {
    let database = state.database().await?;
    let cache = Cache::find()
        .filter(cache::Column::Name.eq(&cache_name))
        .filter(cache::Column::DeletedAt.is_null())
        .one(database)
        .await
        .map_err(ServerError::database_error)?
        .ok_or_else(|| ErrorKind::NoSuchCache)?;

    let total = Object::find()
        .filter(object::Column::CacheId.eq(cache.id))
        .count(database)
        .await
        .map_err(ServerError::database_error)?;
    let limit = params.limit.unwrap_or(20).clamp(1, 100);
    let offset = params.offset.unwrap_or(0);

    let rows = Object::find()
        .filter(object::Column::CacheId.eq(cache.id))
        .order_by_desc(object::Column::CreatedAt)
        .offset(offset)
        .limit(limit)
        .find_also_related(Nar)
        .all(database)
        .await
        .map_err(ServerError::database_error)?;

    let cache_summary = console_cache(&cache, total, &req_state)?;
    let objects = rows
        .into_iter()
        .filter_map(|(object, nar)| nar.map(|nar| console_object(&cache_summary, object, nar)))
        .collect();

    Ok(Json(ConsoleObjectList {
        cache: cache_summary,
        total,
        limit,
        offset,
        objects,
    }))
}

pub(crate) async fn object_detail(
    Extension(state): Extension<State>,
    Extension(req_state): Extension<RequestState>,
    Path((cache_name, store_path_hash)): Path<(String, String)>,
) -> ServerResult<Json<ConsoleObjectDetail>> {
    let database = state.database().await?;
    let cache = Cache::find()
        .filter(cache::Column::Name.eq(&cache_name))
        .filter(cache::Column::DeletedAt.is_null())
        .one(database)
        .await
        .map_err(ServerError::database_error)?
        .ok_or_else(|| ErrorKind::NoSuchCache)?;

    let total = Object::find()
        .filter(object::Column::CacheId.eq(cache.id))
        .count(database)
        .await
        .map_err(ServerError::database_error)?;
    let cache_summary = console_cache(&cache, total, &req_state)?;

    let (object, nar) = Object::find()
        .filter(object::Column::CacheId.eq(cache.id))
        .filter(object::Column::StorePathHash.eq(&store_path_hash))
        .find_also_related(Nar)
        .one(database)
        .await
        .map_err(ServerError::database_error)?
        .and_then(|(object, nar)| nar.map(|nar| (object, nar)))
        .ok_or_else(|| ErrorKind::NoSuchObject)?;

    Ok(Json(ConsoleObjectDetail {
        object: console_object(&cache_summary, object, nar),
        cache: cache_summary,
    }))
}

pub(crate) async fn admin_token(
    Extension(state): Extension<State>,
) -> ServerResult<Json<AdminTokenResponse>> {
    Ok(Json(regenerate_admin_token(&state).await?))
}

async fn current_admin_token(state: &State) -> ServerResult<AdminTokenResponse> {
    let mut admin_token = state.console_admin_token.lock().await;
    if let Some((token, expires_at)) = admin_token.as_ref() {
        return Ok(AdminTokenResponse {
            token: token.clone(),
            expires_at: expires_at.clone(),
        });
    }

    let token = build_admin_token(state)?;
    *admin_token = Some((token.token.clone(), token.expires_at.clone()));
    Ok(token)
}

async fn regenerate_admin_token(state: &State) -> ServerResult<AdminTokenResponse> {
    let token = build_admin_token(state)?;
    let mut admin_token = state.console_admin_token.lock().await;
    *admin_token = Some((token.token.clone(), token.expires_at.clone()));
    Ok(token)
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

fn console_cache(
    cache: &cache::Model,
    objects: u64,
    req_state: &RequestState,
) -> ServerResult<ConsoleCache> {
    let name = attic::cache::CacheName::new(cache.name.clone())?;

    Ok(ConsoleCache {
        name: cache.name.clone(),
        public_key: cache.keypair()?.export_public_key(),
        is_public: cache.is_public,
        store_dir: cache.store_dir.clone(),
        priority: cache.priority,
        objects,
        created_at: cache.created_at.to_rfc3339(),
        retention_period: cache.retention_period,
        upstream_cache_key_names: cache.upstream_cache_key_names.0.clone(),
        api_endpoint: req_state.api_endpoint()?,
        substituter_endpoint: req_state.substituter_endpoint(name)?,
    })
}

fn console_object(cache: &ConsoleCache, object: object::Model, nar: NarModel) -> ConsoleObject {
    ConsoleObject {
        narinfo_url: format!(
            "{}/{}.narinfo",
            cache.substituter_endpoint, object.store_path_hash
        ),
        nar_url: format!(
            "{}/nar/{}.nar",
            cache.substituter_endpoint, object.store_path_hash
        ),
        store_path_hash: object.store_path_hash,
        store_path: object.store_path,
        system: object.system,
        deriver: object.deriver,
        created_at: object.created_at.to_rfc3339(),
        last_accessed_at: object.last_accessed_at.map(|value| value.to_rfc3339()),
        created_by: object.created_by,
        references: object.references.0,
        sigs: object.sigs.0,
        ca: object.ca,
        nar: ConsoleNar {
            nar_hash: nar.nar_hash,
            nar_size: nar.nar_size,
            compression: nar.compression,
            num_chunks: nar.num_chunks,
            completeness_hint: nar.completeness_hint,
            holders_count: nar.holders_count,
            created_at: nar.created_at.to_rfc3339(),
        },
    }
}
