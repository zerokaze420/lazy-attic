<script lang="ts">
  import { cn } from '$lib/utils';
  let { class: className = '', ...rest }: { class?: string; [key: string]: unknown } = $props();
</script>

<textarea
  class={cn('flex min-h-24 w-full rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-[0.95rem] text-foreground outline-none backdrop-blur-md transition-all duration-300 placeholder:text-muted-foreground focus:border-violet-400/40 focus:bg-white/10 focus:ring-2 focus:ring-violet-500/20 disabled:cursor-not-allowed disabled:opacity-50', className)}
  {...rest}
></textarea>
