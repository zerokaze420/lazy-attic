<script>
  import { onMount } from 'svelte';
  import {
    AlertCircle,
    ArrowLeft,
    CheckCircle2,
    Clipboard,
    KeyRound,
    Server,
    Terminal,
    Wrench
  } from '@lucide/svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { Button } from '$lib/components/ui/button';

  let summary = null;
  let error = '';
  let origin = '';
  let copyMessage = '';
  let copyState = '';

  onMount(() => {
    origin = location.origin;
    loadSummary();
  });

  async function loadSummary() {
    try {
      const response = await fetch('/_api/web/summary');
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      summary = await response.json();
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
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

  $: caches = summary?.caches ?? [];
  $: cache = caches.find((item) => item.name === 'main') || caches.find((item) => item.is_public) || caches[0];
  $: cacheName = cache?.name || 'main';
  $: publicKey = cache?.public_key || '<public-key>';
  $: adminToken = summary?.admin_token?.token || '<admin-token>';
  $: endpoint = origin ? `${origin}/${cacheName}` : `https://<your-domain>/${cacheName}`;
  $: nixConf = `substituters = ${endpoint}\ntrusted-public-keys = ${publicKey}`;
  $: nixosConf = `nix.settings.substituters = [ "${endpoint}" ];\nnix.settings.trusted-public-keys = [ "${publicKey}" ];`;
  $: loginCommand = `attic login lazy-attic ${origin || 'https://<your-domain>'}/ ${adminToken}`;
  $: useCommand = `attic use lazy-attic:${cacheName}`;
  $: pushCommand = `attic push lazy-attic:${cacheName} /nix/store/<path>`;
  $: clientBlocks = [
    { label: t('guide.loginCommand'), value: loginCommand },
    { label: t('guide.selectCache'), value: useCommand }
  ];
  $: pushBlocks = [
    { label: t('guide.pushExample'), value: pushCommand },
    { label: t('guide.pushFlake'), value: `nix build .#default --print-out-paths\nattic push lazy-attic:${cacheName} ./result` }
  ];
  $: nixBlocks = [
    { label: t('guide.nixConf'), value: nixConf },
    { label: t('guide.nixosConfig'), value: nixosConf }
  ];
  $: valueBlocks = [
    { label: t('guide.adminTokenValue'), value: adminToken },
    { label: t('guide.publicKeyValue'), value: publicKey },
    { label: t('guide.cacheEndpointValue'), value: endpoint }
  ];
</script>

<svelte:head>
  <title>{t('guide.title')}</title>
</svelte:head>

<div class="page-header">
  <div class="guide-hero">
    <div class="guide-hero-top">
      <Button variant="ghost" size="icon" class="size-8" href="/">
        <ArrowLeft size={16} />
      </Button>
    </div>
    <div class="guide-hero-title">
      <div class="icon-box"><Server size={20} /></div>
      <div>
        <h1>{t('guide.headTitle')}</h1>
        <p>{t('guide.headDesc')}</p>
      </div>
    </div>
  </div>
</div>

<div class="page-body">
  {#if error}
    <div class="notice">
      <AlertCircle size={16} />
      <span>{t('guide.apiUnavailable', { error })}</span>
    </div>
  {/if}

  <div class="guide-stats">
    <div class="guide-stat">
      <div class="stat-icon"><KeyRound size={16} /></div>
      <div class="stat-content">
        <span class="stat-label">{t('guide.adminToken')}</span>
        <span class="stat-value">{adminToken === '<admin-token>' ? t('waitServer') : t('dash.pageDisplayed')}</span>
      </div>
    </div>
    <div class="guide-stat">
      <div class="stat-icon"><CheckCircle2 size={16} /></div>
      <div class="stat-content">
        <span class="stat-label">{t('guide.recommendedCache')}</span>
        <span class="stat-value">{cacheName}</span>
      </div>
    </div>
    <div class="guide-stat">
      <div class="stat-icon"><Terminal size={16} /></div>
      <div class="stat-content">
        <span class="stat-label">{t('guide.cacheEndpoint')}</span>
        <span class="stat-value stat-value-sm">{endpoint}</span>
      </div>
    </div>
  </div>

  <div class="guide-sections">
    <article class="card guide-section">
      <div class="guide-section-head">
        <span class="guide-section-num">1</span>
        <div>
          <h2>{t('guide.step1Title')}</h2>
          <p>{t('guide.step1Desc')}</p>
        </div>
      </div>
      <p>{t('guide.step1Body')}</p>
    </article>

    <article class="card guide-section">
      <div class="guide-section-head">
        <span class="guide-section-num">2</span>
        <div>
          <h2>{t('guide.step2Title')}</h2>
          <p>{@html t('guide.step2Desc')}</p>
        </div>
      </div>
      <ol>
        <li>{t('guide.step2Item1')}</li>
        <li>{t('guide.step2Item2')}</li>
        <li>{t('guide.step2Item3')}</li>
      </ol>
    </article>

    <article class="card guide-section">
      <div class="guide-section-head">
        <span class="guide-section-num">3</span>
        <div>
          <h2>{t('guide.step3Title')}</h2>
          <p>{t('guide.step3Desc')}</p>
        </div>
      </div>
      {#each clientBlocks as block}
        <div class="code-block mb-sm">
          <div class="code-block-header">
            <span>{block.label}</span>
            <Button variant="ghost" size="sm" type="button" on:click={() => copyText(block.value, block.label)}>
              <Clipboard size={13} />
              <span>{t('copy')}</span>
            </Button>
          </div>
          <pre><code>{block.value}</code></pre>
        </div>
      {/each}
    </article>

    <article class="card guide-section">
      <div class="guide-section-head">
        <span class="guide-section-num">4</span>
        <div>
          <h2>{t('guide.step4Title')}</h2>
          <p>{t('guide.step4Desc')}</p>
        </div>
      </div>
      {#each pushBlocks as block}
        <div class="code-block mb-sm">
          <div class="code-block-header">
            <span>{block.label}</span>
            <Button variant="ghost" size="sm" type="button" on:click={() => copyText(block.value, block.label)}>
              <Clipboard size={13} />
              <span>{t('copy')}</span>
            </Button>
          </div>
          <pre><code>{block.value}</code></pre>
        </div>
      {/each}
    </article>

    <article class="card guide-section">
      <div class="guide-section-head">
        <span class="guide-section-num">5</span>
        <div>
          <h2>{t('guide.step5Title')}</h2>
          <p>{t('guide.step5Desc')}</p>
        </div>
      </div>
      {#each nixBlocks as block}
        <div class="code-block mb-sm">
          <div class="code-block-header">
            <span>{block.label}</span>
            <Button variant="ghost" size="sm" type="button" on:click={() => copyText(block.value, block.label)}>
              <Clipboard size={13} />
              <span>{t('copy')}</span>
            </Button>
          </div>
          <pre><code>{block.value}</code></pre>
        </div>
      {/each}
    </article>

    <article class="card guide-section">
      <div class="guide-section-head">
        <span class="guide-section-num">6</span>
        <div>
          <h2>{t('guide.step6Title')}</h2>
          <p>{t('guide.step6Desc')}</p>
        </div>
      </div>
      <div class="troubleshoot-grid">
        <div class="troubleshoot-item">
          <strong>{t('guide.q1Title')}</strong>
          <p>{@html t('guide.q1Desc')}</p>
        </div>
        <div class="troubleshoot-item">
          <strong>{t('guide.q2Title')}</strong>
          <p>{t('guide.q2Desc')}</p>
        </div>
        <div class="troubleshoot-item">
          <strong>{t('guide.q3Title')}</strong>
          <p>{@html t('guide.q3Desc')}</p>
        </div>
        <div class="troubleshoot-item">
          <strong>{t('guide.q4Title')}</strong>
          <p>{@html t('guide.q4Desc')}</p>
        </div>
      </div>
    </article>

    <article class="card guide-section">
      <div class="guide-section-head">
        <span class="guide-section-num"><Wrench size={14} /></span>
        <div>
          <h2>{t('guide.copyValues')}</h2>
          <p>{t('guide.copyValuesDesc')}</p>
        </div>
      </div>
      {#each valueBlocks as block}
        <div class="code-block mb-sm">
          <div class="code-block-header">
            <span>{block.label}</span>
            <Button variant="ghost" size="sm" type="button" on:click={() => copyText(block.value, block.label)}>
              <Clipboard size={13} />
              <span>{t('copy')}</span>
            </Button>
          </div>
          <pre><code>{block.value}</code></pre>
        </div>
      {/each}
    </article>
  </div>
</div>

{#if copyMessage}
  <div class="toast" class:error={copyState === 'error'}>{copyMessage}</div>
{/if}
