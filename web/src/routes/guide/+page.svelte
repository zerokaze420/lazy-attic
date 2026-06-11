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

  let summary = null;
  let error = '';
  let origin = '';
  let copyMessage = '';

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
    await navigator.clipboard.writeText(value);
    copyMessage = `${label} 已复制`;
    setTimeout(() => { copyMessage = ''; }, 1800);
  }

  $: caches = summary?.caches ?? [];
  $: cache = caches.find((item) => item.name === 'main') || caches.find((item) => item.is_public) || caches[0];
  $: cacheName = cache?.name || 'main';
  $: publicKey = cache?.public_key || '<public-key>';
  $: adminToken = summary?.admin_token?.token || '<admin-token>';
  $: endpoint = origin ? `${origin}/${cacheName}` : `https://<你的应用域名>/${cacheName}`;
  $: nixConf = `substituters = ${endpoint}\ntrusted-public-keys = ${publicKey}`;
  $: nixosConf = `nix.settings.substituters = [ "${endpoint}" ];\nnix.settings.trusted-public-keys = [ "${publicKey}" ];`;
  $: loginCommand = `attic login lazy-attic ${origin || 'https://<你的应用域名>'}/ ${adminToken}`;
  $: useCommand = `attic use lazy-attic:${cacheName}`;
  $: pushCommand = `attic push lazy-attic:${cacheName} /nix/store/<path>`;
  $: clientBlocks = [
    { label: '登录命令', value: loginCommand },
    { label: '选择缓存', value: useCommand }
  ];
  $: pushBlocks = [
    { label: '推送示例', value: pushCommand },
    { label: '从当前 flake 推送结果', value: `nix build .#default --print-out-paths\nattic push lazy-attic:${cacheName} ./result` }
  ];
  $: nixBlocks = [
    { label: '/etc/nix/nix.conf', value: nixConf },
    { label: 'NixOS 配置', value: nixosConf }
  ];
  $: valueBlocks = [
    { label: '管理员 Token', value: adminToken },
    { label: '缓存 public key', value: publicKey },
    { label: '缓存地址', value: endpoint }
  ];
</script>

<svelte:head>
  <title>Attic 使用教程</title>
</svelte:head>

<div class="page-header">
  <div class="guide-hero">
    <div class="guide-hero-top">
      <a class="btn btn-ghost btn-sm btn-icon" href="/">
        <ArrowLeft size={16} />
      </a>
    </div>
    <div class="guide-hero-title">
      <div class="icon-box"><Server size={20} /></div>
      <div>
        <h1>使用教程</h1>
        <p>从首次打开页面到推送 Nix store path，再到配置机器拉取二进制缓存。</p>
      </div>
    </div>
  </div>
</div>

<div class="page-body">
  {#if error}
    <div class="notice">
      <AlertCircle size={16} />
      <span>控制台接口暂时不可用：{error}。教程仍可阅读，命令中的占位内容需要稍后回到控制台复制。</span>
    </div>
  {/if}

  <div class="guide-stats">
    <div class="guide-stat">
      <div class="stat-icon"><KeyRound size={16} /></div>
      <div class="stat-content">
        <span class="stat-label">管理员 Token</span>
        <span class="stat-value">{adminToken === '<admin-token>' ? '等待服务返回' : '页面公开显示'}</span>
      </div>
    </div>
    <div class="guide-stat">
      <div class="stat-icon"><CheckCircle2 size={16} /></div>
      <div class="stat-content">
        <span class="stat-label">推荐缓存</span>
        <span class="stat-value">{cacheName}</span>
      </div>
    </div>
    <div class="guide-stat">
      <div class="stat-icon"><Terminal size={16} /></div>
      <div class="stat-content">
        <span class="stat-label">缓存端点</span>
        <span class="stat-value" style="font-size:0.78rem;">{endpoint}</span>
      </div>
    </div>
  </div>

  <div class="guide-sections">
    <article class="card guide-section">
      <div class="guide-section-head">
        <span class="guide-section-num">1</span>
        <div>
          <h2>首次打开控制台</h2>
          <p>LPK 的路由已经设置为公开访问，进入应用后首页会自动请求后端摘要并公开显示管理员 Token、缓存列表和 public key。</p>
        </div>
      </div>
      <p>如果首页没有显示 Token，先刷新页面；如果仍然为空，检查 attic 服务是否在线。Token 用于创建缓存和执行客户端登录。</p>
    </article>

    <article class="card guide-section">
      <div class="guide-section-head">
        <span class="guide-section-num">2</span>
        <div>
          <h2>创建第一个缓存</h2>
          <p>建议创建名为 <code class="code-inline">main</code> 的公开缓存，store 目录保持 <code class="code-inline">/nix/store</code>，优先级可以使用默认值。</p>
        </div>
      </div>
      <ol>
        <li>回到首页右侧的"创建缓存"。</li>
        <li>名称填写 <code class="code-inline">main</code>，保持"公开"勾选。</li>
        <li>点击"创建缓存"，列表刷新后会出现 public key。</li>
      </ol>
    </article>

    <article class="card guide-section">
      <div class="guide-section-head">
        <span class="guide-section-num">3</span>
        <div>
          <h2>安装并登录 Attic 客户端</h2>
          <p>在要上传构建产物的机器上安装 Attic CLI，然后用页面公开的管理员 Token 登录。</p>
        </div>
      </div>
      {#each clientBlocks as block}
        <div class="code-block" style="margin-bottom:10px;">
          <div class="code-block-header">
            <span>{block.label}</span>
            <button class="btn btn-ghost btn-sm" type="button" on:click={() => copyText(block.value, block.label)}>
              <Clipboard size={13} />
              <span>复制</span>
            </button>
          </div>
          <pre><code>{block.value}</code></pre>
        </div>
      {/each}
    </article>

    <article class="card guide-section">
      <div class="guide-section-head">
        <span class="guide-section-num">4</span>
        <div>
          <h2>推送构建结果</h2>
          <p>推送对象必须是本机已经存在的 Nix store path。可以推送单个 derivation 输出，也可以一次推送多个路径。</p>
        </div>
      </div>
      {#each pushBlocks as block}
        <div class="code-block" style="margin-bottom:10px;">
          <div class="code-block-header">
            <span>{block.label}</span>
            <button class="btn btn-ghost btn-sm" type="button" on:click={() => copyText(block.value, block.label)}>
              <Clipboard size={13} />
              <span>复制</span>
            </button>
          </div>
          <pre><code>{block.value}</code></pre>
        </div>
      {/each}
    </article>

    <article class="card guide-section">
      <div class="guide-section-head">
        <span class="guide-section-num">5</span>
        <div>
          <h2>配置 Nix 拉取缓存</h2>
          <p>其他机器只需要配置 substituter 和 trusted-public-keys，就可以从这个 Attic 缓存拉取二进制产物。</p>
        </div>
      </div>
      {#each nixBlocks as block}
        <div class="code-block" style="margin-bottom:10px;">
          <div class="code-block-header">
            <span>{block.label}</span>
            <button class="btn btn-ghost btn-sm" type="button" on:click={() => copyText(block.value, block.label)}>
              <Clipboard size={13} />
              <span>复制</span>
            </button>
          </div>
          <pre><code>{block.value}</code></pre>
        </div>
      {/each}
    </article>

    <article class="card guide-section">
      <div class="guide-section-head">
        <span class="guide-section-num">6</span>
        <div>
          <h2>常见问题排查</h2>
          <p>大部分问题都集中在域名、缓存名、Token 和 public key 不一致。</p>
        </div>
      </div>
      <div class="troubleshoot-grid">
        <div class="troubleshoot-item">
          <strong>401 Unauthorized</strong>
          <p>客户端没有登录，或者使用了旧 Token。回到首页复制当前 Token，重新执行 <code class="code-inline">attic login</code>。</p>
        </div>
        <div class="troubleshoot-item">
          <strong>找不到缓存</strong>
          <p>检查命令中的缓存名是否和首页列表完全一致。大小写和短横线都需要一致。</p>
        </div>
        <div class="troubleshoot-item">
          <strong>拉取时签名不可信</strong>
          <p>重新复制 public key，确认 Nix 配置里的 key 来自同一个缓存。</p>
        </div>
        <div class="troubleshoot-item">
          <strong>页面能打开但 API 失败</strong>
          <p>确认 LPK 是最新构建，并且 manifest 中 <code class="code-inline">public_path</code> 包含 <code class="code-inline">/</code>。</p>
        </div>
      </div>
    </article>

    <article class="card guide-section">
      <div class="guide-section-head">
        <span class="guide-section-num"><Wrench size={14} /></span>
        <div>
          <h2>当前页面可直接复制的值</h2>
          <p>这些值来自当前应用实例，适合直接粘贴到客户端或 Nix 配置。</p>
        </div>
      </div>
      {#each valueBlocks as block}
        <div class="code-block" style="margin-bottom:10px;">
          <div class="code-block-header">
            <span>{block.label}</span>
            <button class="btn btn-ghost btn-sm" type="button" on:click={() => copyText(block.value, block.label)}>
              <Clipboard size={13} />
              <span>复制</span>
            </button>
          </div>
          <pre><code>{block.value}</code></pre>
        </div>
      {/each}
    </article>
  </div>
</div>

{#if copyMessage}
  <div class="toast">{copyMessage}</div>
{/if}
