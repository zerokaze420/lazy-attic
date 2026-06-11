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

  function matchesObject(object) {
    const query = objectQuery.trim().toLowerCase();
    if (!query) {
      return true;
    }
    return [
      object.store_path,
      object.store_path_hash,
      object.system,
      object.created_by,
      object.deriver,
      object.ca,
      object.nar?.nar_hash,
      ...(object.references ?? []),
      ...(object.sigs ?? [])
    ].some((value) => String(value ?? '').toLowerCase().includes(query));
  }

  $: cache = payload?.cache;
  $: objects = payload?.objects ?? [];
  $: visibleObjects = objects.filter(matchesObject);
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

    <section class="detail-layout">
      <section class="panel config-panel">
        <div class="panel-head">
          <div>
            <h2>{configTab === 'settings' ? '缓存配置' : '连接信息'}</h2>
            <p>{configTab === 'settings' ? '修改后立即调用 Attic 配置接口。' : '这些值可直接复制到客户端或 Nix 配置。'}</p>
          </div>
          <div class="segmented">
            <button class:active={configTab === 'settings'} class="secondary" type="button" on:click={() => configTab = 'settings'}>
              <Settings2 size={15} />
              <span>配置</span>
            </button>
            <button class:active={configTab === 'connection'} class="secondary" type="button" on:click={() => configTab = 'connection'}>
              <Info size={15} />
              <span>连接</span>
            </button>
          </div>
        </div>
        {#if configTab === 'settings'}
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
        {:else}
          <div class="connection-list">
            <InfoLine label="API Endpoint" value={cache.api_endpoint} oncopy={copyText} />
            <InfoLine label="Substituter" value={cache.substituter_endpoint} oncopy={copyText} />
            <InfoLine label="Public key" value={cache.public_key} oncopy={copyText} />
            <InfoLine label="Upstream keys" value={(cache.upstream_cache_key_names ?? []).join(', ') || '无'} oncopy={copyText} />
          </div>
        {/if}
      </section>
    </section>

    <section class="objects">
      <div class="section-head">
        <div>
          <h2>Store paths</h2>
          <p>最近上传优先，当前页显示 {visibleObjects.length} / {objects.length}，全量 {total}。</p>
        </div>
        <div class="pager">
          <button class="secondary" type="button" disabled={!hasPrev || loading} on:click={() => loadObjects(offset - limit)}>上一页</button>
          <button class="secondary" type="button" disabled={!hasNext || loading} on:click={() => loadObjects(offset + limit)}>下一页</button>
        </div>
      </div>

      <section class="panel object-tools">
        <label class="search-field">
          <Search size={16} />
          <input bind:value={objectQuery} placeholder="搜索 store path、hash、system、引用" autocomplete="off" />
        </label>
        <span class="status-pill">
          <FileSearch size={15} />
          {offset + 1}-{Math.min(offset + limit, total)} / {total}
        </span>
      </section>

      <div class="object-grid">
        <section class="object-list">
          {#if visibleObjects.length}
            {#each visibleObjects as object}
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
            <div class="empty panel">{loading ? '正在加载对象...' : '没有匹配的 store path。'}</div>
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
