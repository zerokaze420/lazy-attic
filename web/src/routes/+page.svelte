<script>
  import { onMount } from 'svelte';
  import {
    AlertCircle,
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
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Input } from '$lib/components/ui/input';

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
  let storageTab = 'current';
  let s3Form = { region: 'us-east-1', bucket: '', endpoint: '', access_key_id: '', secret_access_key: '' };
  let storageBusy = false;
  let storageMessage = '';
  let copyState = '';

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

  async function saveStorageConfig() {
    storageBusy = true;
    storageMessage = '';
    try {
      const response = await fetch('/_api/web/storage', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(s3Form)
      });
      const result = await response.json();
      if (!response.ok) throw new Error(result.message || `HTTP ${response.status}`);
      storageMessage = result.message;
      await refresh();
    } catch (err) {
      storageMessage = err instanceof Error ? err.message : String(err);
    } finally {
      storageBusy = false;
    }
  }

  async function copyText(value, label) {
    if (!value || value.startsWith('<')) return;
    try {
      await navigator.clipboard.writeText(value);
      copyMessage = t('cache.copyToast', { label });
      copyState = 'success';
    } catch (err) {
      copyMessage = err instanceof Error ? err.message : String(err);
      copyState = 'error';
    } finally {
      setTimeout(() => {
        copyMessage = '';
        copyState = '';
      }, 1800);
    }
  }

  function matchesCache(cache, queryText, visibility) {
    const query = queryText.trim().toLowerCase();
    const matchesQuery = !query || [
      cache.name, cache.store_dir, cache.substituter_endpoint,
      cache.api_endpoint, cache.public_key,
      ...(cache.upstream_cache_key_names ?? [])
    ].some((v) => String(v ?? '').toLowerCase().includes(query));
    const matchesVisibility =
      visibility === 'all' ||
      (visibility === 'public' && cache.is_public) ||
      (visibility === 'private' && !cache.is_public);
    return matchesQuery && matchesVisibility;
  }

  $: caches = summary?.caches ?? [];
  $: filteredCaches = caches.filter((cache) => matchesCache(cache, cacheQuery, cacheVisibility));
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
  <div class="page-header-row">
    <div class="page-heading">
      <h1 class="page-title">{t('dash.title')}</h1>
      <p class="page-description">{t('dash.description')}</p>
    </div>
    <div class="page-actions">
      <Button variant="secondary" on:click={refresh} disabled={loading}>
        <span class:spin={loading}><RefreshCw size={15} /></span>
        <span>{t('refresh')}</span>
      </Button>
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

  <div class="dashboard-grid">
    <div class="stack">
      <div class="card">
        <div class="card-body no-padding">
          <div class="panel-toolbar">
            <div class="search-wrapper panel-toolbar-main">
              <Search size={15} />
              <Input bind:value={cacheQuery} placeholder={t('dash.searchPlaceholder')} autocomplete="off" />
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
            <div class="panel-body">
              {#if filteredCaches.length}
                <div class="cache-cards">
                  {#each filteredCaches as cache}
                    <div class="cache-card">
                      <div class="cache-card-header">
                        <div>
                          <h3>{cache.name}</h3>
                          <p>{cache.store_dir}</p>
                        </div>
                        <Badge variant="secondary" class={cache.is_public ? 'status-public' : 'status-private'}>
                          {cache.is_public ? t('public') : t('private')} · P{cache.priority}
                        </Badge>
                      </div>
                      <div class="cache-card-facts">
                        <div class="cache-card-fact"><span>{t('objects')}</span><strong>{cache.objects}</strong></div>
                        <div class="cache-card-fact"><span>{t('retention')}</span><strong>{formatRetention(cache.retention_period)}</strong></div>
                        <div class="cache-card-fact"><span>{t('created')}</span><strong>{formatDate(cache.created_at)}</strong></div>
                      </div>
                      <div class="cache-card-field">
                        <div class="cache-card-field-head">
                          <span class="field-label">{t('substituter')}</span>
                          <Button variant="ghost" size="icon" class="size-8" on:click={() => copyText(cache.substituter_endpoint, 'Substituter')}>
                            <Clipboard size={13} />
                          </Button>
                        </div>
                        <code class="code-inline block">{cache.substituter_endpoint}</code>
                      </div>
                      <div class="cache-card-actions">
                        <Button size="sm" href={`/cache?name=${encodeURIComponent(cache.name)}`}>
                          <ExternalLink size={14} />
                          <span>{t('details')}</span>
                        </Button>
                        <Button variant="secondary" size="sm" on:click={() => copyText(cache.substituter_endpoint, 'Substituter')}>
                          <Clipboard size={14} />
                          <span>{t('dash.copyUrl')}</span>
                        </Button>
                        <Button variant="secondary" size="sm" on:click={() => copyText(cache.public_key, 'Public key')}>
                          <Clipboard size={14} />
                          <span>{t('dash.copyKey')}</span>
                        </Button>
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
                      <td><strong>{cache.name}</strong><br><span class="meta-text">{cache.store_dir}</span></td>
                      <td><Badge variant="secondary" class={cache.is_public ? 'status-public' : 'status-private'}>{cache.is_public ? t('public') : t('private')}</Badge></td>
                      <td>{cache.objects}</td>
                      <td>{formatRetention(cache.retention_period)}</td>
                      <td><code class="code-inline truncate">{cache.substituter_endpoint}</code></td>
                      <td>
                        <Button variant="ghost" size="icon" class="size-8" href={`/cache?name=${encodeURIComponent(cache.name)}`}>
                          <ExternalLink size={14} />
                        </Button>
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
            <div class="command-row">
              <div class="command-body">
                <span class="field-label">{command.label}</span>
                <code>{command.value}</code>
              </div>
              <Button variant="ghost" size="icon" class="size-8" title={`${t('copy')} ${command.label}`} on:click={() => copyText(command.value, command.label)}>
                <Clipboard size={13} />
              </Button>
            </div>
          {/each}
        </div>
      </div>
    </div>

    <div class="stack">
      <div class="card">
        <div class="card-header">
          <div>
            <h2>{t('storage')}</h2>
            <p>{t('storageDesc') || '后端类型：' + (summary?.storage?.kind === 's3' ? 'S3' : 'Local')}</p>
          </div>
          <Database size={18} class="muted-icon" />
        </div>
        <div class="card-body">
          <div class="tabs">
            <button class="tab" class:active={storageTab === 'current'} on:click={() => storageTab = 'current'}>{t('dash.current') || '当前'}</button>
            <button class="tab" class:active={storageTab === 's3'} on:click={() => storageTab = 's3'}>S3</button>
          </div>
          {#if storageTab === 'current'}
            <div class="stack stack-sm mt-sm">
              <div class="surface">
                <span class="meta-text">{t('dash.storageKind') || '后端类型'}</span>
                <strong>{summary?.storage?.kind === 's3' ? 'S3' : 'Local'}</strong>
              </div>
              <div class="surface">
                <span class="meta-text">{summary?.storage?.kind === 's3' ? 'Bucket' : '路径'}</span>
                <strong>{summary?.storage?.location || '-'}</strong>
              </div>
            </div>
          {:else}
            <div class="stack stack-sm mt-sm">
              <label class="label">
                <span>Region</span>
                <Input bind:value={s3Form.region} placeholder="us-east-1" />
              </label>
              <label class="label">
                <span>Bucket</span>
                <Input bind:value={s3Form.bucket} placeholder="attic-cache" />
              </label>
              <label class="label">
                <span>Endpoint <span class="meta-text">（MinIO / R2 等填此项）</span></span>
                <Input bind:value={s3Form.endpoint} placeholder="https://s3.amazonaws.com" />
              </label>
              <label class="label">
                <span>Access Key ID</span>
                <Input bind:value={s3Form.access_key_id} placeholder="可选，留空则读取环境变量" />
              </label>
              <label class="label">
                <span>Secret Access Key</span>
                <Input bind:value={s3Form.secret_access_key} type="password" placeholder="可选，留空则读取环境变量" />
              </label>
              <Button class="full-width-button" on:click={saveStorageConfig} disabled={storageBusy}>
                <span>{storageBusy ? '保存中' : '保存 S3 配置'}</span>
              </Button>
              {#if storageMessage}
                <p class="message-text">{storageMessage}</p>
              {/if}
            </div>
          {/if}
        </div>
      </div>

      <div class="card">
        <div class="card-header">
          <div>
            <h2>{t('dash.adminToken')}</h2>
            <p>{t('dash.adminTokenDesc')}</p>
          </div>
          <KeyRound size={18} class="muted-icon" />
        </div>
        <div class="card-body">
          <code class="code-token">{token || t('waitServer')}</code>
          {#if adminTokenExpires}
            <p class="meta-text mt-xs">{t('dash.expiresAt')} {formatDate(adminTokenExpires)}</p>
          {/if}
          <div class="token-actions">
            <Button on:click={issueAdminToken} disabled={tokenBusy}>
              <Sparkles size={15} />
              <span>{tokenBusy ? t('dash.regenerating') : t('dash.regenerate')}</span>
            </Button>
            <Button variant="secondary" size="icon" on:click={() => copyText(token, t('dash.adminToken'))}>
              <Clipboard size={15} />
            </Button>
          </div>
        </div>
      </div>

      <div class="card">
        <div class="card-header">
          <div>
            <h2>{t('dash.createCache')}</h2>
            <p>{t('dash.createCacheDesc')}</p>
          </div>
          <Plus size={18} class="muted-icon" />
        </div>
        <div class="card-body">
          <div class="stack stack-sm">
            <label class="label">
              <span>{t('dash.name')}</span>
              <Input bind:value={create.name} placeholder="main" autocomplete="off" />
            </label>
            <label class="label">
              <span>{t('dash.storeDir')}</span>
              <Input bind:value={create.storeDir} />
            </label>
            <div class="form-row">
              <label class="label">
                <span>{t('dash.priority')}</span>
                <Input bind:value={create.priority} type="number" />
              </label>
              <label class="checkbox-label inline-checkbox">
                <input bind:checked={create.isPublic} type="checkbox" />
                <span>{t('dash.isPublic')}</span>
              </label>
            </div>
            <Button class="full-width-button" on:click={createCache} disabled={createBusy}>
              <Plus size={15} />
              <span>{createBusy ? t('dash.creating') : t('dash.create')}</span>
            </Button>
            {#if createMessage}
              <p class="message-text">{createMessage}</p>
            {/if}
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

{#if copyMessage}
  <div class="toast" class:error={copyState === 'error'}>{copyMessage}</div>
{/if}
