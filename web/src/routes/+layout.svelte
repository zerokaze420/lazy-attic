<script>
  import '../app.css';
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { BookOpen, Database, Globe, Moon, Server, Sun } from '@lucide/svelte';
  import { t, locale, toggleLocale } from '$lib/i18n/index.svelte';
  import { Button } from '$lib/components/ui/button';

  $: langLabel = $locale === 'zh' ? 'EN' : '中文';
  let theme = 'dark';

  function applyTheme(nextTheme) {
    theme = nextTheme;
    if (typeof document !== 'undefined') {
      document.documentElement.dataset.theme = nextTheme;
      document.documentElement.lang = $locale;
    }
    try { localStorage.setItem('attic.theme', nextTheme); } catch {}
  }

  function toggleTheme() {
    applyTheme(theme === 'dark' ? 'light' : 'dark');
  }

  onMount(() => {
    try {
      const stored = localStorage.getItem('attic.theme');
      if (stored === 'light' || stored === 'dark') {
        applyTheme(stored);
      }
    } catch {}
  });

  $: if (typeof document !== 'undefined') {
    document.documentElement.lang = $locale;
  }
</script>

{#key $locale}
  <div class="app-shell">
    <header class="topbar">
      <div class="topbar-start">
        <a href="/" class="topbar-logo">
          <Server size={20} />
          <span>Attic</span>
        </a>
        <nav class="topbar-nav">
          <a class="topbar-link" class:active={$page.url.pathname === '/'} href="/">
            <Database size={16} />
            <span>{t('sidebar.dash')}</span>
          </a>
          <a class="topbar-link" class:active={$page.url.pathname === '/guide'} href="/guide">
            <BookOpen size={16} />
            <span>{t('sidebar.guide')}</span>
          </a>
        </nav>
      </div>
      <div class="topbar-end">
        <span class="topbar-version">{t('sidebar.footer')}</span>
        <Button variant="ghost" size="icon" onclick={toggleTheme} title="Toggle theme">
          {#if theme === 'dark'}
            <Sun size={15} />
          {:else}
            <Moon size={15} />
          {/if}
        </Button>
        <Button variant="ghost" size="sm" onclick={toggleLocale} title="Switch language">
          <Globe size={15} />
          <span>{langLabel}</span>
        </Button>
      </div>
    </header>
    <main class="main-content">
      <slot />
    </main>
  </div>
{/key}