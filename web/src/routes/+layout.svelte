<script>
  import '../app.css';
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { BookOpen, Database, Globe, Moon, Server, Sun } from '@lucide/svelte';
  import { t, locale, toggleLocale } from '$lib/i18n/index.svelte';
  import { Button } from '$lib/components/ui/button';

  let langLabel = $derived($locale === 'zh' ? 'EN' : '中文');
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
      if (stored === 'light' || stored === 'dark') applyTheme(stored);
    } catch {}
  });

  $effect(() => {
    if (typeof document !== 'undefined') {
      document.documentElement.lang = $locale;
    }
  });
</script>

{#key $locale}
  <div class="flex min-h-screen flex-col">
    <header class="sticky top-0 z-30 flex h-13 shrink-0 items-center justify-between gap-4 border-b border-border bg-card px-5">
      <div class="flex items-center gap-5 min-w-0">
        <a href="/" class="flex shrink-0 items-center gap-2 text-base font-bold text-foreground">
          <Server size={18} />
          <span>Attic</span>
        </a>
        <nav class="flex items-center gap-1">
          <a href="/" class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 text-sm font-medium transition-colors {($page.url.pathname === '/') ? 'bg-secondary text-foreground' : 'text-muted-foreground hover:bg-secondary hover:text-foreground'}">
            <Database size={15} />
            <span>{t('sidebar.dash')}</span>
          </a>
          <a href="/guide" class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 text-sm font-medium transition-colors {($page.url.pathname === '/guide') ? 'bg-secondary text-foreground' : 'text-muted-foreground hover:bg-secondary hover:text-foreground'}">
            <BookOpen size={15} />
            <span>{t('sidebar.guide')}</span>
          </a>
        </nav>
      </div>
      <div class="flex shrink-0 items-center gap-2">
        <span class="hidden text-xs text-muted-foreground sm:inline">{t('sidebar.footer')}</span>
        <Button variant="ghost" size="icon" onclick={toggleTheme} title="Toggle theme">
          {#if theme === 'dark'}<Sun size={15} />{:else}<Moon size={15} />{/if}
        </Button>
        <Button variant="ghost" size="sm" onclick={toggleLocale} title="Switch language">
          <Globe size={15} />
          <span>{langLabel}</span>
        </Button>
      </div>
    </header>
    <main class="flex-1">
      <slot />
    </main>
  </div>
{/key}