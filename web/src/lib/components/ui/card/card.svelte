<script lang="ts">
  import { cn } from '$lib/utils';
  let { class: className = '', children, ...rest }: { class?: string; children?: () => unknown; [key: string]: unknown } = $props();
</script>

  <div
  class={cn(
    'rounded-3xl border border-white/10 bg-white/5 text-card-foreground shadow-sm backdrop-blur-md animate-slide-up transition-all duration-300 hover:border-white/15 hover:bg-white/[0.07]',
    className
  )}
  {...rest}
>
  {@render children?.()}
</div>
