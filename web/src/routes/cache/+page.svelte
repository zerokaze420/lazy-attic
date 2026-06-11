<script>
  import { onMount } from 'svelte';
  import {
    AlertCircle,
    ArrowLeft,
    Box,
    CheckCircle2,
    Clipboard,
    ExternalLink,
    RefreshCw,
    Save,
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
    if (!confirm(`删除缓存 ${cacheName}？这个操作会让该缓存不再可用。`)) {
      return;
    }

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
    if (!value) {
      return;
    }
    await navigator.clipboard.writeText(value);
    copyMessage = `${label} 已复制`;
    setTimeout(() => {
      copyMessage = '';
    }, 1800);
  }

  function formatDate(value) {
    return value ? formatter.format(new Date(value)) : '-';
  }

  function formatBytes(value) {
    if (!Number.isFinite(value)) {
      return '-';
    }
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
    if (seconds === null || seconds === undefined) {
      return '全局默认';
    }
    if (seconds === 0) {
      return '永久保留';
    }
    const days = Math.round(seconds / 86400);
    return days >= 1 ? `${days} 天` : `${Math.max(1, Math.round(seconds / 3600))} 小时`;
  }

  $: cache = payload?.cache;
  $: objects = payload?.objects ?? [];
  $: total = payload?.total ?? 0;
  $: hasPrev = offset > 0;
  $: hasNext = offset + limit < total;
</script>

<svelte:head>
  <title>{cacheName ? `${cacheName} - Attic 缓存详情` : 'Attic 缓存详情'}</title>
</svelte:head>

<main class="page">
  <header class="topbar">
    <a class="nav-link" href="/">
      <ArrowLeft size={16} />
      <span>返回</span>
    </a>
    <div class="title">
      <p class="eyebrow">Cache Detail</p>
      <h1>{cacheName || '缓存详情'}</h1>
    </div>
    <button class="secondary" type="button" on:click={() => loadObjects(offset)} disabled={loading}>
      <span class:spin={loading}><RefreshCw size={16} /></span>
      <span>刷新</span>
    </button>
  </header>

  {#if error}
    <section class="notice">
      <AlertCircle size={18} />
      <span>{error}</span>
    </section>
  {/if}

  {#if cache}
    <section class="summary">
      <article class="metric">
        <Box size={19} />
        <span>对象</span>
        <strong>{cache.objects}</strong>
      </article>
      <article class="metric">
        <ShieldCheck size={19} />
        <span>状态</span>
        <strong>{cache.is_public ? '公开' : '私有'} · P{cache.priority}</strong>
      </article>
      <article class="metric wide">
        <CheckCircle2 size={19} />
        <span>保留策略</span>
        <strong>{formatRetention(cache.retention_period)}</strong>
      </article>
      <article class="metric wide">
        <ExternalLink size={19} />
        <span>Substituter</span>
        <strong>{cache.substituter_endpoint}</strong>
      </article>
    </section>

    <section class="grid">
      <section class="panel editor">
        <div class="panel-head">
          <div>
            <h2>缓存配置</h2>
            <p>修改后立即调用 Attic 配置接口。</p>
          </div>
          <span class="tag">{cache.store_dir}</span>
        </div>
        <div class="form-grid">
          <label>
            <span>Store 目录</span>
            <input bind:value={edit.storeDir} />
          </label>
          <label>
            <span>优先级</span>
            <input type="number" bind:value={edit.priority} />
          </label>
          <label class="check">
            <input type="checkbox" bind:checked={edit.isPublic} />
            <span>公开缓存</span>
          </label>
          <label>
            <span>保留策略</span>
            <select bind:value={edit.retentionMode}>
              <option value="global">全局默认</option>
              <option value="period">指定秒数</option>
            </select>
          </label>
          {#if edit.retentionMode === 'period'}
            <label>
              <span>保留秒数</span>
              <input type="number" min="0" bind:value={edit.retentionSeconds} />
            </label>
          {/if}
          <label class="full">
            <span>上游 key，每行一个或逗号分隔</span>
            <textarea bind:value={edit.upstream} rows="3"></textarea>
          </label>
        </div>
        <div class="actions">
          <button type="button" on:click={saveCache} disabled={busy}>
            <Save size={16} />
            <span>{busy ? '处理中' : '保存配置'}</span>
          </button>
          <button class="danger" type="button" on:click={deleteCache} disabled={busy}>
            <Trash2 size={16} />
            <span>删除缓存</span>
          </button>
        </div>
        {#if message}
          <p class="hint">{message}</p>
        {/if}
      </section>

      <section class="panel info">
        <div class="panel-head">
          <div>
            <h2>连接信息</h2>
            <p>这些值可直接复制到客户端或 Nix 配置。</p>
          </div>
        </div>
        <InfoLine label="API Endpoint" value={cache.api_endpoint} oncopy={copyText} />
        <InfoLine label="Substituter" value={cache.substituter_endpoint} oncopy={copyText} />
        <InfoLine label="Public key" value={cache.public_key} oncopy={copyText} />
        <InfoLine label="Upstream keys" value={(cache.upstream_cache_key_names ?? []).join(', ') || '无'} oncopy={copyText} />
      </section>
    </section>

    <section class="objects">
      <div class="section-head">
        <div>
          <h2>Store paths</h2>
          <p>最近上传优先，当前显示 {offset + 1}-{Math.min(offset + limit, total)} / {total}。</p>
        </div>
        <div class="pager">
          <button class="secondary" type="button" disabled={!hasPrev || loading} on:click={() => loadObjects(offset - limit)}>上一页</button>
          <button class="secondary" type="button" disabled={!hasNext || loading} on:click={() => loadObjects(offset + limit)}>下一页</button>
        </div>
      </div>

      <div class="object-grid">
        <section class="object-list">
          {#if objects.length}
            {#each objects as object}
              <button
                class:selected={selected?.store_path_hash === object.store_path_hash}
                class="object-row"
                type="button"
                on:click={() => selected = object}
              >
                <strong>{object.store_path}</strong>
                <span>{object.system || 'unknown'} · {formatBytes(object.nar.nar_size)} · {formatDate(object.created_at)}</span>
              </button>
            {/each}
          {:else}
            <div class="empty panel">{loading ? '正在加载对象...' : '还没有上传 store path。'}</div>
          {/if}
        </section>

        {#if selected}
          <section class="panel object-detail">
            <div class="panel-head">
              <div>
                <h2>对象详情</h2>
                <p>{selected.store_path_hash}</p>
              </div>
              <button class="secondary" type="button" on:click={() => copyText(selected.store_path, 'Store path')}>
                <Clipboard size={16} />
                <span>复制路径</span>
              </button>
            </div>
            <InfoLine label="Store path" value={selected.store_path} oncopy={copyText} />
            <InfoLine label="Narinfo" value={selected.narinfo_url} oncopy={copyText} />
            <InfoLine label="NAR URL" value={selected.nar_url} oncopy={copyText} />
            <div class="detail-grid">
              <div><span>System</span><strong>{selected.system || '-'}</strong></div>
              <div><span>上传者</span><strong>{selected.created_by || '-'}</strong></div>
              <div><span>创建时间</span><strong>{formatDate(selected.created_at)}</strong></div>
              <div><span>最后访问</span><strong>{formatDate(selected.last_accessed_at)}</strong></div>
              <div><span>NAR 大小</span><strong>{formatBytes(selected.nar.nar_size)}</strong></div>
              <div><span>压缩</span><strong>{selected.nar.compression}</strong></div>
              <div><span>分块</span><strong>{selected.nar.num_chunks}</strong></div>
              <div><span>完整</span><strong>{selected.nar.completeness_hint ? '是' : '否'}</strong></div>
            </div>
            <InfoBlock label="NAR hash" value={selected.nar.nar_hash} />
            <InfoBlock label="Deriver" value={selected.deriver || '-'} />
            <InfoBlock label="Content address" value={selected.ca || '-'} />
            <InfoBlock label="References" value={selected.references.length ? selected.references.join('\n') : '无'} />
            <InfoBlock label="Signatures" value={selected.sigs.length ? selected.sigs.join('\n') : '无'} />
          </section>
        {/if}
      </div>
    </section>
  {:else if loading}
    <section class="empty panel">正在加载缓存详情...</section>
  {/if}

  {#if copyMessage}
    <div class="toast">{copyMessage}</div>
  {/if}
</main>

<style>
  :global(*) {
    box-sizing: border-box;
  }

  :global(body) {
    margin: 0;
    min-width: 320px;
    background: #f4f7fb;
    color: #17202a;
    font-family:
      Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont,
      "Segoe UI", sans-serif;
  }

  .page {
    width: min(1440px, calc(100% - 32px));
    margin: 0 auto;
    padding: 20px 0 36px;
  }

  .topbar,
  .nav-link,
  button,
  .panel-head,
  .section-head,
  .actions,
  .pager {
    display: flex;
    align-items: center;
  }

  .topbar,
  .section-head {
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 12px;
  }

  .title {
    flex: 1;
  }

  h1,
  h2,
  p {
    margin: 0;
  }

  h1 {
    font-size: 1.65rem;
    letter-spacing: 0;
  }

  h2 {
    font-size: 1rem;
    letter-spacing: 0;
  }

  .eyebrow,
  .hint,
  label span,
  .panel-head p,
  .section-head p,
  .metric span,
  .object-row span,
  .detail-grid span {
    color: #667487;
    font-size: 0.84rem;
  }

  .eyebrow {
    font-weight: 800;
    text-transform: uppercase;
  }

  .panel,
  .metric {
    border: 1px solid #dbe3ee;
    border-radius: 8px;
    background: #fff;
  }

  button,
  .nav-link {
    justify-content: center;
    gap: 8px;
    min-height: 36px;
    border: 1px solid #175cd3;
    border-radius: 6px;
    padding: 0 12px;
    background: #175cd3;
    color: #fff;
    font: inherit;
    font-size: 0.92rem;
    font-weight: 750;
    text-decoration: none;
    cursor: pointer;
  }

  .secondary,
  .nav-link {
    border-color: #cad2df;
    background: #fff;
    color: #263548;
  }

  .danger {
    border-color: #b42318;
    background: #b42318;
  }

  button:disabled {
    cursor: progress;
    opacity: 0.62;
  }

  .spin {
    animation: spin 900ms linear infinite;
  }

  .notice {
    display: flex;
    gap: 10px;
    margin-bottom: 12px;
    padding: 12px 14px;
    border: 1px solid #f3b7b0;
    border-radius: 8px;
    background: #fff4f2;
    color: #7c251f;
  }

  .summary,
  .grid,
  .object-grid,
  .detail-grid,
  .form-grid {
    display: grid;
    gap: 12px;
  }

  .summary {
    grid-template-columns: 0.7fr 1fr 1fr 2fr;
    margin-bottom: 12px;
  }

  .metric {
    display: grid;
    grid-template-columns: 22px 1fr;
    gap: 4px 9px;
    padding: 13px;
    color: #175cd3;
  }

  .metric strong {
    grid-column: 2;
    min-width: 0;
    color: #17202a;
    font-size: 1.08rem;
    line-height: 1.3;
    overflow-wrap: anywhere;
  }

  .grid {
    grid-template-columns: minmax(0, 1.1fr) minmax(0, 0.9fr);
    margin-bottom: 16px;
  }

  .editor,
  .info,
  .object-detail {
    display: grid;
    gap: 12px;
    padding: 14px;
  }

  .panel-head {
    justify-content: space-between;
    gap: 10px;
  }

  .tag {
    border-radius: 999px;
    padding: 5px 9px;
    background: #edf5ff;
    color: #175cd3;
    font-size: 0.78rem;
    font-weight: 750;
  }

  .form-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  label {
    display: grid;
    gap: 6px;
  }

  .full {
    grid-column: 1 / -1;
  }

  .check {
    grid-template-columns: 18px 1fr;
    align-items: center;
    min-height: 36px;
    gap: 8px;
  }

  input,
  select,
  textarea {
    width: 100%;
    min-height: 36px;
    border: 1px solid #cbd5e1;
    border-radius: 6px;
    padding: 8px 10px;
    background: #fff;
    color: #17202a;
    font: inherit;
  }

  textarea {
    resize: vertical;
  }

  input,
  textarea {
    font-family: "SFMono-Regular", Consolas, "Liberation Mono", monospace;
  }

  .actions,
  .pager {
    flex-wrap: wrap;
    gap: 10px;
  }

  .objects {
    margin-top: 4px;
  }

  .object-grid {
    grid-template-columns: minmax(0, 0.95fr) minmax(0, 1.25fr);
    align-items: start;
  }

  .object-list {
    display: grid;
    gap: 8px;
  }

  .object-row {
    display: grid;
    gap: 5px;
    min-height: auto;
    justify-content: stretch;
    border-color: #dbe3ee;
    padding: 11px;
    background: #fff;
    color: #17202a;
    text-align: left;
  }

  .object-row.selected {
    border-color: #175cd3;
    background: #edf5ff;
  }

  .object-row strong,
  .object-row span {
    min-width: 0;
    overflow-wrap: anywhere;
  }

  .detail-grid {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }

  .detail-grid div {
    border: 1px solid #edf1f5;
    border-radius: 6px;
    padding: 9px;
    background: #f8fafc;
  }

  .detail-grid strong {
    display: block;
    margin-top: 3px;
    overflow-wrap: anywhere;
  }

  .empty {
    padding: 34px;
    color: #667487;
    text-align: center;
  }

  .toast {
    position: fixed;
    right: 18px;
    bottom: 18px;
    z-index: 20;
    border: 1px solid #b7e4c7;
    border-radius: 8px;
    padding: 11px 13px;
    background: #effaf3;
    color: #1f7a43;
    font-size: 0.9rem;
    font-weight: 750;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  @media (max-width: 1100px) {
    .summary,
    .grid,
    .object-grid {
      grid-template-columns: 1fr;
    }

    .detail-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  @media (max-width: 680px) {
    .page {
      width: min(100% - 20px, 1440px);
      padding-top: 14px;
    }

    .topbar,
    .panel-head,
    .section-head {
      align-items: stretch;
      flex-direction: column;
    }

    .summary,
    .form-grid,
    .detail-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
