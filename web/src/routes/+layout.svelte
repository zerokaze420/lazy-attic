<script>
  import '../app.css';
  import { page } from '$app/stores';
  import { Database, BookOpen, Globe, Server } from '@lucide/svelte';
  import { t, getLocale, toggleLocale } from '$lib/i18n/index.svelte';

  $: langLabel = getLocale() === 'zh' ? 'EN' : '中文';
</script>

<div class="app-shell">
  <nav class="sidebar">
    <div class="sidebar-header">
      <div class="logo">
        <Server size={20} />
        <span>Attic</span>
      </div>
    </div>
    <div class="sidebar-nav">
      <a class="nav-item" class:active={$page.url.pathname === '/'} href="/">
        <Database size={16} />
        <span>{t('sidebar.dash')}</span>
      </a>
      <a class="nav-item" class:active={$page.url.pathname === '/guide'} href="/guide">
        <BookOpen size={16} />
        <span>{t('sidebar.guide')}</span>
      </a>
    </div>
    <div class="sidebar-footer">
      <div class="sidebar-footer-row">
        <p>{t('sidebar.footer')}</p>
        <button class="btn btn-ghost btn-sm lang-toggle" onclick={toggleLocale} title="Switch language">
          <Globe size={14} />
          <span>{langLabel}</span>
        </button>
      </div>
    </div>
  </nav>
  <main class="main-content">
    <slot />
  </main>
</div>

<style>
  .sidebar-footer-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .lang-toggle {
    height: 28px;
    padding: 0 8px;
    font-size: 0.7rem;
    gap: 4px;
  }
</style>
