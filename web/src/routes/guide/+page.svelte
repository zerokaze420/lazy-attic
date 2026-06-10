<script>
  import { onMount } from 'svelte';
  import {
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
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }
      summary = await response.json();
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
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

  $: caches = summary?.caches ?? [];
  $: cache = caches.find((item) => item.name === 'main') || caches.find((item) => item.is_public) || caches[0];
  $: cacheName = cache?.name || 'main';
  $: publicKey = cache?.public_key || '<public-key>';
  $: adminToken = summary?.admin_token?.token || '<admin-token>';
  $: endpoint = origin ? `${origin}/${cacheName}` : `https://<你的应用域名>/${cacheName}`;
  $: nixConf = `substituters = ${endpoint}
trusted-public-keys = ${publicKey}`;
  $: nixosConf = `nix.settings.substituters = [ "${endpoint}" ];
nix.settings.trusted-public-keys = [ "${publicKey}" ];`;
  $: loginCommand = `attic login lazy-attic ${origin || 'https://<你的应用域名>'}/ ${adminToken}`;
  $: useCommand = `attic use lazy-attic:${cacheName}`;
  $: pushCommand = `attic push lazy-attic:${cacheName} /nix/store/<path>`;
  $: clientBlocks = [
    { label: '登录命令', value: loginCommand },
    { label: '选择缓存', value: useCommand }
  ];
  $: pushBlocks = [
    { label: '推送示例', value: pushCommand },
    {
      label: '从当前 flake 推送结果',
      value: `nix build .#default --print-out-paths
attic push lazy-attic:${cacheName} ./result`
    }
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
  <meta name="description" content="LazyCat Attic 的初始化、推送、拉取和排错教程" />
</svelte:head>

<main class="page">
  <header class="hero">
    <a class="back" href="/">
      <ArrowLeft size={16} />
      <span>返回控制台</span>
    </a>
    <div class="hero-title">
      <div class="hero-mark"><Server size={24} /></div>
      <div>
        <p class="eyebrow">LazyCat Attic</p>
        <h1>使用教程</h1>
        <p class="lead">从首次打开页面到推送 Nix store path，再到配置机器拉取二进制缓存。</p>
      </div>
    </div>
  </header>

  {#if error}
    <section class="notice">
      控制台接口暂时不可用：{error}。教程仍可阅读，命令中的占位内容需要稍后回到控制台复制。
    </section>
  {/if}

  <section class="summary-grid">
    <article>
      <KeyRound size={20} />
      <span>管理员 Token</span>
      <strong>{adminToken === '<admin-token>' ? '等待服务返回' : '页面公开显示'}</strong>
    </article>
    <article>
      <CheckCircle2 size={20} />
      <span>推荐缓存</span>
      <strong>{cacheName}</strong>
    </article>
    <article>
      <Terminal size={20} />
      <span>缓存端点</span>
      <strong>{endpoint}</strong>
    </article>
  </section>

  <section class="content">
    <article class="section">
      <div class="section-head">
        <span>1</span>
        <div>
          <h2>首次打开控制台</h2>
          <p>这个 LPK 的路由已经设置为公开访问，懒猫不会再额外拦截页面或 API。进入应用后，首页会自动请求后端摘要并公开显示管理员 Token、缓存列表和 public key。</p>
        </div>
      </div>
      <p>如果首页没有显示 Token，先刷新页面；如果仍然为空，检查应用状态里 attic 服务是否在线。Token 用于创建缓存和执行客户端登录，当前打包版本默认认为前端访问已经由你的网络环境负责保护。</p>
    </article>

    <article class="section">
      <div class="section-head">
        <span>2</span>
        <div>
          <h2>创建第一个缓存</h2>
          <p>建议创建名为 <code>main</code> 的公开缓存，store 目录保持 <code>/nix/store</code>，优先级可以使用默认值。</p>
        </div>
      </div>
      <ol>
        <li>回到首页右侧的“创建缓存”。</li>
        <li>名称填写 <code>main</code>，保持“公开”勾选。</li>
        <li>点击“创建缓存”，列表刷新后会出现 public key。</li>
      </ol>
      <p>public key 是客户端信任缓存签名时必须使用的值。创建缓存时后端会生成签名 keypair，用户只需要复制页面展示的 public key。</p>
    </article>

    <article class="section">
      <div class="section-head">
        <span>3</span>
        <div>
          <h2>安装并登录 Attic 客户端</h2>
          <p>在要上传构建产物的机器上安装 Attic CLI，然后用页面公开的管理员 Token 登录。</p>
        </div>
      </div>
      {#each clientBlocks as block}
        <div class="code-block">
          <div class="code-head">
            <span>{block.label}</span>
            <button type="button" on:click={() => copyText(block.value, block.label)}>
              <Clipboard size={15} />
              <span>复制</span>
            </button>
          </div>
          <pre><code>{block.value}</code></pre>
        </div>
      {/each}
      <p>如果你不想把 Token 写进命令历史，可以先在 shell 中设置临时变量，再把命令最后的 Token 替换为变量引用。</p>
    </article>

    <article class="section">
      <div class="section-head">
        <span>4</span>
        <div>
          <h2>推送构建结果</h2>
          <p>推送对象必须是本机已经存在的 Nix store path。可以推送单个 derivation 输出，也可以一次推送多个路径。</p>
        </div>
      </div>
      {#each pushBlocks as block}
        <div class="code-block">
          <div class="code-head">
            <span>{block.label}</span>
            <button type="button" on:click={() => copyText(block.value, block.label)}>
              <Clipboard size={15} />
              <span>复制</span>
            </button>
          </div>
          <pre><code>{block.value}</code></pre>
        </div>
      {/each}
      <p>推送成功后，首页的对象、NAR 和分块数量会增加。若数量没有变化，刷新页面确认，或检查客户端命令是否推送到了同一个缓存名。</p>
    </article>

    <article class="section">
      <div class="section-head">
        <span>5</span>
        <div>
          <h2>配置 Nix 拉取缓存</h2>
          <p>其他机器只需要配置 substituter 和 trusted-public-keys，就可以从这个 Attic 缓存拉取二进制产物。</p>
        </div>
      </div>
      {#each nixBlocks as block}
        <div class="code-block">
          <div class="code-head">
            <span>{block.label}</span>
            <button type="button" on:click={() => copyText(block.value, block.label)}>
              <Clipboard size={15} />
              <span>复制</span>
            </button>
          </div>
          <pre><code>{block.value}</code></pre>
        </div>
      {/each}
      <p>修改系统配置后重启 nix-daemon，或者重新登录当前 shell。NixOS 可以通过 <code>sudo nixos-rebuild switch</code> 应用配置。</p>
    </article>

    <article class="section">
      <div class="section-head">
        <span>6</span>
        <div>
          <h2>常见问题排查</h2>
          <p>大部分问题都集中在域名、缓存名、Token 和 public key 不一致。</p>
        </div>
      </div>
      <div class="troubles">
        <div>
          <strong>401 Unauthorized</strong>
          <p>客户端没有登录，或者使用了旧 Token。回到首页复制当前 Token，重新执行 <code>attic login</code>。</p>
        </div>
        <div>
          <strong>找不到缓存</strong>
          <p>检查命令中的 <code>{cacheName}</code> 是否和首页列表完全一致。大小写和短横线都需要一致。</p>
        </div>
        <div>
          <strong>拉取时签名不可信</strong>
          <p>重新复制 public key，确认 Nix 配置里的 key 来自同一个缓存。</p>
        </div>
        <div>
          <strong>页面能打开但 API 失败</strong>
          <p>确认 LPK 是最新构建，并且 manifest 中 <code>public_path</code> 包含 <code>/</code>。</p>
        </div>
      </div>
    </article>

    <article class="section">
      <div class="section-head">
        <span><Wrench size={18} /></span>
        <div>
          <h2>当前页面可直接复制的值</h2>
          <p>这些值来自当前应用实例，适合直接粘贴到客户端或 Nix 配置。</p>
        </div>
      </div>
      {#each valueBlocks as block}
        <div class="code-block">
          <div class="code-head">
            <span>{block.label}</span>
            <button type="button" on:click={() => copyText(block.value, block.label)}>
              <Clipboard size={15} />
              <span>复制</span>
            </button>
          </div>
          <pre><code>{block.value}</code></pre>
        </div>
      {/each}
    </article>
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
      radial-gradient(circle at 18% 0%, rgba(23, 92, 211, 0.12), transparent 30rem),
      linear-gradient(180deg, #f6f8fb 0%, #edf2f7 100%);
    color: #17202a;
    font-family:
      Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont,
      "Segoe UI", sans-serif;
  }

  .page {
    width: min(1080px, calc(100% - 32px));
    margin: 0 auto;
    padding: 24px 0 48px;
  }

  .hero {
    display: grid;
    gap: 18px;
    margin-bottom: 18px;
  }

  .back,
  .hero-title,
  .summary-grid article,
  .section-head,
  .code-head,
  button {
    display: flex;
    align-items: center;
  }

  .back {
    width: fit-content;
    gap: 8px;
    min-height: 36px;
    border: 1px solid #cad2df;
    border-radius: 6px;
    padding: 0 12px;
    background: #fff;
    color: #263548;
    font-size: 0.92rem;
    font-weight: 750;
    text-decoration: none;
  }

  .hero-title {
    gap: 14px;
  }

  .hero-mark {
    display: grid;
    place-items: center;
    width: 48px;
    height: 48px;
    border: 1px solid #c8d7ea;
    border-radius: 8px;
    background: #fff;
    color: #175cd3;
    box-shadow: 0 14px 32px rgba(15, 34, 58, 0.08);
  }

  h1,
  h2,
  p {
    margin: 0;
  }

  h1 {
    margin-top: 3px;
    font-size: 2rem;
    letter-spacing: 0;
  }

  h2 {
    font-size: 1.08rem;
    letter-spacing: 0;
  }

  .eyebrow {
    color: #657080;
    font-size: 0.76rem;
    font-weight: 800;
    letter-spacing: 0;
    text-transform: uppercase;
  }

  .lead {
    margin-top: 8px;
    color: #526176;
    line-height: 1.55;
  }

  .notice {
    margin-bottom: 16px;
    border: 1px solid #f3b7b0;
    border-radius: 8px;
    padding: 13px 15px;
    background: #fff4f2;
    color: #7c251f;
  }

  .summary-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 12px;
    margin-bottom: 16px;
  }

  .summary-grid article,
  .section {
    border: 1px solid #dbe3ee;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.94);
    box-shadow: 0 18px 40px rgba(15, 34, 58, 0.08);
  }

  .summary-grid article {
    gap: 10px;
    min-width: 0;
    padding: 15px;
    color: #175cd3;
  }

  .summary-grid span {
    color: #667487;
    font-size: 0.86rem;
    font-weight: 750;
  }

  .summary-grid strong {
    min-width: 0;
    overflow: hidden;
    color: #17202a;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .content {
    display: grid;
    gap: 16px;
  }

  .section {
    display: grid;
    gap: 14px;
    padding: 18px;
  }

  .section-head {
    gap: 12px;
    align-items: flex-start;
  }

  .section-head > span {
    display: grid;
    place-items: center;
    flex: 0 0 auto;
    width: 32px;
    height: 32px;
    border-radius: 8px;
    background: #edf5ff;
    color: #175cd3;
    font-weight: 850;
  }

  .section-head p,
  .section > p,
  li,
  .troubles p {
    color: #526176;
    font-size: 0.94rem;
    line-height: 1.65;
  }

  ol {
    margin: 0;
    padding-left: 22px;
  }

  code,
  pre {
    font-family: "SFMono-Regular", Consolas, "Liberation Mono", monospace;
  }

  .code-block {
    overflow: hidden;
    border: 1px solid #dfe6ef;
    border-radius: 8px;
    background: #f8fafc;
  }

  .code-head {
    justify-content: space-between;
    gap: 10px;
    border-bottom: 1px solid #e6ebf2;
    padding: 9px 10px 9px 12px;
  }

  .code-head span {
    color: #526176;
    font-size: 0.86rem;
    font-weight: 800;
  }

  button {
    justify-content: center;
    gap: 7px;
    min-height: 32px;
    border: 1px solid #cad2df;
    border-radius: 6px;
    padding: 0 10px;
    background: #fff;
    color: #263548;
    font: inherit;
    font-size: 0.86rem;
    font-weight: 750;
    cursor: pointer;
  }

  pre {
    max-width: 100%;
    margin: 0;
    overflow-x: auto;
    padding: 12px;
    color: #263548;
    font-size: 0.84rem;
    line-height: 1.55;
  }

  .troubles {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 12px;
  }

  .troubles div {
    border: 1px solid #edf1f5;
    border-radius: 8px;
    padding: 13px;
    background: #fbfcfe;
  }

  .troubles strong {
    display: block;
    margin-bottom: 5px;
  }

  .section code:not(pre code) {
    border-radius: 5px;
    padding: 2px 5px;
    background: #eef4fb;
    color: #263548;
    font-size: 0.86em;
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

  @media (max-width: 820px) {
    .summary-grid,
    .troubles {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 560px) {
    .page {
      width: min(100% - 20px, 1080px);
      padding-top: 14px;
    }

    .hero-title {
      align-items: flex-start;
    }

    h1 {
      font-size: 1.6rem;
    }

    .section {
      padding: 15px;
    }

    .section-head {
      flex-direction: column;
    }
  }
</style>
