<script>
  import { onMount } from 'svelte';
  import {
    AlertCircle,
    BookOpen,
    Box,
    CheckCircle2,
    Clipboard,
    Database,
    ExternalLink,
    KeyRound,
    Layers3,
    Plus,
    RefreshCw,
    Server,
    ShieldCheck,
    Sparkles,
    Terminal
  } from '@lucide/svelte';

  const defaultCreate = {
    name: '',
    isPublic: true,
    priority: 40,
    storeDir: '/nix/store'
  };

  let summary = null;
  let loading = true;
  let error = '';
  let token = '';
  let create = { ...defaultCreate };
  let createBusy = false;
  let createMessage = '';
  let tokenBusy = false;
  let origin = '';
  let adminTokenExpires = '';
  let copyMessage = '';

  const formatter = new Intl.DateTimeFormat('zh-CN', {
    year: 'numeric',
    month: 'short',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  });

  onMount(() => {
    origin = location.origin;
    token = localStorage.getItem('attic.console.token') ?? '';
    refresh();
  });

  async function refresh() {
    loading = true;
    error = '';

    try {
      const response = await fetch('/_api/web/summary');
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }
      summary = await response.json();
      if (summary.admin_token?.token) {
        token = summary.admin_token.token;
        adminTokenExpires = summary.admin_token.expires_at;
        localStorage.setItem('attic.console.token', token);
      }
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      loading = false;
    }
  }

  async function issueAdminToken() {
    tokenBusy = true;
    createMessage = '';

    try {
      const response = await fetch('/_api/web/admin-token', { method: 'POST' });
      if (!response.ok) {
        const text = await response.text();
        throw new Error(text || `HTTP ${response.status}`);
      }

      const payload = await response.json();
      token = payload.token;
      adminTokenExpires = payload.expires_at;
      localStorage.setItem('attic.console.token', token);
      createMessage = `管理员 Token 已生成，有效期至 ${formatDate(payload.expires_at)}。`;
    } catch (err) {
      createMessage = err instanceof Error ? err.message : String(err);
    } finally {
      tokenBusy = false;
    }
  }

  async function createCache() {
    createBusy = true;
    createMessage = '';

    try {
      const name = create.name.trim();
      if (!name) {
        throw new Error('请输入缓存名称。');
      }
      if (!token.trim()) {
        throw new Error('请先填写管理员 Token。');
      }

      const response = await fetch(`/_api/v1/cache-config/${encodeURIComponent(name)}`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token.trim()}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          keypair: 'Generate',
          is_public: create.isPublic,
          store_dir: create.storeDir || '/nix/store',
          priority: Number(create.priority),
          upstream_cache_key_names: []
        })
      });

      if (!response.ok) {
        const text = await response.text();
        throw new Error(text || `HTTP ${response.status}`);
      }

      create = { ...defaultCreate };
      createMessage = `缓存 "${name}" 已创建。`;
      await refresh();
    } catch (err) {
      createMessage = err instanceof Error ? err.message : String(err);
    } finally {
      createBusy = false;
    }
  }

  async function copyText(value, label) {
    if (!value || value.startsWith('<')) {
      return;
    }
    await navigator.clipboard.writeText(value);
    copyMessage = `${label} 已复制`;
    setTimeout(() => {
      copyMessage = '';
    }, 1800);
  }

  function formatDate(value) {
    return formatter.format(new Date(value));
  }

  function formatRetention(seconds) {
    if (seconds === null || seconds === undefined) {
      return '全局默认';
    }
    if (seconds === 0) {
      return '永久保留';
    }
    const days = Math.round(seconds / 86400);
    if (days >= 1) {
      return `${days} 天`;
    }
    const hours = Math.round(seconds / 3600);
    return `${hours || 1} 小时`;
  }

  function joinList(values) {
    return values?.length ? values.join(', ') : '无';
  }

  $: caches = summary?.caches ?? [];
  $: publicCacheModel = caches.find((cache) => cache.is_public) || caches[0];
  $: exampleCache = caches[0]?.name || '<缓存名>';
  $: publicCache = publicCacheModel?.name || exampleCache;
  $: publicKey = publicCacheModel?.public_key || '<缓存公钥>';
  $: apiEndpoint = publicCacheModel?.api_endpoint || `${origin}/`;
  $: substituterEndpoint = publicCacheModel?.substituter_endpoint || `${origin}/${publicCache}`;
  $: loginCommand = `attic login local ${apiEndpoint} <token>`;
  $: useCommand = `attic use local:${exampleCache}`;
  $: pushCommand = `attic push local:${exampleCache} /nix/store/<path>`;
  $: substituterCommand = `substituters = ${substituterEndpoint}`;
  $: trustedKeysCommand = `trusted-public-keys = ${publicKey}`;
  $: clientCommands = [
    { value: loginCommand, label: '登录命令' },
    { value: useCommand, label: '启用命令' },
    { value: pushCommand, label: '推送命令' }
  ];
  $: nixCommands = [
    { value: substituterCommand, label: 'Substituter' },
    { value: trustedKeysCommand, label: 'Public key' }
  ];
</script>

<svelte:head>
  <title>Attic 控制台</title>
  <meta name="description" content="Attic 的 LazyCat Web 控制台" />
</svelte:head>

<main class="app">
  <header class="topbar">
    <div class="brand">
      <div class="brand-mark"><Server size={22} /></div>
      <div>
        <p class="eyebrow">Nix 二进制缓存</p>
        <h1>Attic 控制台</h1>
      </div>
    </div>
    <div class="top-actions">
      <span class:online={summary?.status === 'online'} class="status-pill">
        <CheckCircle2 size={16} />
        {summary?.status === 'online' ? '服务在线' : (loading ? '加载中' : '未知状态')}
      </span>
      <a class="nav-link" href="/guide">
        <BookOpen size={16} />
        <span>使用教程</span>
      </a>
      <button class="secondary" type="button" on:click={refresh} disabled={loading}>
        <span class:spin={loading}><RefreshCw size={16} /></span>
        <span>{loading ? '刷新中' : '刷新'}</span>
      </button>
    </div>
  </header>

  {#if error}
    <section class="notice">
      <AlertCircle size={18} />
      <div>
        <strong>控制台接口不可用</strong>
        <span>{error}</span>
      </div>
    </section>
  {/if}

  <section class="metrics" aria-label="服务指标">
    <article class="metric">
      <Database size={19} />
      <span>缓存</span>
      <strong>{summary?.counts?.caches ?? '-'}</strong>
    </article>
    <article class="metric">
      <Box size={19} />
      <span>对象</span>
      <strong>{summary?.counts?.objects ?? '-'}</strong>
    </article>
    <article class="metric">
      <Layers3 size={19} />
      <span>NAR / 分块</span>
      <strong>{summary ? `${summary.counts.nars} / ${summary.counts.chunks}` : '-'}</strong>
    </article>
    <article class="metric">
      <ShieldCheck size={19} />
      <span>{summary?.storage?.kind ?? '存储'}</span>
      <strong>{summary?.storage?.location ?? '-'}</strong>
    </article>
  </section>

  <section class="toolbar panel">
    <div>
      <h2>缓存</h2>
      <p>显示完整 endpoint、保留策略和上游 key。点击详情查看 store path 与 NAR 信息。</p>
    </div>
    <div class="toolbar-actions">
      <button class="secondary" type="button" on:click={() => copyText(substituterEndpoint, 'Substituter')}>
        <Clipboard size={16} />
        <span>复制 substituter</span>
      </button>
      <button class="secondary" type="button" on:click={() => copyText(publicKey, 'Public key')}>
        <Clipboard size={16} />
        <span>复制 public key</span>
      </button>
    </div>
  </section>

  <section class="cache-grid">
    {#if caches.length}
      {#each caches as cache}
        <article class="cache-card panel">
          <div class="cache-head">
            <div>
              <h3>{cache.name}</h3>
              <p>{cache.store_dir}</p>
            </div>
            <span class="tag">{cache.is_public ? '公开' : '私有'} · P{cache.priority}</span>
          </div>
          <div class="facts">
            <div><span>对象</span><strong>{cache.objects}</strong></div>
            <div><span>保留</span><strong>{formatRetention(cache.retention_period)}</strong></div>
            <div><span>创建</span><strong>{formatDate(cache.created_at)}</strong></div>
          </div>
          <div class="field">
            <span>Substituter</span>
            <code>{cache.substituter_endpoint}</code>
          </div>
          <div class="field">
            <span>API Endpoint</span>
            <code>{cache.api_endpoint}</code>
          </div>
          <div class="field">
            <span>Upstream keys</span>
            <code>{joinList(cache.upstream_cache_key_names)}</code>
          </div>
          <div class="field">
            <span>Public key</span>
            <code>{cache.public_key}</code>
          </div>
          <div class="card-actions">
            <a class="nav-link primary-link" href={`/cache?name=${encodeURIComponent(cache.name)}`}>
              <ExternalLink size={16} />
              <span>详情</span>
            </a>
            <button class="secondary" type="button" on:click={() => copyText(cache.public_key, 'Public key')}>
              <Clipboard size={16} />
              <span>复制 key</span>
            </button>
          </div>
        </article>
      {/each}
    {:else}
      <div class="empty panel">{loading ? '正在加载缓存...' : '还没有创建缓存。'}</div>
    {/if}
  </section>

  <section class="workspace">
    <section class="panel control-card">
      <div class="panel-head compact">
        <div>
          <h2>管理员 Token</h2>
          <p>Token 保存在当前浏览器，可用于创建缓存和客户端登录。</p>
        </div>
        <KeyRound size={20} />
      </div>
      <code class="wrap-code">{token || '等待服务返回'}</code>
      {#if adminTokenExpires}
        <p class="hint">有效期至 {formatDate(adminTokenExpires)}</p>
      {/if}
      <div class="button-row">
        <button type="button" on:click={issueAdminToken} disabled={tokenBusy}>
          <Sparkles size={16} />
          <span>{tokenBusy ? '生成中' : '重新生成'}</span>
        </button>
        <button class="secondary" type="button" on:click={() => copyText(token, '管理员 Token')}>
          <Clipboard size={16} />
          <span>复制</span>
        </button>
      </div>
    </section>

    <section class="panel control-card">
      <div class="panel-head compact">
        <div>
          <h2>创建缓存</h2>
          <p>默认生成签名 keypair。</p>
        </div>
        <Plus size={20} />
      </div>
      <label>
        <span>名称</span>
        <input bind:value={create.name} placeholder="main" autocomplete="off" />
      </label>
      <label>
        <span>Store 目录</span>
        <input bind:value={create.storeDir} />
      </label>
      <div class="split">
        <label>
          <span>优先级</span>
          <input bind:value={create.priority} type="number" />
        </label>
        <label class="check">
          <input bind:checked={create.isPublic} type="checkbox" />
          <span>公开</span>
        </label>
      </div>
      <button type="button" on:click={createCache} disabled={createBusy}>
        <Plus size={16} />
        <span>{createBusy ? '创建中' : '创建缓存'}</span>
      </button>
      {#if createMessage}
        <p class="hint">{createMessage}</p>
      {/if}
    </section>
  </section>

  <section class="command-grid">
    <article class="panel command-card">
      <div class="panel-head compact">
        <div>
          <h2>客户端命令</h2>
          <p>登录、启用缓存并推送 store path。</p>
        </div>
        <Terminal size={20} />
      </div>
      {#each clientCommands as command}
        <div class="code-line">
          <code>{command.value}</code>
          <button class="icon-button" type="button" title={`复制${command.label}`} on:click={() => copyText(command.value, command.label)}>
            <Clipboard size={15} />
          </button>
        </div>
      {/each}
    </article>

    <article class="panel command-card">
      <div class="panel-head compact">
        <div>
          <h2>Nix 配置</h2>
          <p>公开缓存可直接作为 substituter。</p>
        </div>
        <ShieldCheck size={20} />
      </div>
      {#each nixCommands as command}
        <div class="code-line">
          <code>{command.value}</code>
          <button class="icon-button" type="button" title={`复制${command.label}`} on:click={() => copyText(command.value, command.label)}>
            <Clipboard size={15} />
          </button>
        </div>
      {/each}
    </article>
  </section>

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

  .app {
    width: min(1440px, calc(100% - 32px));
    margin: 0 auto;
    padding: 20px 0 36px;
  }

  .topbar,
  .brand,
  .top-actions,
  button,
  .status-pill,
  .nav-link,
  a {
    display: flex;
    align-items: center;
  }

  .topbar {
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 14px;
  }

  .brand,
  .top-actions,
  .toolbar-actions,
  .card-actions {
    gap: 10px;
  }

  .brand-mark {
    display: grid;
    place-items: center;
    width: 42px;
    height: 42px;
    border: 1px solid #c8d7ea;
    border-radius: 8px;
    background: #ffffff;
    color: #175cd3;
  }

  h1,
  h2,
  h3,
  p {
    margin: 0;
  }

  h1 {
    margin-top: 2px;
    font-size: 1.7rem;
    letter-spacing: 0;
  }

  h2 {
    font-size: 1rem;
    letter-spacing: 0;
  }

  h3 {
    font-size: 1.08rem;
    letter-spacing: 0;
  }

  .eyebrow,
  .hint,
  label span,
  .panel-head p,
  .cache-card p,
  .field span,
  .metric span,
  .facts span {
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

  .primary-link {
    border-color: #175cd3;
    background: #175cd3;
    color: #fff;
  }

  button:disabled {
    cursor: progress;
    opacity: 0.72;
  }

  .icon-button {
    width: 32px;
    min-height: 32px;
    flex: 0 0 auto;
    padding: 0;
  }

  .status-pill {
    gap: 7px;
    min-height: 36px;
    border: 1px solid #d5dde8;
    border-radius: 999px;
    padding: 0 12px;
    background: #fff;
    color: #5c6878;
    font-size: 0.9rem;
    font-weight: 750;
  }

  .status-pill.online {
    border-color: #b7e4c7;
    color: #1f7a43;
  }

  .spin {
    animation: spin 900ms linear infinite;
  }

  .notice {
    display: flex;
    gap: 12px;
    margin-bottom: 12px;
    padding: 12px 14px;
    border: 1px solid #f3b7b0;
    border-radius: 8px;
    background: #fff4f2;
    color: #7c251f;
  }

  .notice span {
    display: block;
    margin-top: 3px;
    font-size: 0.9rem;
  }

  .metrics,
  .cache-grid,
  .workspace,
  .command-grid {
    display: grid;
    gap: 12px;
  }

  .metrics {
    grid-template-columns: 0.7fr 0.7fr 1fr 1.6fr;
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
    font-size: 1.25rem;
    line-height: 1.25;
    overflow-wrap: anywhere;
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 12px;
    padding: 14px;
  }

  .toolbar p {
    margin-top: 4px;
    color: #667487;
    font-size: 0.9rem;
  }

  .toolbar-actions,
  .card-actions {
    display: flex;
    flex-wrap: wrap;
  }

  .cache-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
    margin-bottom: 12px;
  }

  .cache-card,
  .control-card,
  .command-card {
    display: grid;
    gap: 12px;
    padding: 14px;
  }

  .cache-head,
  .panel-head,
  .code-line {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 10px;
  }

  .tag {
    display: inline-flex;
    flex: 0 0 auto;
    border-radius: 999px;
    padding: 5px 9px;
    background: #edf5ff;
    color: #175cd3;
    font-size: 0.78rem;
    font-weight: 750;
  }

  .facts {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 8px;
  }

  .facts div {
    border: 1px solid #edf1f5;
    border-radius: 6px;
    padding: 9px;
    background: #f8fafc;
  }

  .facts strong {
    display: block;
    margin-top: 3px;
    overflow-wrap: anywhere;
  }

  .field {
    display: grid;
    gap: 5px;
  }

  code,
  input {
    font-family: "SFMono-Regular", Consolas, "Liberation Mono", monospace;
  }

  .field code,
  .code-line code,
  .wrap-code {
    display: block;
    min-width: 0;
    border-radius: 6px;
    background: #f3f6fa;
    color: #263548;
    font-size: 0.79rem;
    line-height: 1.45;
    overflow-wrap: anywhere;
    white-space: pre-wrap;
  }

  .field code,
  .code-line code,
  .wrap-code {
    padding: 9px 10px;
  }

  .workspace,
  .command-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
    margin-top: 12px;
  }

  label {
    display: grid;
    gap: 6px;
  }

  input {
    width: 100%;
    min-height: 36px;
    border: 1px solid #cbd5e1;
    border-radius: 6px;
    padding: 8px 10px;
    background: #fff;
    color: #17202a;
    font: inherit;
  }

  input:focus {
    border-color: #175cd3;
    outline: 3px solid rgba(23, 92, 211, 0.14);
  }

  .button-row,
  .split {
    display: grid;
    gap: 10px;
  }

  .button-row {
    grid-template-columns: 1fr 108px;
  }

  .split {
    grid-template-columns: 1fr 96px;
    align-items: end;
  }

  .check {
    grid-template-columns: 18px 1fr;
    align-items: center;
    min-height: 36px;
    gap: 8px;
  }

  .check input {
    min-height: auto;
  }

  .code-line {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 32px;
    align-items: center;
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
    .metrics,
    .cache-grid,
    .workspace,
    .command-grid {
      grid-template-columns: 1fr;
    }

    .metrics {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  @media (max-width: 680px) {
    .app {
      width: min(100% - 20px, 1440px);
      padding-top: 14px;
    }

    .topbar,
    .top-actions,
    .toolbar,
    .panel-head,
    .cache-head {
      align-items: stretch;
      flex-direction: column;
    }

    .metrics,
    .facts,
    .button-row,
    .split {
      grid-template-columns: 1fr;
    }

    h1 {
      font-size: 1.45rem;
    }
  }
</style>
