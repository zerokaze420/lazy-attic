<script>
  import '../app.css';
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { Database, Globe, Moon, Server, Sun } from '@lucide/svelte';
  import { t, locale, toggleLocale } from '$lib/i18n/index.svelte';
  import { Button } from '$lib/components/ui/button';

  $: langLabel = $locale === 'zh' ? 'EN' : '中文';
  let theme = 'dark';

  function applyTheme(nextTheme) {
    theme = nextTheme;
    if (typeof document !== 'undefined') {
      document.documentElement.dataset.theme = nextTheme;
      document.documentElement.classList.toggle('dark', nextTheme === 'dark');
      document.documentElement.lang = $locale;
    }
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('attic.theme', nextTheme);
    }
  }

  function toggleTheme() {
    applyTheme(theme === 'dark' ? 'light' : 'dark');
  }

  onMount(() => {
    const stored = localStorage.getItem('attic.theme');
    applyTheme(stored === 'light' || stored === 'dark' ? stored : 'dark');
  });

  $: if (typeof document !== 'undefined') {
    document.documentElement.lang = $locale;
  }
</script>

{#key $locale}
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
      </div>
      <div class="sidebar-footer">
        <div class="sidebar-footer-row">
          <p>{t('sidebar.footer')}</p>
          <div class="sidebar-actions">
            <Button variant="ghost" size="icon" class="sidebar-action-button" onclick={toggleTheme} title="Toggle theme">
              {#if theme === 'dark'}
                <Sun size={14} />
              {:else}
                <Moon size={14} />
              {/if}
            </Button>
            <Button variant="ghost" size="sm" class="lang-toggle" onclick={toggleLocale} title="Switch language">
              <Globe size={14} />
              <span>{langLabel}</span>
            </Button>
          </div>
        </div>
      </div>
    </nav>
    <main class="main-content">
      <slot />
    </main>
  </div>
{/key}
