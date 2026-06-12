<script>
  import { onMount } from 'svelte';
  import {
    AlertCircle,
    ArrowLeft,
    Box,
    CheckCircle2,
    Clipboard,
    ExternalLink,
    Info,
    RefreshCw,
    Save,
    Settings2,
    ShieldCheck,
    Trash2
  } from '@lucide/svelte';
  import InfoLine from './InfoLine.svelte';

  let cacheName = '';
  let token = '';
  let cache = null;
  let usage = null;
  let loading = true;
  let busy = false;
  let error = '';
  let message = '';
  let copyMessage = '';
  let configTab = 'settings';

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
    loadData();
  });

  async function loadData() {
    if (!cacheName) {
      error = '缺少缓存名称。';
      loading = false;
      return;
    }

    loading = true;
    error = '';

    try {
      const [summaryRes, usageRes] = await Promise.all([
        fetch('/_api/web/summary'),
        fetch('/_api/web/usage')
      ]);

      if (!summaryRes.ok) {
        const text = await summaryRes.text();
        throw new Error(text || `HTTP ${summaryRes.status}`);
      }
      if (!usageRes.ok) {
        const text = await usageRes.text();
        throw new Error(text || `HTTP ${usageRes.status}`);
      }

      const summary = await summaryRes.json();
      const usagePayload = await usageRes.json();

      cache = summary.caches.find((c) => c.name === cacheName) ?? null;
      if (!cache) throw new Error(`缓存 "${cacheName}" 不存在`);

      usage = usagePayload.cache_usage.find((u) => u.cache.name === cacheName) ?? null;

      syncEdit(cache);
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      loading = false;
    }
  }

  function syncEdit(c) {
    edit = {
      isPublic: c.is_public,
      storeDir: c.store_dir,
      priority: c.priority,
      upstream: (c.upstream_cache_key_names ?? []).join('\n'),
      retentionMode: c.retention_period === null || c.retention_period === undefined ? 'global' : 'period',
      retentionSeconds: c.retention_period ?? 0
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
      await loadData();
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

  $: maxSysCount = usage?.systems?.length ? Math.max(...usage.systems.map(s => s.count)) : 1;
  $: maxCompCount = usage?.compressions?.length ? Math.max(...usage.compressions.map(c => c.count)) : 1;
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
        <h1 class="page-title">{cacheName || '缓存详情'}</h1>
        <p class="page-description">{cache?.store_dir ?? 'Nix Store'}</p>
      </div>
    </div>
    <button class="btn btn-secondary" type="button" on:click={loadData} disabled={loading}>
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
          <span class="stat-label">对象数</span>
          <span class="stat-value">{cache.objects}</span>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon"><ShieldCheck size={18} /></div>
        <div class="stat-content">
          <span class="stat-label">状态</span>
          <span class="stat-value">{cache.is_public ? '公开' : '私有'} · P{cache.priority}</span>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon"><CheckCircle2 size={18} /></div>
        <div class="stat-content">
          <span class="stat-label">保留策略</span>
          <span class="stat-value">{formatRetention(cache.retention_period)}</span>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon"><ExternalLink size={18} /></div>
        <div class="stat-content">
          <span class="stat-label">最后上传</span>
          <span class="stat-value" style="font-size:0.82rem;">{usage ? formatDate(usage.last_upload_at) : '-'}</span>
        </div>
      </div>
    </div>

    <div class="stats-grid" style="margin-top:8px;">
      <div class="stat-card" style="grid-column:1/-1;">
        <div class="stat-content" style="width:100%;">
          <span class="stat-label">NAR 存储量</span>
          <span class="stat-value" style="font-size:1.4rem;">{usage ? formatBytes(usage.nar_size) : '-'}</span>
        </div>
        {#if usage}
          <div style="margin-top:8px;height:8px;background:hsl(var(--muted));border-radius:4px;overflow:hidden;">
            <div
              style="height:100%;background:linear-gradient(90deg,hsl(var(--primary)),hsl(var(--ring)));border-radius:4px;transition:width .4s;"
            ></div>
          </div>
        {/if}
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

    {#if usage}
      <div class="card">
        <div class="card-header">
          <div>
            <h2>存储与健康状况</h2>
            <p>缓存整体健康度与分布</p>
          </div>
        </div>
        <div class="card-body">
          <div class="detail-grid">
            <div class="detail-item">
              <span>最后上传</span>
              <strong>{formatDate(usage.last_upload_at)}</strong>
            </div>
            <div class="detail-item">
              <span>最后访问</span>
              <strong>{formatDate(usage.last_accessed_at)}</strong>
            </div>
            <div class="detail-item">
              <span>不完整对象</span>
              <strong>{usage.incomplete_objects} / {cache.objects}</strong>
            </div>
            <div class="detail-item">
              <span>从未访问</span>
              <strong>{usage.never_accessed_objects} / {cache.objects}</strong>
            </div>
          </div>

          {#if usage.systems?.length}
            <h3 style="font-size:0.9rem;margin:20px 0 8px;font-weight:600;">系统架构分布</h3>
            <div class="breakdown-list">
              {#each usage.systems as sys}
                <div class="breakdown-row">
                  <div class="breakdown-row-label">
                    <span class="badge badge-default">{sys.name}</span>
                  </div>
                  <div class="breakdown-bar-track">
                    <div class="breakdown-bar" style="width:{(sys.count / maxSysCount) * 100}%"></div>
                  </div>
                  <div class="breakdown-row-value">
                    {sys.count} 个 · {formatBytes(sys.nar_size)}
                  </div>
                </div>
              {/each}
            </div>
          {/if}

          {#if usage.compressions?.length}
            <h3 style="font-size:0.9rem;margin:20px 0 8px;font-weight:600;">压缩方式分布</h3>
            <div class="breakdown-list">
              {#each usage.compressions as comp}
                <div class="breakdown-row">
                  <div class="breakdown-row-label">
                    <span class="badge badge-default">{comp.name}</span>
                  </div>
                  <div class="breakdown-bar-track">
                    <div class="breakdown-bar" style="width:{(comp.count / maxCompCount) * 100}%"></div>
                  </div>
                  <div class="breakdown-row-value">
                    {comp.count} 个 · {formatBytes(comp.nar_size)}
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    {/if}
  {:else if loading}
    <div class="empty">正在加载缓存详情...</div>
  {/if}
</div>

{#if copyMessage}
  <div class="toast">{copyMessage}</div>
{/if}

<style>
  .breakdown-list {
    display:flex;
    flex-direction:column;
    gap:10px;
  }
  .breakdown-row {
    display:grid;
    grid-template-columns:auto 1fr auto;
    align-items:center;
    gap:12px;
  }
  .breakdown-row-label {
    min-width:140px;
  }
  .breakdown-bar-track {
    height:10px;
    background:hsl(var(--muted));
    border-radius:5px;
    overflow:hidden;
  }
  .breakdown-bar {
    height:100%;
    background:linear-gradient(90deg,hsl(var(--primary)),hsl(var(--ring)));
    border-radius:5px;
    transition:width .4s ease;
  }
  .breakdown-row-value {
    font-size:0.8rem;
    color:hsl(var(--muted-foreground));
    white-space:nowrap;
  }
</style>
