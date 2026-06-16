<script>
  import { onMount } from 'svelte';
  import {
    Activity,
    AlertCircle,
    ArrowLeft,
    BarChart3,
    Box,
    CheckCircle2,
    Clipboard,
    ExternalLink,
    Info,
    RefreshCw,
    Save,
    Settings2,
    ShieldCheck,
    Trash2,
    TrendingUp
  } from '@lucide/svelte';
  import InfoLine from './InfoLine.svelte';
  import LineChart from '$lib/LineChart.svelte';
  import { t, formatDate, formatBytes, formatRetention } from '$lib/i18n/index.svelte';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Input } from '$lib/components/ui/input';
  import { Select } from '$lib/components/ui/select';
  import { Textarea } from '$lib/components/ui/textarea';

  let cacheName = '';
  let token = '';
  let cache = null;
  let usage = null;
  let stats = null;
  let loading = true;
  let busy = false;
  let error = '';
  let message = '';
  let copyMessage = '';
  let copyState = '';
  let configTab = 'settings';

  let edit = {
    isPublic: true,
    storeDir: '/nix/store',
    priority: 40,
    upstream: '',
    retentionMode: 'global',
    retentionSeconds: 0
  };

  onMount(() => {
    cacheName = new URLSearchParams(location.search).get('name') || '';
    token = localStorage.getItem('attic.console.token') ?? '';
    loadData();
  });

  async function loadData() {
    if (!cacheName) {
      error = t('cache.noCache');
      loading = false;
      return;
    }

    loading = true;
    error = '';

    try {
      const [summaryRes, usageRes, statsRes] = await Promise.all([
        fetch('/_api/web/summary'),
        fetch('/_api/web/usage'),
        fetch(`/_api/web/caches/${encodeURIComponent(cacheName)}/stats?period=30`)
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
      if (!cache) throw new Error(t('cache.cacheMissing', { name: cacheName }));

      usage = usagePayload.cache_usage.find((u) => u.cache.name === cacheName) ?? null;

      if (statsRes.ok) {
        stats = await statsRes.json();
      }

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
      message = t('dash.needTokenHome');
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
      message = t('cache.configSaved');
      await loadData();
    } catch (err) {
      message = err instanceof Error ? err.message : String(err);
    } finally {
      busy = false;
    }
  }

  async function deleteCache() {
    if (!token.trim()) {
      message = t('dash.needTokenHome');
      return;
    }
    if (!confirm(t('cache.deleteConfirm', { name: cacheName }))) return;
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

  $: period = stats?.period ?? 30;

  // Build chart series from stats
  $: chartSeries = stats ? [
    {
      key: 'uploads',
      label: t('cache.uploadsCount'),
      color: 'hsl(var(--primary))',
      points: stats.points.map((p) => ({ x: p.date, y: p.upload_count }))
    },
    {
      key: 'downloads',
      label: t('cache.accessesPerDay'),
      color: 'hsl(142 76% 36%)',
      points: stats.points.map((p) => ({ x: p.date, y: p.accessed_count }))
    }
  ] : [];

  // Distribution data for simplified bars
  $: systems = usage?.systems ?? [];
  $: compressions = usage?.compressions ?? [];
  $: maxSysCount = systems.length ? Math.max(...systems.map(s => s.count)) : 1;
  $: maxCompCount = compressions.length ? Math.max(...compressions.map(c => c.count)) : 1;
</script>

<svelte:head>
  <title>{cacheName ? `${cacheName} - Attic` : 'Attic Cache'}</title>
</svelte:head>

<div class="page-header">
  <div class="page-header-row">
    <div class="page-heading-row">
      <Button variant="ghost" size="icon" class="size-8" href="/">
        <ArrowLeft size={16} />
      </Button>
      <div class="page-heading">
        <h1 class="page-title">{cacheName || t('cache.title')}</h1>
        <p class="page-description">{cache?.store_dir ?? t('cache.nixStore')}</p>
      </div>
    </div>
    <Button variant="secondary" type="button" on:click={loadData} disabled={loading}>
      <span class:spin={loading}><RefreshCw size={15} /></span>
      <span>{t('refresh')}</span>
    </Button>
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
    <!-- Overview Stats -->
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-icon"><Box size={18} /></div>
        <div class="stat-content">
          <span class="stat-label">{t('cache.objectCount')}</span>
          <span class="stat-value">{cache.objects}</span>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon"><ShieldCheck size={18} /></div>
        <div class="stat-content">
          <span class="stat-label">{t('status')}</span>
          <span class="stat-value">{cache.is_public ? t('public') : t('private')} · P{cache.priority}</span>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon"><CheckCircle2 size={18} /></div>
        <div class="stat-content">
          <span class="stat-label">{t('retention')}</span>
          <span class="stat-value">{formatRetention(cache.retention_period)}</span>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon"><ExternalLink size={18} /></div>
        <div class="stat-content">
          <span class="stat-label">{t('cache.lastUpload')}</span>
          <span class="stat-value stat-value-sm">{usage ? formatDate(usage.last_upload_at) : '-'}</span>
        </div>
      </div>
    </div>

    <!-- Usage stats cards -->
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-icon"><Activity size={18} /></div>
        <div class="stat-content">
          <span class="stat-label">{t('cache.totalAccesses')}</span>
          <span class="stat-value">{stats?.total_accesses ?? '-'}</span>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon"><TrendingUp size={18} /></div>
        <div class="stat-content">
          <span class="stat-label">{t('cache.totalStorage')}</span>
          <span class="stat-value stat-value-md">{usage ? formatBytes(usage.nar_size) : '-'}</span>
        </div>
      </div>
    </div>

    <!-- Usage Trends Chart -->
    {#if stats && chartSeries.length}
      <div class="card">
        <div class="card-header">
          <div>
            <h2>{t('cache.usageTrends')}</h2>
            <p>{t('cache.usageTrendsDesc', { period })}</p>
          </div>
          <BarChart3 size={18} class="muted-icon" />
        </div>
        <div class="card-body">
          <LineChart series={chartSeries} height={220} />
        </div>
      </div>
    {/if}

    <!-- Distribution (simplified) -->
    {#if usage}
      <div class="card">
        <div class="card-header">
          <div>
            <h2>{t('cache.systemDist')}</h2>
            <p>{t('cache.compressionDist')}</p>
          </div>
        </div>
        <div class="card-body">
          <div class="split-grid">
            {#if systems.length}
              <div>
                <h3 class="section-title">{t('cache.systemDist')}</h3>
                <div class="breakdown-list">
                  {#each systems as sys}
                    <div class="breakdown-row">
                      <div class="breakdown-row-label">
                        <Badge variant="secondary">{sys.name}</Badge>
                      </div>
                      <div class="breakdown-bar-track">
                        <div class="breakdown-bar" style="width:{(sys.count / maxSysCount) * 100}%"></div>
                      </div>
                      <div class="breakdown-row-value">{sys.count} · {formatBytes(sys.nar_size)}</div>
                    </div>
                  {/each}
                </div>
              </div>
            {/if}

            {#if compressions.length}
              <div>
                <h3 class="section-title">{t('cache.compressionDist')}</h3>
                <div class="breakdown-list">
                  {#each compressions as comp}
                    <div class="breakdown-row">
                      <div class="breakdown-row-label">
                        <Badge variant="secondary">{comp.name}</Badge>
                      </div>
                      <div class="breakdown-bar-track">
                        <div class="breakdown-bar" style="width:{(comp.count / maxCompCount) * 100}%"></div>
                      </div>
                      <div class="breakdown-row-value">{comp.count} · {formatBytes(comp.nar_size)}</div>
                    </div>
                  {/each}
                </div>
              </div>
            {/if}

            {#if !systems.length && !compressions.length}
              <div class="empty grid-full">{t('noMatch')}</div>
            {/if}
          </div>
        </div>
      </div>
    {/if}

    <!-- Config & Connection -->
    <div class="card">
      <div class="card-header">
        <div>
          <h2>{configTab === 'settings' ? t('cache.config') : t('cache.connection')}</h2>
          <p>{configTab === 'settings' ? t('cache.configDesc') : t('cache.connectionDesc')}</p>
        </div>
        <div class="tabs">
          <button class="tab" class:active={configTab === 'settings'} on:click={() => configTab = 'settings'}>
            <Settings2 size={13} />
            <span>{t('cache.settings')}</span>
          </button>
          <button class="tab" class:active={configTab === 'connection'} on:click={() => configTab = 'connection'}>
            <Info size={13} />
            <span>{t('cache.connectionTab')}</span>
          </button>
        </div>
      </div>
      <div class="card-body">
        {#if configTab === 'settings'}
          <div class="form-grid">
            <label class="label">
              <span>{t('cache.storeDir')}</span>
              <Input bind:value={edit.storeDir} />
            </label>
            <label class="label">
              <span>{t('dash.priority')}</span>
              <Input type="number" bind:value={edit.priority} />
            </label>
            <label class="checkbox-label">
              <input type="checkbox" bind:checked={edit.isPublic} />
              <span>{t('cache.isPublic')}</span>
            </label>
            <label class="label">
              <span>{t('cache.retentionMode')}</span>
              <Select bind:value={edit.retentionMode}>
                <option value="global">{t('globalDefault')}</option>
                <option value="period">{t('cache.retentionSeconds')}</option>
              </Select>
            </label>
            {#if edit.retentionMode === 'period'}
              <label class="label">
                <span>{t('cache.retentionSeconds')}</span>
                <Input type="number" min="0" bind:value={edit.retentionSeconds} />
              </label>
            {/if}
            <label class="label full-width">
              <span>{t('cache.upstreamKeys')}</span>
              <Textarea bind:value={edit.upstream} rows="3" />
            </label>
          </div>
          <div class="form-actions mt-md">
            <Button type="button" on:click={saveCache} disabled={busy}>
              <Save size={15} />
              <span>{busy ? t('saving') : t('cache.saveConfig')}</span>
            </Button>
            <Button variant="destructive" type="button" on:click={deleteCache} disabled={busy}>
              <Trash2 size={15} />
              <span>{t('cache.deleteCache')}</span>
            </Button>
          </div>
          {#if message}
            <p class="message-text mt-sm">{message}</p>
          {/if}
        {:else}
          <div class="connection-grid">
            <InfoLine label={t('cache.apiEndpoint')} value={cache.api_endpoint} oncopy={copyText} />
            <InfoLine label={t('substituter')} value={cache.substituter_endpoint} oncopy={copyText} />
            <InfoLine label={t('publicKey')} value={cache.public_key} oncopy={copyText} />
            <InfoLine label={t('cache.upstreamKeysLabel')} value={(cache.upstream_cache_key_names ?? []).join(', ') || t('cache.none')} oncopy={copyText} />
          </div>
        {/if}
      </div>
    </div>
  {:else if loading}
    <div class="empty">{t('cache.cacheLoading')}</div>
  {/if}
</div>

{#if copyMessage}
  <div class="toast" class:error={copyState === 'error'}>{copyMessage}</div>
{/if}
