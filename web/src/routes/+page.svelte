<script>
  import { onMount } from 'svelte';
  import {
    AlertCircle,
    Activity,
    BarChart3,
    BookOpen,
    Box,
    CheckCircle2,
    Clipboard,
    Database,
    ExternalLink,
    Filter,
    Grid2X2,
    KeyRound,
    Layers3,
    List,
    PieChart,
    Plus,
    RefreshCw,
    Search,
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
  let usage = null;
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
  let cacheQuery = '';
  let cacheVisibility = 'all';
  let cacheView = 'cards';
  let commandTab = 'client';
  let usageTab = 'overview';

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

      const usageResponse = await fetch('/_api/web/usage');
      if (usageResponse.ok) {
        usage = await usageResponse.json();
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

  function formatBytes(value) {
    if (!Number.isFinite(value)) {
      return '-';
    }
    const units = ['B', 'KiB', 'MiB', 'GiB', 'TiB'];
    let size = Math.max(0, value);
    let unit = 0;
    while (size >= 1024 && unit < units.length - 1) {
      size /= 1024;
      unit += 1;
    }
    return `${size >= 10 || unit === 0 ? size.toFixed(0) : size.toFixed(1)} ${units[unit]}`;
  }

  function maxBucketValue(items) {
    return Math.max(1, ...(items ?? []).map((item) => item.nar_size || item.count || 0));
  }

  function bucketWidth(item, items) {
    return `${Math.max(8, ((item.nar_size || item.count || 0) / maxBucketValue(items)) * 100)}%`;
  }

  function healthLabel(score) {
    if (score >= 90) return '健康';
    if (score >= 70) return '需关注';
    return '异常';
  }

  function scoreColor(score) {
    if (score >= 90) return '#247347';
    if (score >= 70) return '#8a5a18';
    return '#b42318';
  }

  function joinList(values) {
    return values?.length ? values.join(', ') : '无';
  }

  function matchesCache(cache) {
    const query = cacheQuery.trim().toLowerCase();
    const matchesQuery = !query || [
      cache.name,
      cache.store_dir,
      cache.substituter_endpoint,
      cache.api_endpoint,
      cache.public_key,
      ...(cache.upstream_cache_key_names ?? [])
    ].some((value) => String(value ?? '').toLowerCase().includes(query));

    const matchesVisibility =
      cacheVisibility === 'all' ||
      (cacheVisibility === 'public' && cache.is_public) ||
      (cacheVisibility === 'private' && !cache.is_public);

    return matchesQuery && matchesVisibility;
  }

  $: caches = summary?.caches ?? [];
  $: filteredCaches = caches.filter(matchesCache);
  $: publicCacheModel = caches.find((cache) => cache.is_public) || caches[0];
  $: exampleCache = caches[0]?.name || '<缓存名>';
  $: publicCache = publicCacheModel?.name || exampleCache;
  $: publicKey = publicCacheModel?.public_key || '<缓存公钥>';
  $: topCacheUsage = usage?.cache_usage?.[0];
  $: recentActivity = usage?.recent_uploads ?? [];
  $: healthIssues = usage?.health?.issues ?? [];
  $: healthScore = usage?.health?.score ?? 0;
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
  $: activeCommands = commandTab === 'client' ? clientCommands : nixCommands;
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

  <section class="hero-panel panel">
    <div>
      <p class="eyebrow">Workspace</p>
      <h2>缓存工作台</h2>
    </div>
    <div class="quick-actions">
      <button type="button" on:click={() => copyText(substituterEndpoint, 'Substituter')}>
        <Clipboard size={16} />
        <span>Substituter</span>
      </button>
      <button class="secondary" type="button" on:click={() => copyText(publicKey, 'Public key')}>
        <Clipboard size={16} />
        <span>Public key</span>
      </button>
    </div>
  </section>

  <section class="panel usage-panel" aria-label="缓存使用情况">
    <div class="panel-head compact">
      <div>
        <p class="eyebrow">Insights</p>
        <h2>使用洞察</h2>
      </div>
      <div class="segmented">
        <button class:active={usageTab === 'overview'} class="secondary" type="button" on:click={() => usageTab = 'overview'}>
          <BarChart3 size={15} />
          <span>概览</span>
        </button>
        <button class:active={usageTab === 'health'} class="secondary" type="button" on:click={() => usageTab = 'health'}>
          <ShieldCheck size={15} />
          <span>健康</span>
        </button>
        <button class:active={usageTab === 'profile'} class="secondary" type="button" on:click={() => usageTab = 'profile'}>
          <PieChart size={15} />
          <span>画像</span>
        </button>
        <button class:active={usageTab === 'activity'} class="secondary" type="button" on:click={() => usageTab = 'activity'}>
          <Activity size={15} />
          <span>活动</span>
        </button>
      </div>
    </div>

    {#if usageTab === 'overview'}
      <div class="visual-overview">
        <div class="score-ring" style={`--score: ${healthScore}; --score-color: ${scoreColor(healthScore)}`}>
          <strong>{usage ? healthScore : '-'}</strong>
          <span>健康</span>
        </div>
        <div class="visual-stat">
          <strong>{usage ? formatBytes(usage.totals.logical_nar_size) : '-'}</strong>
          <span>逻辑占用</span>
          <div class="bar"><span style="width: 100%"></span></div>
        </div>
        <div class="visual-stat">
          <strong>{usage ? formatBytes(usage.totals.nar_size) : '-'}</strong>
          <span>唯一 NAR</span>
          <div class="bar"><span style={`width: ${usage?.totals?.logical_nar_size ? Math.max(8, (usage.totals.nar_size / usage.totals.logical_nar_size) * 100) : 0}%`}></span></div>
        </div>
        <div class="visual-stat">
          <strong>{topCacheUsage ? `${topCacheUsage.cache.name} · ${formatBytes(topCacheUsage.nar_size)}` : '-'}</strong>
          <span>最大缓存</span>
          <div class="bar"><span style={`width: ${topCacheUsage ? bucketWidth(topCacheUsage, usage.cache_usage) : '0%'}`}></span></div>
        </div>
      </div>
      <div class="rank-list compact-list">
        {#each (usage?.cache_usage ?? []).slice(0, 3) as item}
          <div class="rank-row">
            <div>
              <strong>{item.cache.name}</strong>
              <span>{item.cache.objects} 对象 · {formatBytes(item.nar_size)}</span>
            </div>
            <div class="bar"><span style={`width: ${bucketWidth(item, usage.cache_usage)}`}></span></div>
          </div>
        {:else}
          <p class="hint">暂无缓存使用数据。</p>
        {/each}
      </div>
    {:else if usageTab === 'health'}
      <div class="health-focus">
        <div class="score-ring large" style={`--score: ${healthScore}; --score-color: ${scoreColor(healthScore)}`}>
          <strong>{usage ? healthScore : '-'}</strong>
          <span>{usage ? healthLabel(healthScore) : '等待'}</span>
        </div>
        <div class="health-bars">
          <div><span>不完整</span><strong>{usage?.totals?.incomplete_objects ?? '-'}</strong></div>
          <div><span>未访问</span><strong>{usage?.totals?.never_accessed_objects ?? '-'}</strong></div>
          <div><span>建议</span><strong>{healthIssues.length}</strong></div>
        </div>
      </div>
      <div class="issue-list compact-list">
        {#each healthIssues.slice(0, 5) as issue}
          <div class={`issue ${issue.severity}`}>
            <strong>{issue.title}</strong>
            <span>{issue.cache ? `${issue.cache} · ` : ''}{issue.detail}</span>
          </div>
        {:else}
          <div class="issue ok">
            <strong>没有发现明显问题</strong>
            <span>不完整对象、空缓存和长期未访问对象会显示在这里。</span>
          </div>
        {/each}
      </div>
    {:else if usageTab === 'profile'}
      <div class="bucket-columns">
        <div>
          <h3>System</h3>
          {#each (usage?.systems ?? []).slice(0, 5) as item}
            <div class="bucket-row">
              <span>{item.name}</span>
              <strong>{item.count}</strong>
              <div class="bar"><span style={`width: ${bucketWidth(item, usage.systems)}`}></span></div>
            </div>
          {:else}
            <p class="hint">暂无 system 数据。</p>
          {/each}
        </div>
        <div>
          <h3>Compression</h3>
          {#each (usage?.compressions ?? []).slice(0, 5) as item}
            <div class="bucket-row">
              <span>{item.name}</span>
              <strong>{item.count}</strong>
              <div class="bar"><span style={`width: ${bucketWidth(item, usage.compressions)}`}></span></div>
            </div>
          {:else}
            <p class="hint">暂无压缩数据。</p>
          {/each}
        </div>
      </div>
    {:else}
      <div class="activity-list compact-list">
        {#each recentActivity.slice(0, 6) as object}
          <button class="activity-row secondary" type="button" on:click={() => copyText(object.store_path, 'Store path')}>
            <strong>{object.store_path}</strong>
            <span>{formatBytes(object.nar.nar_size)} · {object.system || 'unknown'} · {formatDate(object.created_at)}</span>
          </button>
        {:else}
          <p class="hint">暂无上传记录。</p>
        {/each}
      </div>
    {/if}
  </section>

  <section class="console-layout">
    <section class="main-column">
      <section class="panel cache-toolbar">
        <label class="search-field">
          <Search size={16} />
          <input bind:value={cacheQuery} placeholder="搜索缓存、endpoint、public key" autocomplete="off" />
        </label>
        <div class="segmented" aria-label="缓存可见性">
          <button class:active={cacheVisibility === 'all'} class="secondary" type="button" on:click={() => cacheVisibility = 'all'}>
            <Filter size={15} />
            <span>全部</span>
          </button>
          <button class:active={cacheVisibility === 'public'} class="secondary" type="button" on:click={() => cacheVisibility = 'public'}>公开</button>
          <button class:active={cacheVisibility === 'private'} class="secondary" type="button" on:click={() => cacheVisibility = 'private'}>私有</button>
        </div>
        <div class="segmented compact" aria-label="缓存视图">
          <button class:active={cacheView === 'cards'} class="secondary" type="button" title="卡片视图" on:click={() => cacheView = 'cards'}>
            <Grid2X2 size={15} />
          </button>
          <button class:active={cacheView === 'table'} class="secondary" type="button" title="表格视图" on:click={() => cacheView = 'table'}>
            <List size={15} />
          </button>
        </div>
      </section>

      {#if cacheView === 'cards'}
        <section class="cache-grid">
          {#if filteredCaches.length}
            {#each filteredCaches as cache}
              <article class="cache-card panel">
                <div class="cache-head">
                  <div>
                    <h3>{cache.name}</h3>
                    <p>{cache.store_dir}</p>
                  </div>
                  <span class:private={!cache.is_public} class="tag">{cache.is_public ? '公开' : '私有'} · P{cache.priority}</span>
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
                  <span>Public key</span>
                  <code>{cache.public_key}</code>
                </div>
                <div class="card-actions">
                  <a class="nav-link primary-link" href={`/cache?name=${encodeURIComponent(cache.name)}`}>
                    <ExternalLink size={16} />
                    <span>详情</span>
                  </a>
                  <button class="secondary" type="button" on:click={() => copyText(cache.substituter_endpoint, 'Substituter')}>
                    <Clipboard size={16} />
                    <span>地址</span>
                  </button>
                  <button class="secondary" type="button" on:click={() => copyText(cache.public_key, 'Public key')}>
                    <Clipboard size={16} />
                    <span>Key</span>
                  </button>
                </div>
              </article>
            {/each}
          {:else}
            <div class="empty panel">{loading ? '正在加载缓存...' : '没有匹配的缓存。'}</div>
          {/if}
        </section>
      {:else}
        <section class="panel cache-table-wrap">
          <table class="cache-table">
            <thead>
              <tr>
                <th>名称</th>
                <th>状态</th>
                <th>对象</th>
                <th>保留</th>
                <th>Substituter</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {#each filteredCaches as cache}
                <tr>
                  <td><strong>{cache.name}</strong><span>{cache.store_dir}</span></td>
                  <td><span class:private={!cache.is_public} class="tag">{cache.is_public ? '公开' : '私有'} · P{cache.priority}</span></td>
                  <td>{cache.objects}</td>
                  <td>{formatRetention(cache.retention_period)}</td>
                  <td><code>{cache.substituter_endpoint}</code></td>
                  <td>
                    <a class="nav-link primary-link" href={`/cache?name=${encodeURIComponent(cache.name)}`}>
                      <ExternalLink size={15} />
                    </a>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
          {#if !filteredCaches.length}
            <div class="empty">{loading ? '正在加载缓存...' : '没有匹配的缓存。'}</div>
          {/if}
        </section>
      {/if}
    </section>

    <aside class="side-column">
      <section class="panel control-card token-card">
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
    </aside>
  </section>

  <section class="panel command-dock">
      <div class="panel-head compact">
        <div>
          <h2>命令面板</h2>
          <p>{commandTab === 'client' ? '登录、启用缓存并推送 store path。' : '公开缓存可直接作为 substituter。'}</p>
        </div>
        <div class="segmented">
          <button class:active={commandTab === 'client'} class="secondary" type="button" on:click={() => commandTab = 'client'}>
            <Terminal size={15} />
            <span>客户端</span>
          </button>
          <button class:active={commandTab === 'nix'} class="secondary" type="button" on:click={() => commandTab = 'nix'}>
            <ShieldCheck size={15} />
            <span>Nix</span>
          </button>
        </div>
      </div>
      <div class="command-list">
      {#each activeCommands as command}
        <div class="code-line">
          <code>{command.value}</code>
          <button class="icon-button" type="button" title={`复制${command.label}`} on:click={() => copyText(command.value, command.label)}>
            <Clipboard size={15} />
          </button>
        </div>
      {/each}
      </div>
  </section>

  {#if copyMessage}
    <div class="toast">{copyMessage}</div>
  {/if}
</main>
