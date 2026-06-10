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

  function saveToken() {
    localStorage.setItem('attic.console.token', token.trim());
    createMessage = 'Token 已保存在当前浏览器。';
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

  $: caches = summary?.caches ?? [];
  $: publicCacheModel = caches.find((cache) => cache.is_public) || caches[0];
  $: exampleCache = caches[0]?.name || '<缓存名>';
  $: publicCache = publicCacheModel?.name || exampleCache;
  $: publicKey = publicCacheModel?.public_key || '<缓存公钥>';
  $: loginCommand = `attic login local ${origin}/ <token>`;
  $: useCommand = `attic use local:${exampleCache}`;
  $: pushCommand = `attic push local:${exampleCache} /nix/store/<path>`;
  $: substituterCommand = `substituters = ${origin}/${publicCache}`;
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
  <header class="topbar motion-card">
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
    <section class="notice motion-card">
      <AlertCircle size={18} />
      <div>
        <strong>控制台接口不可用</strong>
        <span>{error}</span>
      </div>
    </section>
  {/if}

  <section class="metrics" aria-label="服务指标">
    <article class="metric motion-card">
      <Database size={20} />
      <span>缓存</span>
      <strong>{summary?.counts?.caches ?? '-'}</strong>
    </article>
    <article class="metric motion-card">
      <Box size={20} />
      <span>对象</span>
      <strong>{summary?.counts?.objects ?? '-'}</strong>
    </article>
    <article class="metric motion-card">
      <Layers3 size={20} />
      <span>NAR / 分块</span>
      <strong>{summary ? `${summary.counts.nars} / ${summary.counts.chunks}` : '-'}</strong>
    </article>
    <article class="metric motion-card">
      <ShieldCheck size={20} />
      <span>存储</span>
      <strong>{summary?.storage?.kind ?? '-'}</strong>
    </article>
  </section>

  <section class="workspace">
    <div class="panel cache-panel motion-card">
      <div class="panel-head">
        <div>
          <h2>缓存列表</h2>
          <p>public key 已公开显示，可直接复制到 Nix 配置。</p>
        </div>
        <span>{summary?.storage?.location ?? '等待服务数据'}</span>
      </div>

      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>名称</th>
              <th>状态</th>
              <th>对象</th>
              <th>Public Key</th>
              <th>创建时间</th>
              <th>端点</th>
            </tr>
          </thead>
          <tbody>
            {#if caches.length}
              {#each caches as cache}
                <tr>
                  <td>
                    <strong>{cache.name}</strong>
                    <span>{cache.store_dir}</span>
                  </td>
                  <td>
                    <span class="tag">{cache.is_public ? '公开' : '私有'} · P{cache.priority}</span>
                  </td>
                  <td>{cache.objects}</td>
                  <td class="key-cell">
                    <code>{cache.public_key}</code>
                    <button
                      class="icon-button"
                      type="button"
                      title="复制 public key"
                      on:click={() => copyText(cache.public_key, 'Public key')}
                    >
                      <Clipboard size={15} />
                    </button>
                  </td>
                  <td>{formatDate(cache.created_at)}</td>
                  <td>
                    <a href={`/${cache.name}/nix-cache-info`}>
                      <ExternalLink size={15} />
                      缓存信息
                    </a>
                  </td>
                </tr>
              {/each}
            {:else}
              <tr>
                <td class="empty" colspan="6">
                  {loading ? '正在加载缓存...' : '还没有创建缓存。'}
                </td>
              </tr>
            {/if}
          </tbody>
        </table>
      </div>
    </div>

    <aside class="side-stack">
      <section class="panel motion-card">
        <div class="panel-head compact">
          <div>
            <h2>管理员 Token</h2>
            <p>前端受保护，Token 在页面中直接公开。</p>
          </div>
          <KeyRound size={20} />
        </div>
        <textarea bind:value={token} rows="5" spellcheck="false"></textarea>
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

      <section class="panel motion-card">
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
    </aside>
  </section>

  <section class="command-grid">
    <article class="panel command-card motion-card">
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
          <button
            class="icon-button"
            type="button"
            title={`复制${command.label}`}
            on:click={() => copyText(command.value, command.label)}
          >
            <Clipboard size={15} />
          </button>
        </div>
      {/each}
    </article>

    <article class="panel command-card motion-card">
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
          <button
            class="icon-button"
            type="button"
            title={`复制${command.label}`}
            on:click={() => copyText(command.value, command.label)}
          >
            <Clipboard size={15} />
          </button>
        </div>
      {/each}
    </article>
  </section>

  <section class="next-step panel motion-card">
    <div class="panel-head">
      <div>
        <h2>完整使用教程</h2>
        <p>初始化缓存、客户端登录、NixOS 配置和常见错误排查已经移到独立页面。</p>
      </div>
      <a class="nav-link primary-link" href="/guide">
        <BookOpen size={16} />
        <span>打开教程</span>
      </a>
    </div>
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
    background:
      radial-gradient(circle at 20% 0%, rgba(10, 111, 210, 0.12), transparent 28rem),
      linear-gradient(180deg, #f6f8fb 0%, #eef2f6 100%);
    color: #17202a;
    font-family:
      Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont,
      "Segoe UI", sans-serif;
  }

  .app {
    width: min(1320px, calc(100% - 32px));
    margin: 0 auto;
    padding: 24px 0 44px;
  }

  .motion-card {
    opacity: 1;
    animation: card-in 420ms ease-out both;
  }

  .motion-card:nth-child(2) {
    animation-delay: 35ms;
  }

  .motion-card:nth-child(3) {
    animation-delay: 70ms;
  }

  .motion-card:nth-child(4) {
    animation-delay: 105ms;
  }

  .topbar,
  .brand,
  .top-actions,
  button,
  .status-pill,
  .key-cell,
  .nav-link,
  a {
    display: flex;
    align-items: center;
  }

  .topbar {
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 18px;
  }

  .brand {
    gap: 12px;
  }

  .brand-mark {
    display: grid;
    place-items: center;
    width: 44px;
    height: 44px;
    border: 1px solid #c8d7ea;
    border-radius: 8px;
    background: #ffffff;
    color: #175cd3;
    box-shadow: 0 14px 32px rgba(15, 34, 58, 0.08);
  }

  .top-actions {
    gap: 10px;
  }

  h1,
  h2,
  p {
    margin: 0;
  }

  h1 {
    margin-top: 3px;
    font-size: 1.9rem;
    letter-spacing: 0;
  }

  h2 {
    font-size: 1rem;
    letter-spacing: 0;
  }

  .eyebrow {
    color: #657080;
    font-size: 0.76rem;
    font-weight: 800;
    letter-spacing: 0;
    text-transform: uppercase;
  }

  button {
    justify-content: center;
    gap: 8px;
    min-height: 38px;
    border: 1px solid #175cd3;
    border-radius: 6px;
    padding: 0 13px;
    background: #175cd3;
    color: #fff;
    font: inherit;
    font-weight: 750;
    cursor: pointer;
    transition:
      transform 140ms ease,
      background 140ms ease,
      border-color 140ms ease,
      box-shadow 140ms ease;
  }

  button:hover {
    background: #0f4eb8;
    box-shadow: 0 10px 22px rgba(23, 92, 211, 0.18);
    transform: translateY(-1px);
  }

  button:disabled {
    cursor: progress;
    opacity: 0.72;
  }

  .secondary {
    border-color: #cad2df;
    background: #fff;
    color: #263548;
  }

  .secondary:hover {
    border-color: #aebacf;
    background: #f7f9fc;
    box-shadow: 0 8px 18px rgba(15, 34, 58, 0.08);
  }

  .nav-link {
    justify-content: center;
    gap: 8px;
    min-height: 38px;
    border: 1px solid #cad2df;
    border-radius: 6px;
    padding: 0 13px;
    background: #fff;
    color: #263548;
    font-size: 0.95rem;
    font-weight: 750;
    text-decoration: none;
    transition:
      transform 140ms ease,
      background 140ms ease,
      border-color 140ms ease,
      box-shadow 140ms ease;
  }

  .nav-link:hover {
    border-color: #aebacf;
    background: #f7f9fc;
    box-shadow: 0 8px 18px rgba(15, 34, 58, 0.08);
    transform: translateY(-1px);
  }

  .primary-link {
    border-color: #175cd3;
    background: #175cd3;
    color: #fff;
  }

  .primary-link:hover {
    border-color: #0f4eb8;
    background: #0f4eb8;
    box-shadow: 0 10px 22px rgba(23, 92, 211, 0.18);
  }

  .icon-button {
    width: 32px;
    min-height: 32px;
    flex: 0 0 auto;
    padding: 0;
  }

  .status-pill {
    gap: 7px;
    min-height: 38px;
    border: 1px solid #d5dde8;
    border-radius: 999px;
    padding: 0 13px;
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
    margin-bottom: 16px;
    padding: 13px 15px;
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

  .metrics {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 12px;
    margin-bottom: 16px;
  }

  .metric,
  .panel {
    border: 1px solid #dbe3ee;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.92);
    box-shadow: 0 18px 40px rgba(15, 34, 58, 0.08);
  }

  .metric {
    display: grid;
    grid-template-columns: 24px 1fr;
    gap: 6px 10px;
    padding: 16px;
    color: #175cd3;
  }

  .metric span {
    color: #667487;
    font-size: 0.86rem;
    font-weight: 700;
  }

  .metric strong {
    grid-column: 2;
    color: #17202a;
    font-size: 1.55rem;
    letter-spacing: 0;
  }

  .workspace {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 380px;
    gap: 16px;
    align-items: start;
  }

  .panel {
    min-width: 0;
    overflow: hidden;
  }

  .panel-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 16px 18px;
    border-bottom: 1px solid #e6ebf2;
  }

  .panel-head.compact {
    padding-bottom: 12px;
    border-bottom: 0;
  }

  .panel-head p,
  .panel-head span,
  .hint,
  label span,
  td span {
    color: #667487;
    font-size: 0.86rem;
  }

  .panel-head p {
    margin-top: 4px;
  }

  .table-wrap {
    overflow-x: auto;
  }

  table {
    width: 100%;
    min-width: 1040px;
    border-collapse: collapse;
  }

  th,
  td {
    padding: 13px 18px;
    border-bottom: 1px solid #edf1f5;
    text-align: left;
    vertical-align: middle;
    font-size: 0.92rem;
  }

  th {
    color: #526176;
    font-size: 0.78rem;
    text-transform: uppercase;
  }

  td strong,
  td span {
    display: block;
  }

  td span {
    margin-top: 4px;
  }

  td a {
    gap: 6px;
    color: #175cd3;
    font-weight: 750;
    text-decoration: none;
  }

  .tag {
    display: inline-flex;
    width: fit-content;
    border-radius: 999px;
    padding: 5px 9px;
    background: #edf5ff;
    color: #175cd3;
    font-size: 0.78rem;
    font-weight: 750;
  }

  .key-cell {
    gap: 8px;
    max-width: 340px;
  }

  code,
  textarea {
    font-family: "SFMono-Regular", Consolas, "Liberation Mono", monospace;
  }

  .key-cell code,
  .code-line code {
    overflow-x: auto;
    border-radius: 6px;
    background: #f3f6fa;
    color: #263548;
    font-size: 0.8rem;
    line-height: 1.5;
    white-space: nowrap;
  }

  .key-cell code {
    display: block;
    padding: 7px 9px;
  }

  .empty {
    height: 170px;
    color: #667487;
    text-align: center;
  }

  .side-stack {
    display: grid;
    gap: 16px;
  }

  .side-stack .panel {
    display: grid;
    gap: 12px;
    padding: 16px;
  }

  .side-stack .panel-head {
    padding: 0;
  }

  label {
    display: grid;
    gap: 6px;
  }

  input,
  textarea {
    width: 100%;
    min-height: 38px;
    border: 1px solid #cbd5e1;
    border-radius: 6px;
    padding: 8px 10px;
    background: #fff;
    color: #17202a;
    font: inherit;
  }

  textarea {
    min-height: 128px;
    resize: vertical;
    font-size: 0.78rem;
    line-height: 1.5;
  }

  input:focus,
  textarea:focus {
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
    min-height: 38px;
    gap: 8px;
  }

  .check input {
    min-height: auto;
  }

  .command-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 16px;
    margin-top: 16px;
  }

  .command-card {
    display: grid;
    gap: 10px;
    padding: 16px;
  }

  .command-card .panel-head {
    padding: 0 0 4px;
  }

  .code-line {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 32px;
    gap: 8px;
    align-items: center;
  }

  .code-line code {
    display: block;
    padding: 10px 11px;
  }

  .next-step {
    margin-top: 16px;
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
    box-shadow: 0 18px 40px rgba(15, 34, 58, 0.16);
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  @keyframes card-in {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @media (max-width: 980px) {
    .metrics,
    .workspace,
    .command-grid {
      grid-template-columns: 1fr;
    }

    .metrics {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

  }

  @media (max-width: 600px) {
    .app {
      width: min(100% - 20px, 1320px);
      padding-top: 14px;
    }

    .topbar,
    .top-actions,
    .panel-head {
      align-items: stretch;
      flex-direction: column;
    }

    .metrics,
    .button-row {
      grid-template-columns: 1fr;
    }

    h1 {
      font-size: 1.55rem;
    }
  }
</style>
