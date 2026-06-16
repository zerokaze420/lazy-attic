<script>
  import { onMount } from 'svelte';
  import {
    AlertCircle, Box, Clipboard, Database, ExternalLink,
    KeyRound, Layers3, Plus, RefreshCw, Server, Sparkles, Terminal
  } from '@lucide/svelte';
  import { t, formatDate, formatBytes, formatRetention } from '$lib/i18n/index.svelte';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Input } from '$lib/components/ui/input';
  import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '$lib/components/ui/card';

  const defaultCreate = { name: '', isPublic: true, priority: 40, storeDir: '/nix/store' };

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
  let copyState = '';
  let commandTab = 'client';
  let storageTab = 'current';
  let s3Form = { region: 'us-east-1', bucket: '', endpoint: '', access_key_id: '', secret_access_key: '' };
  let storageBusy = false;
  let storageMessage = '';

  onMount(() => {
    origin = location.origin;
    token = localStorage.getItem('attic.console.token') ?? '';
    refresh();
  });

  async function refresh() {
    loading = true; error = '';
    try {
      const res = await fetch('/_api/web/summary');
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      summary = await res.json();
      if (summary.admin_token?.token) {
        token = summary.admin_token.token;
        adminTokenExpires = summary.admin_token.expires_at;
        localStorage.setItem('attic.console.token', token);
      }
      const ur = await fetch('/_api/web/usage');
      if (ur.ok) usage = await ur.json();
    } catch (err) { error = err instanceof Error ? err.message : String(err); }
    finally { loading = false; }
  }

  async function issueAdminToken() {
    tokenBusy = true; createMessage = '';
    try {
      const r = await fetch('/_api/web/admin-token', { method: 'POST' });
      if (!r.ok) { const t = await r.text(); throw new Error(t || `HTTP ${r.status}`); }
      const p = await r.json();
      token = p.token; adminTokenExpires = p.expires_at;
      localStorage.setItem('attic.console.token', token);
      createMessage = t('dash.tokenGenerated', { date: formatDate(p.expires_at) });
    } catch (err) { createMessage = err instanceof Error ? err.message : String(err); }
    finally { tokenBusy = false; }
  }

  async function createCache() {
    createBusy = true; createMessage = '';
    try {
      const name = create.name.trim();
      if (!name) throw new Error(t('dash.needName'));
      if (!token.trim()) throw new Error(t('dash.needToken'));
      const r = await fetch(`/_api/v1/cache-config/${encodeURIComponent(name)}`, {
        method: 'POST',
        headers: { 'Authorization': `Bearer ${token.trim()}`, 'Content-Type': 'application/json' },
        body: JSON.stringify({ keypair: 'Generate', is_public: create.isPublic, store_dir: create.storeDir || '/nix/store', priority: Number(create.priority), upstream_cache_key_names: [] })
      });
      if (!r.ok) { const t = await r.text(); throw new Error(t || `HTTP ${r.status}`); }
      create = { ...defaultCreate };
      createMessage = t('dash.cacheCreated', { name });
      await refresh();
    } catch (err) { createMessage = err instanceof Error ? err.message : String(err); }
    finally { createBusy = false; }
  }

  async function saveStorageConfig() {
    storageBusy = true; storageMessage = '';
    try {
      const r = await fetch('/_api/web/storage', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(s3Form) });
      const result = await r.json();
      if (!r.ok) throw new Error(result.message || `HTTP ${r.status}`);
      storageMessage = result.message; await refresh();
    } catch (err) { storageMessage = err instanceof Error ? err.message : String(err); }
    finally { storageBusy = false; }
  }

  async function copyText(value, label) {
    if (!value || value.startsWith('<')) return;
    try { await navigator.clipboard.writeText(value); copyMessage = t('cache.copyToast', { label }); copyState = 'success'; }
    catch (err) { copyMessage = err instanceof Error ? err.message : String(err); copyState = 'error'; }
    finally { setTimeout(() => { copyMessage = ''; copyState = ''; }, 1800); }
  }

  $: caches = summary?.caches ?? [];
  $: pm = caches.find((c) => c.is_public) || caches[0];
  $: ec = caches[0]?.name || '<cache-name>';
  $: pc = pm?.name || ec;
  $: pk = pm?.public_key || '<public-key>';
  $: latestUsage = usage?.cache_usage?.[0]?.nar_size ?? 0;
  $: ae = pm?.api_endpoint || `${origin}/`;
  $: se = pm?.substituter_endpoint || `${origin}/${pc}`;
  $: loginCommand = `attic login local ${ae} <token>`;
  $: useCommand = `attic use local:${ec}`;
  $: pushCommand = `attic push local:${ec} /nix/store/<path>`;
  $: subCmd = `substituters = ${se}`;
  $: keysCmd = `trusted-public-keys = ${pk}`;
  $: activeCommands = commandTab === 'client'
    ? [{ value: loginCommand, label: t('dash.loginCommand') }, { value: useCommand, label: t('dash.useCommand') }, { value: pushCommand, label: t('dash.pushCommand') }]
    : [{ value: subCmd, label: t('substituter') }, { value: keysCmd, label: t('publicKey') }];
</script>

<svelte:head><title>Attic Console</title></svelte:head>

<div class="mx-auto max-w-6xl px-5 py-6 space-y-6">
  <div class="flex items-center justify-between gap-4">
    <div class="min-w-0">
      <h1 class="text-2xl font-bold text-foreground">{t('dash.title')}</h1>
      <p class="mt-1 text-sm text-muted-foreground">{t('dash.description')}</p>
    </div>
    <Button variant="secondary" onclick={refresh} disabled={loading}>
      <span class:spin={loading}><RefreshCw size={15} /></span>
      <span>{t('refresh')}</span>
    </Button>
  </div>

  {#if error}
    <div class="notice"><AlertCircle size={16} /><span>{error}</span></div>
  {/if}

  <div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
    <div class="flex items-center gap-3 rounded-lg border border-border bg-card px-4 py-3">
      <div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-secondary text-foreground"><Database size={18} /></div>
      <div class="min-w-0"><p class="text-xs text-muted-foreground">{t('dash.caches')}</p><p class="text-lg font-bold">{summary?.counts?.caches ?? '-'}</p></div>
    </div>
    <div class="flex items-center gap-3 rounded-lg border border-border bg-card px-4 py-3">
      <div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-secondary text-foreground"><Box size={18} /></div>
      <div class="min-w-0"><p class="text-xs text-muted-foreground">{t('objects')}</p><p class="text-lg font-bold">{summary?.counts?.objects ?? '-'}</p></div>
    </div>
    <div class="flex items-center gap-3 rounded-lg border border-border bg-card px-4 py-3">
      <div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-secondary text-foreground"><Layers3 size={18} /></div>
      <div class="min-w-0"><p class="text-xs text-muted-foreground">{t('dash.nars')}</p><p class="text-lg font-bold">{summary?.counts?.nars ?? '-'}</p></div>
    </div>
    <div class="flex items-center gap-3 rounded-lg border border-border bg-card px-4 py-3">
      <div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-secondary text-foreground"><Server size={18} /></div>
      <div class="min-w-0"><p class="text-xs text-muted-foreground">{t('storage')}</p><p class="text-lg font-bold">{usage ? formatBytes(latestUsage) : '-'}</p></div>
    </div>
  </div>

  <div class="grid gap-6 lg:grid-cols-[1fr_380px]">
    <div class="space-y-6">
      <Card>
        <CardContent class="p-5">
          {#if caches.length}
            <div class="grid gap-3 sm:grid-cols-2">
              {#each caches as cache}
                <div class="flex flex-col gap-3 rounded-lg border border-border bg-background p-4 transition-colors hover:border-ring/30">
                  <div class="flex items-start justify-between gap-2">
                    <div class="min-w-0"><div class="font-semibold text-foreground">{cache.name}</div><div class="text-xs text-muted-foreground">{cache.store_dir}</div></div>
                    <Badge variant="secondary" class={cache.is_public ? 'status-public' : 'status-private'}>{cache.is_public ? t('public') : t('private')} · P{cache.priority}</Badge>
                  </div>
                  <div class="grid grid-cols-3 gap-2">
                    <div class="rounded border border-border bg-muted/30 px-2 py-1.5"><p class="text-[11px] text-muted-foreground">{t('objects')}</p><p class="text-sm font-semibold">{cache.objects}</p></div>
                    <div class="rounded border border-border bg-muted/30 px-2 py-1.5"><p class="text-[11px] text-muted-foreground">{t('retention')}</p><p class="text-sm font-semibold">{formatRetention(cache.retention_period)}</p></div>
                    <div class="rounded border border-border bg-muted/30 px-2 py-1.5"><p class="text-[11px] text-muted-foreground">{t('created')}</p><p class="text-sm font-semibold">{formatDate(cache.created_at)}</p></div>
                  </div>
                  <div class="flex flex-col gap-1">
                    <div class="flex items-center justify-between"><span class="text-[11px] font-semibold text-muted-foreground">{t('substituter')}</span><Button variant="ghost" size="icon" class="size-7" onclick={() => copyText(cache.substituter_endpoint, 'Substituter')}><Clipboard size={13} /></Button></div>
                    <code class="rounded bg-muted px-2 py-1 text-xs text-foreground break-all" style="font-family:var(--font-mono)">{cache.substituter_endpoint}</code>
                  </div>
                  <div class="flex flex-wrap gap-1.5">
                    <Button size="sm" href={`/cache?name=${encodeURIComponent(cache.name)}`}><ExternalLink size={13} /><span>{t('details')}</span></Button>
                    <Button variant="secondary" size="sm" onclick={() => copyText(cache.substituter_endpoint, 'Substituter')}><Clipboard size={13} /><span>{t('dash.copyUrl')}</span></Button>
                    <Button variant="secondary" size="sm" onclick={() => copyText(cache.public_key, 'Public key')}><Clipboard size={13} /><span>{t('dash.copyKey')}</span></Button>
                  </div>
                </div>
              {/each}
            </div>
          {:else}
            <div class="flex flex-col items-center justify-center gap-2 py-10 text-center text-muted-foreground">{loading ? t('dash.loadingCaches') : t('dash.noMatchingCaches')}</div>
          {/if}
        </CardContent>
      </Card>

      <Card>
        <CardHeader class="flex-row items-center justify-between space-y-0 pb-2">
          <div><CardTitle class="text-sm">{t('dash.commands')}</CardTitle><CardDescription class="text-xs">{commandTab === 'client' ? t('dash.commandsDesc') : t('dash.commandsDescNix')}</CardDescription></div>
          <div class="inline-flex rounded-md border border-border bg-muted p-0.5">
            <button class="inline-flex items-center gap-1 rounded-sm px-2.5 py-1 text-xs font-medium transition-colors {commandTab === 'client' ? 'bg-background text-foreground' : 'text-muted-foreground hover:text-foreground'}" onclick={() => commandTab = 'client'}><Terminal size={13} /><span>{t('dash.client')}</span></button>
            <button class="inline-flex items-center gap-1 rounded-sm px-2.5 py-1 text-xs font-medium transition-colors {commandTab === 'nix' ? 'bg-background text-foreground' : 'text-muted-foreground hover:text-foreground'}" onclick={() => commandTab = 'nix'}><Server size={13} /><span>{t('dash.nix')}</span></button>
          </div>
        </CardHeader>
        <CardContent class="p-0">
          {#each activeCommands as cmd}
            <div class="flex items-center justify-between gap-3 border-t border-border px-4 py-2.5">
              <div class="min-w-0 flex-1"><p class="text-xs font-semibold text-muted-foreground">{cmd.label}</p><code class="text-sm text-foreground break-all" style="font-family:var(--font-mono)">{cmd.value}</code></div>
              <Button variant="ghost" size="icon" class="size-8 shrink-0" title={`${t('copy')} ${cmd.label}`} onclick={() => copyText(cmd.value, cmd.label)}><Clipboard size={13} /></Button>
            </div>
          {/each}
        </CardContent>
      </Card>
    </div>

    <div class="space-y-6">
      <Card>
        <CardHeader>
          <div class="flex items-center justify-between gap-4">
            <div><CardTitle class="text-sm">{t('storage')}</CardTitle><CardDescription class="text-xs">{t('storageDesc') || '后端类型：' + (summary?.storage?.kind === 's3' ? 'S3' : 'Local')}</CardDescription></div>
            <Database size={16} class="text-muted-foreground" />
          </div>
        </CardHeader>
        <CardContent>
          <div class="inline-flex rounded-md border border-border bg-muted p-0.5">
            <button class="rounded-sm px-2.5 py-1 text-xs font-medium transition-colors {storageTab === 'current' ? 'bg-background text-foreground' : 'text-muted-foreground hover:text-foreground'}" onclick={() => storageTab = 'current'}>{t('dash.current') || '当前'}</button>
            <button class="rounded-sm px-2.5 py-1 text-xs font-medium transition-colors {storageTab === 's3' ? 'bg-background text-foreground' : 'text-muted-foreground hover:text-foreground'}" onclick={() => storageTab = 's3'}>S3</button>
          </div>
          {#if storageTab === 'current'}
            <div class="mt-3 space-y-2">
              <div class="rounded-md border border-border bg-muted/30 p-2.5"><p class="text-xs text-muted-foreground">{t('dash.storageKind') || '后端类型'}</p><p class="text-sm font-semibold break-all">{summary?.storage?.kind === 's3' ? 'S3' : 'Local'}</p></div>
              <div class="rounded-md border border-border bg-muted/30 p-2.5"><p class="text-xs text-muted-foreground">{summary?.storage?.kind === 's3' ? 'Bucket' : '路径'}</p><p class="text-sm font-semibold break-all">{summary?.storage?.location || '-'}</p></div>
            </div>
          {:else}
            <div class="mt-3 space-y-2.5">
              <label class="flex flex-col gap-1.5"><span class="text-xs font-medium text-foreground">Region</span><Input bind:value={s3Form.region} placeholder="us-east-1" /></label>
              <label class="flex flex-col gap-1.5"><span class="text-xs font-medium text-foreground">Bucket</span><Input bind:value={s3Form.bucket} placeholder="attic-cache" /></label>
              <label class="flex flex-col gap-1.5"><span class="text-xs font-medium text-foreground">Endpoint <span class="text-muted-foreground">（MinIO / R2 等填此项）</span></span><Input bind:value={s3Form.endpoint} placeholder="https://s3.amazonaws.com" /></label>
              <label class="flex flex-col gap-1.5"><span class="text-xs font-medium text-foreground">Access Key ID</span><Input bind:value={s3Form.access_key_id} placeholder="可选，留空则读取环境变量" /></label>
              <label class="flex flex-col gap-1.5"><span class="text-xs font-medium text-foreground">Secret Access Key</span><Input bind:value={s3Form.secret_access_key} type="password" placeholder="可选，留空则读取环境变量" /></label>
              <Button class="w-full" onclick={saveStorageConfig} disabled={storageBusy}><span>{storageBusy ? '保存中' : '保存 S3 配置'}</span></Button>
              {#if storageMessage}<p class="text-xs text-muted-foreground">{storageMessage}</p>{/if}
            </div>
          {/if}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <div class="flex items-center justify-between gap-4">
            <div><CardTitle class="text-sm">{t('dash.adminToken')}</CardTitle><CardDescription class="text-xs">{t('dash.adminTokenDesc')}</CardDescription></div>
            <KeyRound size={16} class="text-muted-foreground" />
          </div>
        </CardHeader>
        <CardContent>
          <code class="block rounded-md bg-muted px-3 py-2 text-xs break-all" style="font-family:var(--font-mono)">{token || t('waitServer')}</code>
          {#if adminTokenExpires}<p class="mt-2 text-xs text-muted-foreground">{t('dash.expiresAt')} {formatDate(adminTokenExpires)}</p>{/if}
          <div class="mt-3 grid grid-cols-[1fr_auto] gap-2">
            <Button onclick={issueAdminToken} disabled={tokenBusy}><Sparkles size={14} /><span>{tokenBusy ? t('dash.regenerating') : t('dash.regenerate')}</span></Button>
            <Button variant="secondary" size="icon" onclick={() => copyText(token, t('dash.adminToken'))}><Clipboard size={14} /></Button>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <div class="flex items-center justify-between gap-4">
            <div><CardTitle class="text-sm">{t('dash.createCache')}</CardTitle><CardDescription class="text-xs">{t('dash.createCacheDesc')}</CardDescription></div>
            <Plus size={16} class="text-muted-foreground" />
          </div>
        </CardHeader>
        <CardContent>
          <div class="space-y-3">
            <label class="flex flex-col gap-1.5"><span class="text-xs font-medium text-foreground">{t('dash.name')}</span><Input bind:value={create.name} placeholder="main" autocomplete="off" /></label>
            <label class="flex flex-col gap-1.5"><span class="text-xs font-medium text-foreground">{t('dash.storeDir')}</span><Input bind:value={create.storeDir} /></label>
            <div class="grid grid-cols-2 gap-3">
              <label class="flex flex-col gap-1.5"><span class="text-xs font-medium text-foreground">{t('dash.priority')}</span><Input bind:value={create.priority} type="number" /></label>
              <label class="flex items-center gap-2 self-end pb-1 text-xs font-medium text-foreground cursor-pointer"><input bind:checked={create.isPublic} type="checkbox" class="size-4 rounded accent-primary" /><span>{t('dash.isPublic')}</span></label>
            </div>
            <Button class="w-full" onclick={createCache} disabled={createBusy}><Plus size={14} /><span>{createBusy ? t('dash.creating') : t('dash.create')}</span></Button>
            {#if createMessage}<p class="text-xs text-muted-foreground">{createMessage}</p>{/if}
          </div>
        </CardContent>
      </Card>
    </div>
  </div>
</div>

{#if copyMessage}<div class="toast" class:error={copyState === 'error'}>{copyMessage}</div>{/if}