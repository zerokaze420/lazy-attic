<script>
  import { onMount } from 'svelte';
  import {
    AlertCircle,
    BookOpen,
    Box,
    Clipboard,
    Database,
    ExternalLink,
    Grid2X2,
    KeyRound,
    Layers3,
    List,
    Plus,
    RefreshCw,
    Search,
    Server,
    Sparkles,
    Terminal
  } from '@lucide/svelte';
  import { t, formatDate, formatBytes, formatRetention } from '$lib/i18n/index.svelte';

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
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      summary = await response.json();
      if (summary.admin_token?.token) {
        token = summary.admin_token.token;
        adminTokenExpires = summary.admin_token.expires_at;
        localStorage.setItem('attic.console.token', token);
      }
      const usageResponse = await fetch('/_api/web/usage');
      if (usageResponse.ok) usage = await usageResponse.json();
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
      createMessage = t('dash.tokenGenerated', { date: formatDate(payload.expires_at) });
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
      if (!name) throw new Error(t('dash.needName'));
      if (!token.trim()) throw new Error(t('dash.needToken'));
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
      createMessage = t('dash.cacheCreated', { name });
      await refresh();
    } catch (err) {
      createMessage = err instanceof Error ? err.message : String(err);
    } finally {
      createBusy = false;
    }
  }

  async function copyText(value, label) {
    if (!value || value.startsWith('<')) return;
    await navigator.clipboard.writeText(value);
    copyMessage = t('copied');
    setTimeout(() => { copyMessage = ''; }, 1800);
  }

  function matchesCache(cache) {
    const query = cacheQuery.trim().toLowerCase();
    const matchesQuery = !query || [
      cache.name, cache.store_dir, cache.substituter_endpoint,
      cache.api_endpoint, cache.public_key,
      ...(cache.upstream_cache_key_names ?? [])
    ].some((v) => String(v ?? '').toLowerCase().includes(query));
    const matchesVisibility =
      cacheVisibility === 'all' ||
      (cacheVisibility === 'public' && cache.is_public) ||
      (cacheVisibility === 'private' && !cache.is_public);
    return matchesQuery && matchesVisibility;
  }

  $: caches = summary?.caches ?? [];
  $: filteredCaches = caches.filter(matchesCache);
  $: publicCacheModel = caches.find((c) => c.is_public) || caches[0];
  $: exampleCache = caches[0]?.name || '<cache-name>';
  $: publicCache = publicCacheModel?.name || exampleCache;
  $: publicKey = publicCacheModel?.public_key || '<public-key>';
  $: latestUsage = usage?.cache_usage?.[0]?.nar_size ?? 0;
  $: apiEndpoint = publicCacheModel?.api_endpoint || `${origin}/`;
  $: substituterEndpoint = publicCacheModel?.substituter_endpoint || `${origin}/${publicCache}`;
  $: loginCommand = `attic login local ${apiEndpoint} <token>`;
  $: useCommand = `attic use local:${exampleCache}`;
  $: pushCommand = `attic push local:${exampleCache} /nix/store/<path>`;
  $: substituterCommand = `substituters = ${substituterEndpoint}`;
  $: trustedKeysCommand = `trusted-public-keys = ${publicKey}`;
  $: activeCommands = commandTab === 'client'
    ? [{ value: loginCommand, label: t('dash.loginCommand') }, { value: useCommand, label: t('dash.useCommand') }, { value: pushCommand, label: t('dash.pushCommand') }]
    : [{ value: substituterCommand, label: t('substituter') }, { value: trustedKeysCommand, label: t('publicKey') }];
</script>

<svelte:head>
  <title>Attic Console</title>
</svelte:head>

<div class="page-header">
  <div style="display:flex;align-items:center;justify-content:space-between;">
    <div>
      <h1 class="page-title">{t('dash.title')}</h1>
      <p class="page-description">{t('dash.description')}</p>
    </div>
    <div style="display:flex;gap:8px;">
      <a class="btn btn-secondary" href="/guide">
        <BookOpen size={15} />
        <span>{t('sidebar.guide')}</span>
      </a>
      <button class="btn btn-secondary" on:click={refresh} disabled={loading}>
        <span class:spin={loading}><RefreshCw size={15} /></span>
        <span>{t('refresh')}</span>
      </button>
    </div>
  </div>
</div>

<div class="page-body">
  {#if error}
    <div class="notice">
      <AlertCircle size={16} />
      <span>{error}</span>
    </div>
  {/if}

  <div class="stats-grid">
    <div class="stat-card">
      <div class="stat-icon"><Database size={18} /></div>
      <div class="stat-content">
        <span class="stat-label">{t('dash.caches')}</span>
        <span class="stat-value">{summary?.counts?.caches ?? '-'}</span>
      </div>
    </div>
    <div class="stat-card">
      <div class="stat-icon"><Box size={18} /></div>
      <div class="stat-content">
        <span class="stat-label">{t('objects')}</span>
        <span class="stat-value">{summary?.counts?.objects ?? '-'}</span>
      </div>
    </div>
    <div class="stat-card">
      <div class="stat-icon"><Layers3 size={18} /></div>
      <div class="stat-content">
        <span class="stat-label">{t('dash.nars')}</span>
        <span class="stat-value">{summary?.counts?.nars ?? '-'}</span>
      </div>
    </div>
    <div class="stat-card">
      <div class="stat-icon"><Server size={18} /></div>
      <div class="stat-content">
        <span class="stat-label">{t('storage')}</span>
        <span class="stat-value">{usage ? formatBytes(latestUsage) : '-'}</span>
      </div>
    </div>
  </div>

  <div style="display:grid;grid-template-columns:1fr 380px;gap:16px;align-items:start;">
    <div style="display:flex;flex-direction:column;gap:16px;min-width:0;">
      <div class="card">
        <div class="card-body no-padding">
          <div style="display:flex;align-items:center;gap:10px;padding:12px 16px;border-bottom:1px solid hsl(var(--border));">
            <div class="search-wrapper" style="flex:1;">
              <Search size={15} />
              <input class="input" bind:value={cacheQuery} placeholder={t('dash.searchPlaceholder')} autocomplete="off" />
            </div>
            <div class="tabs">
              <button class="tab" class:active={cacheVisibility === 'all'} on:click={() => cacheVisibility = 'all'}>{t('all')}</button>
              <button class="tab" class:active={cacheVisibility === 'public'} on:click={() => cacheVisibility = 'public'}>{t('public')}</button>
              <button class="tab" class:active={cacheVisibility === 'private'} on:click={() => cacheVisibility = 'private'}>{t('private')}</button>
            </div>
            <div class="tabs compact">
              <button class="tab" class:active={cacheView === 'cards'} on:click={() => cacheView = 'cards'}><Grid2X2 size={14} /></button>
              <button class="tab" class:active={cacheView === 'table'} on:click={() => cacheView = 'table'}><List size={14} /></button>
            </div>
          </div>

          {#if cacheView === 'cards'}
            <div style="padding:16px;">
              {#if filteredCaches.length}
                <div class="cache-cards">
                  {#each filteredCaches as cache}
                    <div class="cache-card">
                      <div class="cache-card-header">
                        <div>
                          <h3>{cache.name}</h3>
                          <p>{cache.store_dir}</p>
                        </div>
                        <span class="badge" class:badge-success={cache.is_public} class:badge-warning={!cache.is_public}>
                          {cache.is_public ? t('public') : t('private')} · P{cache.priority}
                        </span>
                      </div>
                      <div class="cache-card-facts">
                        <div class="cache-card-fact"><span>{t('objects')}</span><strong>{cache.objects}</strong></div>
                        <div class="cache-card-fact"><span>{t('retention')}</span><strong>{formatRetention(cache.retention_period)}</strong></div>
                        <div class="cache-card-fact"><span>{t('created')}</span><strong>{formatDate(cache.created_at)}</strong></div>
                      </div>
                      <div style="display:flex;flex-direction:column;gap:4px;">
                        <div style="display:flex;align-items:center;justify-content:space-between;">
                          <span style="font-size:0.72rem;color:hsl(var(--muted-foreground));">{t('substituter')}</span>
                          <button class="btn btn-ghost btn-sm btn-icon" on:click={() => copyText(cache.substituter_endpoint, 'Substituter')}>
                            <Clipboard size={13} />
                          </button>
                        </div>
                        <code class="code-inline" style="display:block;font-size:0.75rem;word-break:break-all;">{cache.substituter_endpoint}</code>
                      </div>
                      <div class="cache-card-actions">
                        <a class="btn btn-primary btn-sm" href={`/cache?name=${encodeURIComponent(cache.name)}`}>
                          <ExternalLink size={14} />
                          <span>{t('details')}</span>
                        </a>
                        <button class="btn btn-secondary btn-sm" on:click={() => copyText(cache.substituter_endpoint, 'Substituter')}>
                          <Clipboard size={14} />
                          <span>{t('dash.copyUrl')}</span>
                        </button>
                        <button class="btn btn-secondary btn-sm" on:click={() => copyText(cache.public_key, 'Public key')}>
                          <Clipboard size={14} />
                          <span>{t('dash.copyKey')}</span>
                        </button>
                      </div>
                    </div>
                  {/each}
                </div>
              {:else}
                <div class="empty">{loading ? t('dash.loadingCaches') : t('dash.noMatchingCaches')}</div>
              {/if}
            </div>
          {:else}
            <div class="table-wrapper">
              <table class="table">
                <thead>
                  <tr>
                    <th>{t('dash.name')}</th>
                    <th>{t('status')}</th>
                    <th>{t('objects')}</th>
                    <th>{t('retention')}</th>
                    <th>{t('substituter')}</th>
                    <th></th>
                  </tr>
                </thead>
                <tbody>
                  {#each filteredCaches as cache}
                    <tr>
                      <td><strong>{cache.name}</strong><br><span style="font-size:0.75rem;color:hsl(var(--muted-foreground));">{cache.store_dir}</span></td>
                      <td><span class="badge" class:badge-success={cache.is_public} class:badge-warning={!cache.is_public}>{cache.is_public ? t('public') : t('private')}</span></td>
                      <td>{cache.objects}</td>
                      <td>{formatRetention(cache.retention_period)}</td>
                      <td><code class="code-inline" style="font-size:0.72rem;max-width:280px;display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;">{cache.substituter_endpoint}</code></td>
                      <td>
                        <a class="btn btn-ghost btn-sm btn-icon" href={`/cache?name=${encodeURIComponent(cache.name)}`}>
                          <ExternalLink size={14} />
                        </a>
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
              {#if !filteredCaches.length}
                <div class="empty">{loading ? t('dash.loadingCaches') : t('dash.noMatchingCaches')}</div>
              {/if}
            </div>
          {/if}
        </div>
      </div>

      <div class="card">
        <div class="card-header">
          <div>
            <h2>{t('dash.commands')}</h2>
            <p>{commandTab === 'client' ? t('dash.commandsDesc') : t('dash.commandsDescNix')}</p>
          </div>
          <div class="tabs">
            <button class="tab" class:active={commandTab === 'client'} on:click={() => commandTab = 'client'}>
              <Terminal size={13} />
              <span>{t('dash.client')}</span>
            </button>
            <button class="tab" class:active={commandTab === 'nix'} on:click={() => commandTab = 'nix'}>
              <Server size={13} />
              <span>{t('dash.nix')}</span>
            </button>
          </div>
        </div>
        <div class="card-body no-padding">
          {#each activeCommands as command}
            <div style="display:flex;align-items:center;justify-content:space-between;padding:10px 16px;border-bottom:1px solid hsl(var(--border));">
              <div style="min-width:0;flex:1;">
                <span style="font-size:0.72rem;font-weight:600;color:hsl(var(--muted-foreground));">{command.label}</span>
                <code style="display:block;font-family:monospace;font-size:0.78rem;color:hsl(var(--foreground));margin-top:2px;word-break:break-all;">{command.value}</code>
              </div>
              <button class="btn btn-ghost btn-sm btn-icon" title={`${t('copy')} ${command.label}`} on:click={() => copyText(command.value, command.label)}>
                <Clipboard size={13} />
              </button>
            </div>
          {/each}
        </div>
      </div>
    </div>

    <div style="display:flex;flex-direction:column;gap:16px;">
      <div class="card">
        <div class="card-header">
          <div>
            <h2>{t('dash.adminToken')}</h2>
            <p>{t('dash.adminTokenDesc')}</p>
          </div>
          <KeyRound size={18} style="color:hsl(var(--muted-foreground));" />
        </div>
        <div class="card-body">
          <code style="display:block;padding:8px 10px;border-radius:var(--radius);background:hsl(var(--muted));font-family:monospace;font-size:0.75rem;word-break:break-all;line-height:1.5;color:hsl(var(--foreground));">{token || t('waitServer')}</code>
          {#if adminTokenExpires}
            <p style="font-size:0.75rem;color:hsl(var(--muted-foreground));margin-top:8px;">{t('dash.expiresAt')} {formatDate(adminTokenExpires)}</p>
          {/if}
          <div style="display:grid;grid-template-columns:1fr auto;gap:8px;margin-top:12px;">
            <button class="btn btn-primary" on:click={issueAdminToken} disabled={tokenBusy}>
              <Sparkles size={15} />
              <span>{tokenBusy ? t('dash.regenerating') : t('dash.regenerate')}</span>
            </button>
            <button class="btn btn-secondary btn-icon" on:click={() => copyText(token, t('dash.adminToken'))}>
              <Clipboard size={15} />
            </button>
          </div>
        </div>
      </div>

      <div class="card">
        <div class="card-header">
          <div>
            <h2>{t('dash.createCache')}</h2>
            <p>{t('dash.createCacheDesc')}</p>
          </div>
          <Plus size={18} style="color:hsl(var(--muted-foreground));" />
        </div>
        <div class="card-body">
          <div style="display:flex;flex-direction:column;gap:12px;">
            <label class="label">
              <span>{t('dash.name')}</span>
              <input class="input" bind:value={create.name} placeholder="main" autocomplete="off" />
            </label>
            <label class="label">
              <span>{t('dash.storeDir')}</span>
              <input class="input" bind:value={create.storeDir} />
            </label>
            <div class="form-row">
              <label class="label">
                <span>{t('dash.priority')}</span>
                <input class="input" bind:value={create.priority} type="number" />
              </label>
              <label class="checkbox-label" style="align-self:end;height:36px;">
                <input bind:checked={create.isPublic} type="checkbox" />
                <span>{t('dash.isPublic')}</span>
              </label>
            </div>
            <button class="btn btn-primary" on:click={createCache} disabled={createBusy} style="width:100%;">
              <Plus size={15} />
              <span>{createBusy ? t('dash.creating') : t('dash.create')}</span>
            </button>
            {#if createMessage}
              <p style="font-size:0.78rem;color:hsl(var(--muted-foreground));">{createMessage}</p>
            {/if}
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

{#if copyMessage}
  <div class="toast">{copyMessage}</div>
{/if}
