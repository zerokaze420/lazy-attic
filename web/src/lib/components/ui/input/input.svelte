<script lang="ts">
  import { cn } from '$lib/utils';
  let { class: className = '', ...rest }: { class?: string; [key: string]: unknown } = $props();
</script>

<input
  class={cn('flex h-11 w-full rounded-2xl border border-white/10 bg-white/5 px-4 py-2.5 text-[0.95rem] text-foreground outline-none backdrop-blur-md transition-all duration-300 placeholder:text-muted-foreground focus:border-white/20 focus:bg-white/8 focus:ring-2 focus:ring-white/10 disabled:cursor-not-allowed disabled:opacity-50', className)}
  {...rest}
/>
