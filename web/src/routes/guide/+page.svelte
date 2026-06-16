<script>
  import { onMount } from 'svelte';
  import { AlertCircle, ArrowLeft, CheckCircle2, Clipboard, KeyRound, Server, Terminal, Wrench } from '@lucide/svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent } from '$lib/components/ui/card';

  let summary = null; let error = ''; let origin = '';
  let copyMessage = ''; let copyState = '';

  onMount(() => { origin = location.origin; loadSummary(); });

  async function loadSummary() {
    try {
      const r = await fetch('/_api/web/summary');
      if (!r.ok) throw new Error(`HTTP ${r.status}`);
      summary = await r.json();
    } catch (err) { error = err instanceof Error ? err.message : String(err); }
  }

  async function copyText(value, label) {
    if (!value || value.startsWith('<')) return;
    try { await navigator.clipboard.writeText(value); copyMessage = t('cache.copyToast', { label }); copyState = 'success'; }
    catch (err) { copyMessage = err instanceof Error ? err.message : String(err); copyState = 'error'; }
    finally { setTimeout(() => { copyMessage = ''; copyState = ''; }, 1800); }
  }

  $: caches = summary?.caches ?? [];
  $: cache = caches.find(x => x.name === 'main') || caches.find(x => x.is_public) || caches[0];
  $: cacheName = cache?.name || 'main';
  $: publicKey = cache?.public_key || '<public-key>';
  $: adminToken = summary?.admin_token?.token || '<admin-token>';
  $: endpoint = origin ? `${origin}/${cacheName}` : `https://<your-domain>/${cacheName}`;
  $: nixConf = `substituters = ${endpoint}\ntrusted-public-keys = ${publicKey}`;
  $: nixosConf = `nix.settings.substituters = [ "${endpoint}" ];\nnix.settings.trusted-public-keys = [ "${publicKey}" ];`;
  $: loginCmd = `attic login lazy-attic ${origin || 'https://<your-domain>'}/ ${adminToken}`;
  $: useCmd = `attic use lazy-attic:${cacheName}`;
  $: pushCmd = `attic push lazy-attic:${cacheName} /nix/store/<path>`;
  $: clientBlocks = [{ label: t('guide.loginCommand'), value: loginCmd }, { label: t('guide.selectCache'), value: useCmd }];
  $: pushBlocks = [{ label: t('guide.pushExample'), value: pushCmd }, { label: t('guide.pushFlake'), value: `nix build .#default --print-out-paths\nattic push lazy-attic:${cacheName} ./result` }];
  $: nixBlocks = [{ label: t('guide.nixConf'), value: nixConf }, { label: t('guide.nixosConfig'), value: nixosConf }];
  $: valueBlocks = [{ label: t('guide.adminTokenValue'), value: adminToken }, { label: t('guide.publicKeyValue'), value: publicKey }, { label: t('guide.cacheEndpointValue'), value: endpoint }];
</script>

<svelte:head><title>{t('guide.title')}</title></svelte:head>

<div class="mx-auto max-w-4xl px-5 py-6 space-y-6">
  <div class="space-y-4">
    <div><Button variant="ghost" size="icon" class="size-8" href="/"><ArrowLeft size={16} /></Button></div>
    <div class="flex items-center gap-3">
      <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg border border-border bg-card"><Server size={20} /></div>
      <div><h1 class="text-2xl font-bold">{t('guide.headTitle')}</h1><p class="text-sm text-muted-foreground">{t('guide.headDesc')}</p></div>
    </div>
  </div>

  {#if error}
    <div class="notice"><AlertCircle size={16} /><span>{t('guide.apiUnavailable', { error })}</span></div>
  {/if}

  <div class="grid gap-3 sm:grid-cols-3">
    <div class="flex items-center gap-3 rounded-lg border border-border bg-card px-4 py-3">
      <div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-secondary"><KeyRound size={14} /></div>
      <div class="min-w-0"><p class="text-xs text-muted-foreground">{t('guide.adminToken')}</p><p class="text-sm font-semibold">{adminToken === '<admin-token>' ? t('waitServer') : t('dash.pageDisplayed')}</p></div>
    </div>
    <div class="flex items-center gap-3 rounded-lg border border-border bg-card px-4 py-3">
      <div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-secondary"><CheckCircle2 size={14} /></div>
      <div class="min-w-0"><p class="text-xs text-muted-foreground">{t('guide.recommendedCache')}</p><p class="text-sm font-semibold">{cacheName}</p></div>
    </div>
    <div class="flex items-center gap-3 rounded-lg border border-border bg-card px-4 py-3">
      <div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-secondary"><Terminal size={14} /></div>
      <div class="min-w-0"><p class="text-xs text-muted-foreground">{t('guide.cacheEndpoint')}</p><p class="text-sm font-semibold truncate">{endpoint}</p></div>
    </div>
  </div>

  <div class="space-y-4">
    {#each [
      { num: 1, title: t('guide.step1Title'), desc: t('guide.step1Desc'), body: t('guide.step1Body'), blocks: [] },
      { num: 2, title: t('guide.step2Title'), desc: t('guide.step2Desc'), body: null, blocks: [], extra: ['step2Item1', 'step2Item2', 'step2Item3'] },
      { num: 3, title: t('guide.step3Title'), desc: t('guide.step3Desc'), body: null, blocks: clientBlocks, extra: [] },
      { num: 4, title: t('guide.step4Title'), desc: t('guide.step4Desc'), body: null, blocks: pushBlocks, extra: [] },
      { num: 5, title: t('guide.step5Title'), desc: t('guide.step5Desc'), body: null, blocks: nixBlocks, extra: [] },
    ] as step}
      <Card>
        <CardContent class="p-5">
          <div class="flex items-start gap-3 mb-3">
            <div class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-secondary text-xs font-bold">{step.num}</div>
            <div><h2 class="text-base font-semibold">{step.title}</h2><p class="text-xs text-muted-foreground">{@html step.desc}</p></div>
          </div>
          {#if step.body}<p class="text-sm text-muted-foreground">{step.body}</p>{/if}
          {#if step.extra.length}
            <ol class="mt-2 pl-5 text-sm text-muted-foreground space-y-1 list-decimal">
              {#each step.extra as key}<li>{t(`guide.${key}`)}</li>{/each}
            </ol>
          {/if}
          {#each step.blocks as block}
            <div class="mt-2 overflow-hidden rounded-lg border border-border">
              <div class="flex items-center justify-between border-b border-border bg-muted/50 px-3 py-1.5"><span class="text-xs font-semibold text-muted-foreground">{block.label}</span><Button variant="ghost" size="sm" onclick={() => copyText(block.value, block.label)}><Clipboard size={13} /><span>{t('copy')}</span></Button></div>
              <pre class="overflow-x-auto bg-background px-3 py-2 text-xs leading-relaxed" style="font-family:var(--font-mono)"><code>{block.value}</code></pre>
            </div>
          {/each}
        </CardContent>
      </Card>
    {/each}

    <Card>
      <CardContent class="p-5 space-y-3">
        <div class="flex items-start gap-3"><div class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-secondary text-xs font-bold">6</div><div><h2 class="text-base font-semibold">{t('guide.step6Title')}</h2><p class="text-xs text-muted-foreground">{t('guide.step6Desc')}</p></div></div>
        <div class="grid gap-2 sm:grid-cols-2">
          {#each ['q1','q2','q3','q4'] as q}
            <div class="rounded-lg border border-border bg-muted/30 px-3 py-2.5"><p class="text-sm font-semibold">{t(`guide.${q}Title`)}</p><p class="text-xs text-muted-foreground">{@html t(`guide.${q}Desc`)}</p></div>
          {/each}
        </div>
      </CardContent>
    </Card>

    <Card>
      <CardContent class="p-5 space-y-3">
        <div class="flex items-start gap-3"><div class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-secondary"><Wrench size={13} /></div><div><h2 class="text-base font-semibold">{t('guide.copyValues')}</h2><p class="text-xs text-muted-foreground">{t('guide.copyValuesDesc')}</p></div></div>
        {#each valueBlocks as block}
          <div class="overflow-hidden rounded-lg border border-border">
            <div class="flex items-center justify-between border-b border-border bg-muted/50 px-3 py-1.5"><span class="text-xs font-semibold text-muted-foreground">{block.label}</span><Button variant="ghost" size="sm" onclick={() => copyText(block.value, block.label)}><Clipboard size={13} /><span>{t('copy')}</span></Button></div>
            <pre class="overflow-x-auto bg-background px-3 py-2 text-xs leading-relaxed" style="font-family:var(--font-mono)"><code>{block.value}</code></pre>
          </div>
        {/each}
      </CardContent>
    </Card>
  </div>
</div>

{#if copyMessage}<div class="toast" class:error={copyState === 'error'}>{copyMessage}</div>{/if}