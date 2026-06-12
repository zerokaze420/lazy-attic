<script>
  import '../app.css';
  import { page } from '$app/stores';
  import { Database, BookOpen, Server } from '@lucide/svelte';
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
        <span>Dashboard</span>
      </a>
      <a class="nav-item" class:active={$page.url.pathname === '/guide'} href="/guide">
        <BookOpen size={16} />
        <span>Guide</span>
      </a>
    </div>
    <div class="sidebar-footer">
      <p>LazyCat Attic</p>
    </div>
  </nav>
  <main class="main-content">
    <slot />
  </main>
</div>
