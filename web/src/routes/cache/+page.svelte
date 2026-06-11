<script>
  import { onMount } from 'svelte';
  import {
    AlertCircle,
    ArrowLeft,
    Box,
    CheckCircle2,
    Clipboard,
    ExternalLink,
    FileSearch,
    Info,
    RefreshCw,
    Save,
    Search,
    Settings2,
    ShieldCheck,
    Trash2
  } from '@lucide/svelte';
  import InfoBlock from './InfoBlock.svelte';
  import InfoLine from './InfoLine.svelte';

  let cacheName = '';
  let token = '';
  let payload = null;
  let selected = null;
  let loading = true;
  let busy = false;
  let error = '';
  let message = '';
  let copyMessage = '';
  let offset = 0;
  let objectQuery = '';
  let configTab = 'settings';
  const limit = 12;

  let edit = {
    isPublic: true,
    storeDir: '/nix/store',
    priority: 40,
    upstream: '',
    retentionMode: 'global',
    retentionSeconds: 0
  };

  const formatter = new Intl.DateTimeFormat('zh-CN', {
    year: 'numeric',
    month: 'short',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  });

  onMount(() => {
    cacheName = new URLSearchParams(location.search).get('name') || '';
    token = localStorage.getItem('attic.console.token') ?? '';
    loadObjects();
  });

  async function loadObjects(nextOffset = offset) {
    if (!cacheName) {
      error = '缺少缓存名称。';
      loading = false;
      return;
    }

    loading = true;
    error = '';
    offset = Math.max(0, nextOffset);

    try {
      const response = await fetch(
        `/_api/web/caches/${encodeURIComponent(cacheName)}/objects?limit=${limit}&offset=${offset}`
      );
      if (!response.ok) {
        const text = await response.text();
        throw new Error(text || `HTTP ${response.status}`);
      }
      payload = await response.json();
      selected = payload.objects[0] ?? null;
      syncEdit(payload.cache);
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      loading = false;
    }
  }

  function syncEdit(cache) {
    edit = {
      isPublic: cache.is_public,
      storeDir: cache.store_dir,
      priority: cache.priority,
      upstream: (cache.upstream_cache_key_names ?? []).join('\n'),
      retentionMode: cache.retention_period === null || cache.retention_period === undefined ? 'global' : 'period',
      retentionSeconds: cache.retention_period ?? 0
    };
  }

  async function saveCache() {
    if (!token.trim()) {
      message = '请先回首页生成或复制管理员 Token。';
      return;
    }
    busy = true;
    message = '';
    try {
      const body = {
        is_public: edit.isPublic,
        store_dir: edit.storeDir || '/nix/store',
        priority: Number(edit.priority),
        upstream_cache_key_names: edit.upstream
          .split(/\r?\n|,/)
          .map((item) => item.trim())
          .filter(Boolean),
        retention_period:
          edit.retentionMode === 'global' ? 'Global' : { Period: Number(edit.retentionSeconds) }
      };
      const response = await fetch(`/_api/web/caches/${encodeURIComponent(cacheName)}/config`, {
        method: 'PATCH',
        headers: {
          'Authorization': `Bearer ${token.trim()}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(body)
      });
      if (!response.ok) {
        const text = await response.text();
        throw new Error(text || `HTTP ${response.status}`);
      }
      message = '缓存配置已保存。';
      await loadObjects(offset);
    } catch (err) {
      message = err instanceof Error ? err.message : String(err);
    } finally {
      busy = false;
    }
  }

  async function deleteCache() {
    if (!token.trim()) {
      message = '请先回首页生成或复制管理员 Token。';
      return;
    }
    if (!confirm(`删除缓存 ${cacheName}？这个操作会让该缓存不再可用。`)) return;
    busy = true;
    message = '';
    try {
      const response = await fetch(`/_api/web/caches/${encodeURIComponent(cacheName)}/config`, {
        method: 'DELETE',
        headers: { 'Authorization': `Bearer ${token.trim()}` }
      });
      if (!response.ok) {
        const text = await response.text();
        throw new Error(text || `HTTP ${response.status}`);
      }
      location.href = '/';
    } catch (err) {
      message = err instanceof Error ? err.message : String(err);
    } finally {
      busy = false;
    }
  }

  async function copyText(value, label) {
    if (!value) return;
    await navigator.clipboard.writeText(value);
    copyMessage = `${label} 已复制`;
    setTimeout(() => { copyMessage = ''; }, 1800);
  }

  function formatDate(value) {
    return value ? formatter.format(new Date(value)) : '-';
  }

  function formatBytes(value) {
    if (!Number.isFinite(value)) return '-';
    const units = ['B', 'KiB', 'MiB', 'GiB', 'TiB'];
    let size = value;
    let unit = 0;
    while (size >= 1024 && unit < units.length - 1) {
      size /= 1024;
      unit += 1;
    }
    return `${size >= 10 || unit === 0 ? size.toFixed(0) : size.toFixed(1)} ${units[unit]}`;
  }

  function formatRetention(seconds) {
    if (seconds === null || seconds === undefined) return '全局默认';
    if (seconds === 0) return '永久保留';
    const days = Math.round(seconds / 86400);
    return days >= 1 ? `${days} 天` : `${Math.max(1, Math.round(seconds / 3600))} 小时`;
  }

  function matchesObject(object) {
    const query = objectQuery.trim().toLowerCase();
    if (!query) return true;
    return [
      object.store_path, object.store_path_hash, object.system,
      object.created_by, object.deriver, object.ca,
      object.nar?.nar_hash,
      ...(object.references ?? []),
      ...(object.sigs ?? [])
    ].some((v) => String(v ?? '').toLowerCase().includes(query));
  }

  $: cache = payload?.cache;
  $: objects = payload?.objects ?? [];
  $: visibleObjects = objects.filter(matchesObject);
  $: total = payload?.total ?? 0;
  $: hasPrev = offset > 0;
  $: hasNext = offset + limit < total;
</script>

<svelte:head>
  <title>{cacheName ? `${cacheName} - Attic` : 'Attic Cache'}</title>
</svelte:head>

<div class="page-header">
  <div style="display:flex;align-items:center;justify-content:space-between;">
    <div style="display:flex;align-items:center;gap:12px;">
      <a class="btn btn-ghost btn-sm btn-icon" href="/">
        <ArrowLeft size={16} />
      </a>
      <div>
        <h1 class="page-title">{cacheName || 'Cache Detail'}</h1>
        <p class="page-description">Cache Detail</p>
      </div>
    </div>
    <button class="btn btn-secondary" type="button" on:click={() => loadObjects(offset)} disabled={loading}>
      <span class:spin={loading}><RefreshCw size={15} /></span>
      <span>刷新</span>
    </button>
  </div>
</div>

<div class="page-body">
  {#if error}
    <div class="notice">
      <AlertCircle size={16} />
      <span>{error}</span>
    </div>
  {/if}

  {#if cache}
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-icon"><Box size={18} /></div>
        <div class="stat-content">
          <span class="stat-label">Objects</span>
          <span class="stat-value">{cache.objects}</span>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon"><ShieldCheck size={18} /></div>
        <div class="stat-content">
          <span class="stat-label">Status</span>
          <span class="stat-value">{cache.is_public ? '公开' : '私有'} · P{cache.priority}</span>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon"><CheckCircle2 size={18} /></div>
        <div class="stat-content">
          <span class="stat-label">Retention</span>
          <span class="stat-value">{formatRetention(cache.retention_period)}</span>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon"><ExternalLink size={18} /></div>
        <div class="stat-content">
          <span class="stat-label">Substituter</span>
          <span class="stat-value" style="font-size:0.82rem;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;max-width:240px;">{cache.substituter_endpoint}</span>
        </div>
      </div>
    </div>

    <div class="card">
      <div class="card-header">
        <div>
          <h2>{configTab === 'settings' ? '缓存配置' : '连接信息'}</h2>
          <p>{configTab === 'settings' ? '修改后立即调用 Attic 配置接口。' : '这些值可直接复制到客户端或 Nix 配置。'}</p>
        </div>
        <div class="tabs">
          <button class="tab" class:active={configTab === 'settings'} on:click={() => configTab = 'settings'}>
            <Settings2 size={13} />
            <span>配置</span>
          </button>
          <button class="tab" class:active={configTab === 'connection'} on:click={() => configTab = 'connection'}>
            <Info size={13} />
            <span>连接</span>
          </button>
        </div>
      </div>
      <div class="card-body">
        {#if configTab === 'settings'}
          <div class="form-grid">
            <label class="label">
              <span>Store 目录</span>
              <input class="input" bind:value={edit.storeDir} />
            </label>
            <label class="label">
              <span>优先级</span>
              <input class="input" type="number" bind:value={edit.priority} />
            </label>
            <label class="checkbox-label">
              <input type="checkbox" bind:checked={edit.isPublic} />
              <span>公开缓存</span>
            </label>
            <label class="label">
              <span>保留策略</span>
              <select class="select" bind:value={edit.retentionMode}>
                <option value="global">全局默认</option>
                <option value="period">指定秒数</option>
              </select>
            </label>
            {#if edit.retentionMode === 'period'}
              <label class="label">
                <span>保留秒数</span>
                <input class="input" type="number" min="0" bind:value={edit.retentionSeconds} />
              </label>
            {/if}
            <label class="label full-width">
              <span>上游 key，每行一个或逗号分隔</span>
              <textarea class="textarea" bind:value={edit.upstream} rows="3"></textarea>
            </label>
          </div>
          <div class="form-actions" style="margin-top:16px;">
            <button class="btn btn-primary" type="button" on:click={saveCache} disabled={busy}>
              <Save size={15} />
              <span>{busy ? '处理中' : '保存配置'}</span>
            </button>
            <button class="btn btn-destructive" type="button" on:click={deleteCache} disabled={busy}>
              <Trash2 size={15} />
              <span>删除缓存</span>
            </button>
          </div>
          {#if message}
            <p style="font-size:0.8rem;color:hsl(var(--muted-foreground));margin-top:12px;">{message}</p>
          {/if}
        {:else}
          <div class="connection-grid">
            <InfoLine label="API Endpoint" value={cache.api_endpoint} oncopy={copyText} />
            <InfoLine label="Substituter" value={cache.substituter_endpoint} oncopy={copyText} />
            <InfoLine label="Public key" value={cache.public_key} oncopy={copyText} />
            <InfoLine label="Upstream keys" value={(cache.upstream_cache_key_names ?? []).join(', ') || '无'} oncopy={copyText} />
          </div>
        {/if}
      </div>
    </div>

    <div class="card">
      <div class="card-header">
        <div>
          <h2>Store paths</h2>
          <p>当前页 {visibleObjects.length} / {objects.length}，全量 {total}。</p>
        </div>
        <div style="display:flex;gap:8px;align-items:center;">
          <div class="search-wrapper" style="width:260px;">
            <Search size={14} />
            <input class="input" bind:value={objectQuery} placeholder="搜索 store path、hash..." autocomplete="off" />
          </div>
          <button class="btn btn-secondary btn-sm" type="button" disabled={!hasPrev || loading} on:click={() => loadObjects(offset - limit)}>上一页</button>
          <button class="btn btn-secondary btn-sm" type="button" disabled={!hasNext || loading} on:click={() => loadObjects(offset + limit)}>下一页</button>
        </div>
      </div>
      <div class="card-body no-padding">
        <div class="object-layout">
          <div class="object-list-panel">
            {#if visibleObjects.length}
              {#each visibleObjects as object}
                <button
                  class="object-item"
                  class:active={selected?.store_path_hash === object.store_path_hash}
                  type="button"
                  on:click={() => selected = object}
                >
                  <strong>{object.store_path}</strong>
                  <span>{object.system || 'unknown'} · {formatBytes(object.nar.nar_size)} · {formatDate(object.created_at)}</span>
                </button>
              {/each}
            {:else}
              <div class="empty">{loading ? '正在加载对象...' : '没有匹配的 store path。'}</div>
            {/if}
          </div>

          <div class="object-detail-panel">
            {#if selected}
              <div class="card">
                <div class="card-header">
                  <div>
                    <h2>对象详情</h2>
                    <p>{selected.store_path_hash}</p>
                  </div>
                  <button class="btn btn-secondary btn-sm" type="button" on:click={() => copyText(selected.store_path, 'Store path')}>
                    <Clipboard size={14} />
                    <span>复制路径</span>
                  </button>
                </div>
                <div class="card-body">
                  <InfoLine label="Store path" value={selected.store_path} oncopy={copyText} />
                  <InfoLine label="Narinfo" value={selected.narinfo_url} oncopy={copyText} />
                  <InfoLine label="NAR URL" value={selected.nar_url} oncopy={copyText} />

                  <div class="detail-grid" style="margin-top:12px;">
                    <div class="detail-item"><span>System</span><strong>{selected.system || '-'}</strong></div>
                    <div class="detail-item"><span>上传者</span><strong>{selected.created_by || '-'}</strong></div>
                    <div class="detail-item"><span>创建时间</span><strong>{formatDate(selected.created_at)}</strong></div>
                    <div class="detail-item"><span>最后访问</span><strong>{formatDate(selected.last_accessed_at)}</strong></div>
                    <div class="detail-item"><span>NAR 大小</span><strong>{formatBytes(selected.nar.nar_size)}</strong></div>
                    <div class="detail-item"><span>压缩</span><strong>{selected.nar.compression}</strong></div>
                    <div class="detail-item"><span>分块</span><strong>{selected.nar.num_chunks}</strong></div>
                    <div class="detail-item"><span>完整</span><strong>{selected.nar.completeness_hint ? '是' : '否'}</strong></div>
                  </div>

                  <InfoBlock label="NAR hash" value={selected.nar.nar_hash} />
                  <InfoBlock label="Deriver" value={selected.deriver || '-'} />
                  <InfoBlock label="Content address" value={selected.ca || '-'} />
                  <InfoBlock label="References" value={selected.references.length ? selected.references.join('\n') : '无'} />
                  <InfoBlock label="Signatures" value={selected.sigs.length ? selected.sigs.join('\n') : '无'} />
                </div>
              </div>
            {:else}
              <div class="empty">选择左侧对象查看详情</div>
            {/if}
          </div>
        </div>
      </div>
    </div>
  {:else if loading}
    <div class="empty">正在加载缓存详情...</div>
  {/if}
</div>

{#if copyMessage}
  <div class="toast">{copyMessage}</div>
{/if}
