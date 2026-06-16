<script>
  import { onMount } from 'svelte';
  import {
    Activity, AlertCircle, ArrowLeft, BarChart3, Box,
    CheckCircle2, Clipboard, ExternalLink, Info, RefreshCw,
    Save, Settings2, ShieldCheck, Trash2, TrendingUp
  } from '@lucide/svelte';
  import InfoLine from './InfoLine.svelte';
  import LineChart from '$lib/LineChart.svelte';
  import { t, formatDate, formatBytes, formatRetention } from '$lib/i18n/index.svelte';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Input } from '$lib/components/ui/input';
  import { Select } from '$lib/components/ui/select';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '$lib/components/ui/card';

  let cacheName = ''; let token = ''; let cache = null; let usage = null; let stats = null;
  let loading = true; let busy = false; let error = ''; let message = '';
  let copyMessage = ''; let copyState = ''; let configTab = 'settings';

  let edit = { isPublic: true, storeDir: '/nix/store', priority: 40, upstream: '', retentionMode: 'global', retentionSeconds: 0 };

  onMount(() => {
    cacheName = new URLSearchParams(location.search).get('name') || '';
    token = localStorage.getItem('attic.console.token') ?? '';
    loadData();
  });

  async function loadData() {
    if (!cacheName) { error = t('cache.noCache'); loading = false; return; }
    loading = true; error = '';
    try {
      const [sr, ur, st] = await Promise.all([
        fetch('/_api/web/summary'), fetch('/_api/web/usage'),
        fetch(`/_api/web/caches/${encodeURIComponent(cacheName)}/stats?period=30`)
      ]);
      if (!sr.ok) { const t = await sr.text(); throw new Error(t || `HTTP ${sr.status}`); }
      if (!ur.ok) { const t = await ur.text(); throw new Error(t || `HTTP ${ur.status}`); }
      const s = await sr.json(); const u = await ur.json();
      cache = s.caches.find((c) => c.name === cacheName) ?? null;
      if (!cache) throw new Error(t('cache.cacheMissing', { name: cacheName }));
      usage = u.cache_usage.find((x) => x.cache.name === cacheName) ?? null;
      if (st.ok) stats = await st.json();
      syncEdit(cache);
    } catch (err) { error = err instanceof Error ? err.message : String(err); }
    finally { loading = false; }
  }

  function syncEdit(c) { edit = { isPublic: c.is_public, storeDir: c.store_dir, priority: c.priority, upstream: (c.upstream_cache_key_names ?? []).join('\n'), retentionMode: c.retention_period === null || c.retention_period === undefined ? 'global' : 'period', retentionSeconds: c.retention_period ?? 0 }; }

  async function saveCache() {
    if (!token.trim()) { message = t('dash.needTokenHome'); return; }
    busy = true; message = '';
    try {
      const body = { is_public: edit.isPublic, store_dir: edit.storeDir || '/nix/store', priority: Number(edit.priority), upstream_cache_key_names: edit.upstream.split(/[\r\n,]+/).map(x => x.trim()).filter(Boolean), retention_period: edit.retentionMode === 'global' ? 'Global' : { Period: Number(edit.retentionSeconds) } };
      const r = await fetch(`/_api/web/caches/${encodeURIComponent(cacheName)}/config`, { method: 'PATCH', headers: { 'Authorization': `Bearer ${token.trim()}`, 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
      if (!r.ok) { const t = await r.text(); throw new Error(t || `HTTP ${r.status}`); }
      message = t('cache.configSaved'); await loadData();
    } catch (err) { message = err instanceof Error ? err.message : String(err); }
    finally { busy = false; }
  }

  async function deleteCache() {
    if (!token.trim()) { message = t('dash.needTokenHome'); return; }
    if (!confirm(t('cache.deleteConfirm', { name: cacheName }))) return;
    busy = true; message = '';
    try {
      const r = await fetch(`/_api/web/caches/${encodeURIComponent(cacheName)}/config`, { method: 'DELETE', headers: { 'Authorization': `Bearer ${token.trim()}` } });
      if (!r.ok) { const t = await r.text(); throw new Error(t || `HTTP ${r.status}`); }
      location.href = '/';
    } catch (err) { message = err instanceof Error ? err.message : String(err); busy = false; }
  }

  async function copyText(value, label) {
    if (!value) return;
    try { await navigator.clipboard.writeText(value); copyMessage = t('cache.copyToast', { label }); copyState = 'success'; }
    catch (err) { copyMessage = err instanceof Error ? err.message : String(err); copyState = 'error'; }
    finally { setTimeout(() => { copyMessage = ''; copyState = ''; }, 1800); }
  }

  $: period = stats?.period ?? 30;
  $: chartSeries = stats ? [
    { key: 'uploads', label: t('cache.uploadsCount'), color: 'hsl(var(--primary))', points: stats.points.map(p => ({ x: p.date, y: p.upload_count })) },
    { key: 'downloads', label: t('cache.accessesPerDay'), color: 'hsl(142 70% 40%)', points: stats.points.map(p => ({ x: p.date, y: p.accessed_count })) }
  ] : [];
  $: systems = (usage?.systems ?? []).filter(s => s.name !== 'unknown');
  $: compressions = (usage?.compressions ?? []).filter(c => c.name !== 'unknown');
  $: maxSysCount = systems.length ? Math.max(...systems.map(s => s.count)) : 1;
  $: maxCompCount = compressions.length ? Math.max(...compressions.map(c => c.count)) : 1;
</script>

<svelte:head><title>{cacheName ? `${cacheName} - Attic` : 'Attic Cache'}</title></svelte:head>

<div class="mx-auto max-w-6xl px-5 py-6 space-y-6">
  <div class="flex items-center justify-between gap-4">
    <div class="flex items-center gap-3 min-w-0">
      <Button variant="ghost" size="icon" class="size-8" href="/"><ArrowLeft size={16} /></Button>
      <div class="min-w-0"><h1 class="text-2xl font-bold text-foreground">{cacheName || t('cache.title')}</h1><p class="text-sm text-muted-foreground">{cache?.store_dir ?? t('cache.nixStore')}</p></div>
    </div>
    <Button variant="secondary" onclick={loadData} disabled={loading}><span class:spin={loading}><RefreshCw size={15} /></span><span>{t('refresh')}</span></Button>
  </div>

  {#if error}<div class="notice"><AlertCircle size={16} /><span>{error}</span></div>{/if}

  {#if cache}
    <div class="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-6">
      <div class="flex items-center gap-3 rounded-lg border border-border bg-card px-4 py-3"><div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-secondary"><Box size={18} /></div><div class="min-w-0"><p class="text-xs text-muted-foreground">{t('cache.objectCount')}</p><p class="text-lg font-bold">{cache.objects}</p></div></div>
      <div class="flex items-center gap-3 rounded-lg border border-border bg-card px-4 py-3"><div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-secondary"><ShieldCheck size={18} /></div><div class="min-w-0"><p class="text-xs text-muted-foreground">{t('status')}</p><p class="text-lg font-bold">{cache.is_public ? t('public') : t('private')} · P{cache.priority}</p></div></div>
      <div class="flex items-center gap-3 rounded-lg border border-border bg-card px-4 py-3"><div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-secondary"><CheckCircle2 size={18} /></div><div class="min-w-0"><p class="text-xs text-muted-foreground">{t('retention')}</p><p class="text-lg font-bold">{formatRetention(cache.retention_period)}</p></div></div>
      <div class="flex items-center gap-3 rounded-lg border border-border bg-card px-4 py-3"><div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-secondary"><Activity size={18} /></div><div class="min-w-0"><p class="text-xs text-muted-foreground">{t('cache.totalAccesses')}</p><p class="text-lg font-bold">{stats?.total_accesses ?? '-'}</p></div></div>
      <div class="flex items-center gap-3 rounded-lg border border-border bg-card px-4 py-3"><div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-secondary"><TrendingUp size={18} /></div><div class="min-w-0"><p class="text-xs text-muted-foreground">{t('cache.totalStorage')}</p><p class="text-lg font-bold">{usage ? formatBytes(usage.nar_size) : '-'}</p></div></div>
      <div class="flex items-center gap-3 rounded-lg border border-border bg-card px-4 py-3"><div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-secondary"><ExternalLink size={18} /></div><div class="min-w-0"><p class="text-xs text-muted-foreground">{t('cache.lastUpload')}</p><p class="text-lg font-bold">{usage ? formatDate(usage.last_upload_at) : '-'}</p></div></div>
    </div>

    {#if stats && chartSeries.length}
      <Card><CardHeader><div class="flex items-center justify-between"><div><CardTitle class="text-sm">{t('cache.usageTrends')}</CardTitle><CardDescription class="text-xs">{t('cache.usageTrendsDesc', { period })}</CardDescription></div><BarChart3 size={16} class="text-muted-foreground" /></div></CardHeader><CardContent><LineChart series={chartSeries} height={220} /></CardContent></Card>
    {/if}

    {#if usage && (systems.length || compressions.length)}
      <Card><CardHeader><div><CardTitle class="text-sm">{t('cache.systemDist')}</CardTitle><CardDescription class="text-xs">{t('cache.compressionDist')}</CardDescription></div></CardHeader><CardContent>
        <div class="grid gap-6 md:grid-cols-2">
          {#if systems.length}
            <div><p class="mb-3 text-xs font-semibold text-muted-foreground">{t('cache.systemDist')}</p><div class="space-y-2.5">
              {#each systems as sys}
                <div class="flex items-center gap-3"><div class="shrink-0"><Badge variant="secondary">{sys.name}</Badge></div><div class="flex-1"><div class="h-2 rounded-full bg-muted"><div class="h-full rounded-full bg-primary transition-all" style="width:{(sys.count / maxSysCount) * 100}%"></div></div></div><span class="shrink-0 text-xs text-muted-foreground">{sys.count} · {formatBytes(sys.nar_size)}</span></div>
              {/each}
            </div></div>
          {/if}
          {#if compressions.length}
            <div><p class="mb-3 text-xs font-semibold text-muted-foreground">{t('cache.compressionDist')}</p><div class="space-y-2.5">
              {#each compressions as comp}
                <div class="flex items-center gap-3"><div class="shrink-0"><Badge variant="secondary">{comp.name}</Badge></div><div class="flex-1"><div class="h-2 rounded-full bg-muted"><div class="h-full rounded-full bg-primary transition-all" style="width:{(comp.count / maxCompCount) * 100}%"></div></div></div><span class="shrink-0 text-xs text-muted-foreground">{comp.count} · {formatBytes(comp.nar_size)}</span></div>
              {/each}
            </div></div>
          {/if}
        </div>
      </CardContent></Card>
    {/if}

    <Card>
      <CardHeader class="flex-row items-center justify-between space-y-0 pb-2">
        <div><CardTitle class="text-sm">{configTab === 'settings' ? t('cache.config') : t('cache.connection')}</CardTitle><CardDescription class="text-xs">{configTab === 'settings' ? t('cache.configDesc') : t('cache.connectionDesc')}</CardDescription></div>
        <div class="inline-flex rounded-md border border-border bg-muted p-0.5">
          <button class="inline-flex items-center gap-1 rounded-sm px-2.5 py-1 text-xs font-medium transition-colors {configTab === 'settings' ? 'bg-background text-foreground' : 'text-muted-foreground hover:text-foreground'}" onclick={() => configTab = 'settings'}><Settings2 size={13} /><span>{t('cache.settings')}</span></button>
          <button class="inline-flex items-center gap-1 rounded-sm px-2.5 py-1 text-xs font-medium transition-colors {configTab === 'connection' ? 'bg-background text-foreground' : 'text-muted-foreground hover:text-foreground'}" onclick={() => configTab = 'connection'}><Info size={13} /><span>{t('cache.connectionTab')}</span></button>
        </div>
      </CardHeader>
      <CardContent>
        {#if configTab === 'settings'}
          <div class="grid gap-4 sm:grid-cols-2">
            <label class="flex flex-col gap-1.5"><span class="text-xs font-medium">{t('cache.storeDir')}</span><Input bind:value={edit.storeDir} /></label>
            <label class="flex flex-col gap-1.5"><span class="text-xs font-medium">{t('dash.priority')}</span><Input type="number" bind:value={edit.priority} /></label>
            <label class="flex items-center gap-2 text-xs font-medium cursor-pointer"><input type="checkbox" bind:checked={edit.isPublic} class="size-4 rounded accent-primary" /><span>{t('cache.isPublic')}</span></label>
            <label class="flex flex-col gap-1.5"><span class="text-xs font-medium">{t('cache.retentionMode')}</span><Select bind:value={edit.retentionMode}><option value="global">{t('globalDefault')}</option><option value="period">{t('cache.retentionSeconds')}</option></Select></label>
            {#if edit.retentionMode === 'period'}<label class="flex flex-col gap-1.5 sm:col-span-2"><span class="text-xs font-medium">{t('cache.retentionSeconds')}</span><Input type="number" min="0" bind:value={edit.retentionSeconds} /></label>{/if}
            <label class="flex flex-col gap-1.5 sm:col-span-2"><span class="text-xs font-medium">{t('cache.upstreamKeys')}</span><Textarea bind:value={edit.upstream} rows="3" /></label>
          </div>
          <div class="mt-4 flex flex-wrap gap-3">
            <Button onclick={saveCache} disabled={busy}><Save size={14} /><span>{busy ? t('saving') : t('cache.saveConfig')}</span></Button>
            <Button variant="destructive" onclick={deleteCache} disabled={busy}><Trash2 size={14} /><span>{t('cache.deleteCache')}</span></Button>
          </div>
          {#if message}<p class="mt-3 text-xs text-muted-foreground">{message}</p>{/if}
        {:else}
          <div class="divide-y divide-border">
            <InfoLine label={t('cache.apiEndpoint')} value={cache.api_endpoint} oncopy={copyText} />
            <InfoLine label={t('substituter')} value={cache.substituter_endpoint} oncopy={copyText} />
            <InfoLine label={t('publicKey')} value={cache.public_key} oncopy={copyText} />
            <InfoLine label={t('cache.upstreamKeysLabel')} value={(cache.upstream_cache_key_names ?? []).join(', ') || t('cache.none')} oncopy={copyText} />
          </div>
        {/if}
      </CardContent>
    </Card>
  {:else if loading}
    <div class="flex flex-col items-center justify-center gap-2 py-20 text-center text-muted-foreground">{t('cache.cacheLoading')}</div>
  {/if}
</div>

{#if copyMessage}<div class="toast" class:error={copyState === 'error'}>{copyMessage}</div>{/if}