//! Lightweight web console endpoints.

use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};

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
pub(crate) struct ConsoleUsage {
    totals: ConsoleUsageTotals,
    cache_usage: Vec<ConsoleCacheUsage>,
    recent_uploads: Vec<ConsoleObject>,
    recent_accesses: Vec<ConsoleObject>,
    systems: Vec<ConsoleUsageBucket>,
    compressions: Vec<ConsoleUsageBucket>,
    uploaders: Vec<ConsoleUsageBucket>,
    health: ConsoleHealth,
}

#[derive(Debug, Serialize)]
pub(crate) struct ConsoleUsageTotals {
    nar_size: i64,
    logical_nar_size: i64,
    incomplete_objects: u64,
    never_accessed_objects: u64,
}

#[derive(Debug, Serialize)]
pub(crate) struct ConsoleCacheUsage {
    cache: ConsoleCache,
    nar_size: i64,
    incomplete_objects: u64,
    never_accessed_objects: u64,
    last_upload_at: Option<String>,
    last_accessed_at: Option<String>,
    systems: Vec<ConsoleUsageBucket>,
    compressions: Vec<ConsoleUsageBucket>,
}

#[derive(Debug, Serialize, Clone)]
pub(crate) struct ConsoleUsageBucket {
    name: String,
    count: u64,
    nar_size: i64,
}

#[derive(Debug, Serialize)]
pub(crate) struct ConsoleHealth {
    score: u8,
    issues: Vec<ConsoleHealthIssue>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ConsoleHealthIssue {
    severity: &'static str,
    title: String,
    detail: String,
    cache: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
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

#[derive(Debug, Serialize, Clone)]
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

#[derive(Debug, Serialize, Clone)]
pub(crate) struct ConsoleNar {
    nar_hash: String,
    nar_size: i64,
    compression: String,
    num_chunks: i32,
    completeness_hint: bool,
    holders_count: i32,
    created_at: String,
}

#[derive(Debug)]
struct CacheUsageAcc {
    cache: ConsoleCache,
    nar_size: i64,
    incomplete_objects: u64,
    never_accessed_objects: u64,
    last_upload_at: Option<chrono::DateTime<Utc>>,
    last_accessed_at: Option<chrono::DateTime<Utc>>,
    systems: HashMap<String, ConsoleUsageBucket>,
    compressions: HashMap<String, ConsoleUsageBucket>,
}

impl CacheUsageAcc {
    fn new(cache: ConsoleCache) -> Self {
        Self {
            cache,
            nar_size: 0,
            incomplete_objects: 0,
            never_accessed_objects: 0,
            last_upload_at: None,
            last_accessed_at: None,
            systems: HashMap::new(),
            compressions: HashMap::new(),
        }
    }
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

pub(crate) async fn usage(
    Extension(state): Extension<State>,
    Extension(req_state): Extension<RequestState>,
) -> ServerResult<Json<ConsoleUsage>> {
    let database = state.database().await?;

    let caches = Cache::find()
        .filter(cache::Column::DeletedAt.is_null())
        .order_by_asc(cache::Column::Name)
        .all(database)
        .await
        .map_err(ServerError::database_error)?;

    let cache_ids = caches.iter().map(|cache| cache.id).collect::<HashSet<_>>();
    let rows = Object::find()
        .find_also_related(Nar)
        .all(database)
        .await
        .map_err(ServerError::database_error)?;

    let active_rows = rows
        .into_iter()
        .filter_map(|(object, nar)| {
            if cache_ids.contains(&object.cache_id) {
                nar.map(|nar| (object, nar))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let mut object_counts = HashMap::<i64, u64>::new();
    for (object, _) in &active_rows {
        *object_counts.entry(object.cache_id).or_default() += 1;
    }

    let mut cache_summaries = HashMap::new();
    for cache in &caches {
        cache_summaries.insert(
            cache.id,
            console_cache(cache, object_counts.get(&cache.id).copied().unwrap_or(0), &req_state)?,
        );
    }

    let mut usage_by_cache = caches
        .iter()
        .map(|cache| {
            (
                cache.id,
                CacheUsageAcc::new(cache_summaries[&cache.id].clone()),
            )
        })
        .collect::<HashMap<_, _>>();
    let mut systems = HashMap::<String, ConsoleUsageBucket>::new();
    let mut compressions = HashMap::<String, ConsoleUsageBucket>::new();
    let mut uploaders = HashMap::<String, ConsoleUsageBucket>::new();
    let mut unique_nars = HashMap::<i64, i64>::new();
    let mut recent_uploads = Vec::new();
    let mut recent_accesses = Vec::new();
    let mut incomplete_objects = 0;
    let mut never_accessed_objects = 0;
    let mut logical_nar_size = 0;
    let now = Utc::now();
    let stale_cutoff = now - ChronoDuration::days(30);
    let mut issues = Vec::new();

    for (object, nar) in active_rows {
        let cache_summary = match cache_summaries.get(&object.cache_id) {
            Some(cache) => cache,
            None => continue,
        };
        let nar_size = nar.nar_size.max(0);
        logical_nar_size += nar_size;
        unique_nars.entry(nar.id).or_insert(nar_size);

        if !nar.completeness_hint {
            incomplete_objects += 1;
        }
        if object.last_accessed_at.is_none() {
            never_accessed_objects += 1;
        }

        add_bucket(
            &mut systems,
            object.system.as_deref().unwrap_or("unknown"),
            nar_size,
        );
        add_bucket(&mut compressions, &nar.compression, nar_size);
        add_bucket(
            &mut uploaders,
            object.created_by.as_deref().unwrap_or("unknown"),
            nar_size,
        );

        if let Some(acc) = usage_by_cache.get_mut(&object.cache_id) {
            acc.nar_size += nar_size;
            if !nar.completeness_hint {
                acc.incomplete_objects += 1;
            }
            if object.last_accessed_at.is_none() {
                acc.never_accessed_objects += 1;
            }
            acc.last_upload_at = Some(acc.last_upload_at.map_or(object.created_at, |current| {
                current.max(object.created_at)
            }));
            if let Some(last_accessed_at) = object.last_accessed_at {
                acc.last_accessed_at = Some(
                    acc.last_accessed_at
                        .map_or(last_accessed_at, |current| current.max(last_accessed_at)),
                );
            }
            add_bucket(
                &mut acc.systems,
                object.system.as_deref().unwrap_or("unknown"),
                nar_size,
            );
            add_bucket(&mut acc.compressions, &nar.compression, nar_size);
        }

        if !nar.completeness_hint {
            issues.push(ConsoleHealthIssue {
                severity: "error",
                title: "存在不完整对象".to_string(),
                detail: format!("{} 的 NAR 分块可能不完整。", object.store_path),
                cache: Some(cache_summary.name.clone()),
            });
        } else if object.last_accessed_at.is_none() && object.created_at < stale_cutoff {
            issues.push(ConsoleHealthIssue {
                severity: "warn",
                title: "长期未访问对象".to_string(),
                detail: format!("{} 创建超过 30 天且没有访问记录。", object.store_path),
                cache: Some(cache_summary.name.clone()),
            });
        }

        let console_object = console_object(cache_summary, object, nar);
        recent_uploads.push(console_object.clone());
        if console_object.last_accessed_at.is_some() {
            recent_accesses.push(console_object);
        }
    }

    for cache in &caches {
        if object_counts.get(&cache.id).copied().unwrap_or(0) == 0 {
            issues.push(ConsoleHealthIssue {
                severity: "info",
                title: "空缓存".to_string(),
                detail: "这个缓存还没有上传任何 store path。".to_string(),
                cache: Some(cache.name.clone()),
            });
        }
    }

    recent_uploads.sort_by_key(|object| Reverse(object.created_at.clone()));
    recent_uploads.truncate(8);
    recent_accesses.sort_by_key(|object| Reverse(object.last_accessed_at.clone()));
    recent_accesses.truncate(8);
    issues.truncate(10);

    let score = (100_i32
        - (incomplete_objects as i32 * 18)
        - (issues.iter().filter(|issue| issue.severity == "warn").count() as i32 * 4)
        - (issues.iter().filter(|issue| issue.severity == "info").count() as i32 * 2))
        .clamp(0, 100) as u8;

    let mut cache_usage = usage_by_cache
        .into_values()
        .map(|acc| ConsoleCacheUsage {
            cache: acc.cache,
            nar_size: acc.nar_size,
            incomplete_objects: acc.incomplete_objects,
            never_accessed_objects: acc.never_accessed_objects,
            last_upload_at: acc.last_upload_at.map(|value| value.to_rfc3339()),
            last_accessed_at: acc.last_accessed_at.map(|value| value.to_rfc3339()),
            systems: sorted_buckets(acc.systems, 5),
            compressions: sorted_buckets(acc.compressions, 5),
        })
        .collect::<Vec<_>>();
    cache_usage.sort_by_key(|cache| Reverse(cache.nar_size));

    Ok(Json(ConsoleUsage {
        totals: ConsoleUsageTotals {
            nar_size: unique_nars.values().sum(),
            logical_nar_size,
            incomplete_objects,
            never_accessed_objects,
        },
        cache_usage,
        recent_uploads,
        recent_accesses,
        systems: sorted_buckets(systems, 8),
        compressions: sorted_buckets(compressions, 8),
        uploaders: sorted_buckets(uploaders, 8),
        health: ConsoleHealth { score, issues },
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

fn add_bucket(buckets: &mut HashMap<String, ConsoleUsageBucket>, name: &str, nar_size: i64) {
    let bucket = buckets
        .entry(name.to_string())
        .or_insert_with(|| ConsoleUsageBucket {
            name: name.to_string(),
            count: 0,
            nar_size: 0,
        });
    bucket.count += 1;
    bucket.nar_size += nar_size;
}

fn sorted_buckets(
    buckets: HashMap<String, ConsoleUsageBucket>,
    limit: usize,
) -> Vec<ConsoleUsageBucket> {
    let mut buckets = buckets.into_values().collect::<Vec<_>>();
    buckets.sort_by_key(|bucket| Reverse((bucket.count, bucket.nar_size)));
    buckets.truncate(limit);
    buckets
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
