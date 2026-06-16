<script lang="ts">
  import { cn } from '$lib/utils';
  let { class: className = '', children, ...rest }: { class?: string; children?: () => unknown; [key: string]: unknown } = $props();
</script>

  <div
  class={cn(
    'rounded-lg border border-border bg-card text-card-foreground shadow-sm transition-[background,border-color] duration-200',
    className
  )}
  {...rest}
>
  {@render children?.()}
</div>
